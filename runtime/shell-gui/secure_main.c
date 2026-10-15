#define _POSIX_C_SOURCE 200809L

#include "auth_bounds.h"
#include "boot_splash.h"
#include "fofline_api_proxy.h"
#include "native_bridge.h"

#include <glib-unix.h>
#include <glib/gstdio.h>
#include <gtk/gtk.h>
#include <jsc/jsc.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <syslog.h>
#include <webkit2/webkit2.h>

#define SHELL_URI "empiier://shell/index.html"
#define TRUSTED_URI_PREFIX "empiier://shell/"
#define SHELL_ROOT "/opt/empiieros/shell"
#define PACK_ROOT "/var/lib/empiieros-runtime/fofline-spa/current"
#define MAX_PACK_ASSET_BYTES (256U * 1024U * 1024U)
#define RUNTIME_SOCKET "/run/empiieros-shell/runtime.sock"
#define WINDOW_TITLE "EmpiierOS"
#define WINDOW_ROLE "empiieros-shell-gui"
#define MAX_SHELL_ASSET_BYTES (16U * 1024U * 1024U)
#define RENDER_PROBE_INTERVAL_MS 250U
#define RENDER_PROBE_MAX_ATTEMPTS 40U

static EmpiierNativeBridge bridge;

typedef struct {
    WebKitWebView *web_view;
    guint attempts;
} ShellRenderProbe;

static gboolean stop_main_loop(gpointer user_data) {
    (void)user_data;
    gtk_main_quit();
    return G_SOURCE_REMOVE;
}

static gboolean suppress_context_menu(
    WebKitWebView *web_view,
    WebKitContextMenu *menu,
    GdkEvent *event,
    WebKitHitTestResult *hit_test,
    gpointer user_data
) {
    (void)web_view;
    (void)menu;
    (void)event;
    (void)hit_test;
    (void)user_data;
    return TRUE;
}

static gboolean deny_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
) {
    (void)web_view;
    (void)user_data;
    webkit_permission_request_deny(request);
    return TRUE;
}

static gboolean run_render_probe(gpointer user_data);

static void on_render_probe_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    ShellRenderProbe *probe = user_data;
    GError *error = NULL;
    JSCValue *value = webkit_web_view_evaluate_javascript_finish(
        WEBKIT_WEB_VIEW(source_object),
        result,
        &error
    );
    char *diagnostic = value != NULL && jsc_value_is_string(value)
        ? jsc_value_to_string(value)
        : NULL;
    if (g_strcmp0(diagnostic, "READY") == 0) {
        syslog(LOG_INFO, "shell render ready attempts=%u", probe->attempts);
        g_object_unref(probe->web_view);
        g_free(probe);
    } else if (probe->attempts >= RENDER_PROBE_MAX_ATTEMPTS) {
        syslog(
            LOG_ERR,
            "shell render probe failed after %u attempts: %s%s%s",
            probe->attempts,
            error != NULL ? error->message : "React root is empty",
            diagnostic != NULL ? " diagnostics=" : "",
            diagnostic != NULL ? diagnostic : ""
        );
        g_object_unref(probe->web_view);
        g_free(probe);
    } else {
        g_timeout_add(
            RENDER_PROBE_INTERVAL_MS,
            run_render_probe,
            probe
        );
    }
    g_free(diagnostic);
    g_clear_object(&value);
    g_clear_error(&error);
}

static gboolean run_render_probe(gpointer user_data) {
    ShellRenderProbe *probe = user_data;
    probe->attempts += 1U;
    webkit_web_view_evaluate_javascript(
        probe->web_view,
        "(() => {"
        "const root=document.getElementById('root');"
        "if(root&&root.childElementCount>0"
        "&&getComputedStyle(document.body).backgroundImage!=='none')return 'READY';"
        "return JSON.stringify({readyState:document.readyState,"
        "rootChildren:root?root.childElementCount:null,"
        "bodyBackground:getComputedStyle(document.body).backgroundImage,"
        "scripts:[...document.scripts].map(script=>script.src),"
        "styles:[...document.styleSheets].map(sheet=>sheet.href),"
        "resources:performance.getEntriesByType('resource').map(entry=>entry.name),"
        "errors:window.__empiierErrors||[]});"
        "})()",
        -1,
        NULL,
        "empiieros-render-probe",
        NULL,
        on_render_probe_finished,
        probe
    );
    return G_SOURCE_REMOVE;
}

static void on_fofline_boot_probe_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)user_data;
    GError *error = NULL;
    JSCValue *value = webkit_web_view_evaluate_javascript_finish(
        WEBKIT_WEB_VIEW(source_object),
        result,
        &error
    );
    char *diagnostic = value != NULL && jsc_value_is_string(value)
        ? jsc_value_to_string(value)
        : NULL;
    if (diagnostic != NULL
        && strstr(diagnostic, "eyJ") == NULL
        && strstr(diagnostic, "Bearer") == NULL) {
        syslog(LOG_INFO, "fofline boot %s", diagnostic);
    } else {
        syslog(
            LOG_WARNING,
            "fofline boot probe failed: %s",
            error != NULL ? error->message : "empty"
        );
    }
    g_free(diagnostic);
    g_clear_object(&value);
    g_clear_error(&error);
}

static gboolean run_fofline_boot_probe(gpointer user_data) {
    WebKitWebView *web_view = user_data;
    webkit_web_view_evaluate_javascript(
        web_view,
        "(()=>{try{"
        "var b=window.EmperyDesktopNativeOidc;"
        "var ok=false;"
        "if(b&&b.getBootstrapJson){try{var j=JSON.parse(b.getBootstrapJson());"
        "ok=!!(j&&j.ok&&(j.token||j.accessToken));}catch(e){ok=false;}}"
        "var root=document.getElementById('root');"
        "var tx=String((root&&root.innerText)||'').replace(/\\s+/g,' ');"
        "if(tx.indexOf('eyJ')!==-1||tx.indexOf('Bearer')!==-1)tx='redacted';"
        "return JSON.stringify({"
        "p:String(location.protocol||''),"
        "o:String(location.origin||''),"
        "q:String(location.search||'').indexOf('eliteewFoflineClient=1')!==-1,"
        "n:ok,"
        "r:!!(root&&root.childElementCount>0),"
        "pre:!!document.getElementById('app-preloader'),"
        "css:document.styleSheets.length,"
        "h:String(location.hash||'').slice(0,24),"
        "tx:tx.slice(0,72),"
        "cn:/connect|opening|подключ|открываем/i.test(tx),"
        "err:String(window.__empiierBootErr||''),"
        "es:String(window.__empiierBootSrc||'')"
        "});}catch(e){return '{\"err\":true}';}})()",
        -1,
        NULL,
        "empiieros-fofline-boot",
        NULL,
        on_fofline_boot_probe_finished,
        NULL
    );
    g_object_unref(web_view);
    return G_SOURCE_REMOVE;
}

static void on_load_changed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    gpointer user_data
) {
    (void)user_data;
    if (load_event != WEBKIT_LOAD_FINISHED) {
        return;
    }
    const char *uri = webkit_web_view_get_uri(web_view);
    if (empiier_native_bridge_fofline_uri(uri)) {
        g_timeout_add(400U, run_fofline_boot_probe, g_object_ref(web_view));
        g_timeout_add(2500U, run_fofline_boot_probe, g_object_ref(web_view));
        g_timeout_add(15000U, run_fofline_boot_probe, g_object_ref(web_view));
        g_timeout_add(45000U, run_fofline_boot_probe, g_object_ref(web_view));
        return;
    }
    ShellRenderProbe *probe = g_new0(ShellRenderProbe, 1U);
    probe->web_view = g_object_ref(web_view);
    g_timeout_add(RENDER_PROBE_INTERVAL_MS, run_render_probe, probe);
}

static void finish_scheme_error(
    WebKitURISchemeRequest *request,
    GIOErrorEnum code,
    const char *message
) {
    GError *error = g_error_new_literal(G_IO_ERROR, code, message);
    syslog(
        LOG_ERR,
        "Shell scheme failure uri=%s error=%s",
        webkit_uri_scheme_request_get_uri(request),
        message
    );
    webkit_uri_scheme_request_finish_error(request, error);
    g_error_free(error);
}

static void finish_scheme_ok(
    WebKitURISchemeRequest *request,
    GInputStream *stream,
    gint64 length,
    const char *mime_type
) {
    WebKitURISchemeResponse *response = webkit_uri_scheme_response_new(stream, length);
    SoupMessageHeaders *headers = soup_message_headers_new(SOUP_MESSAGE_HEADERS_RESPONSE);
    webkit_uri_scheme_response_set_status(response, 200U, NULL);
    soup_message_headers_append(headers, "Access-Control-Allow-Origin", "*");
    soup_message_headers_append(headers, "Access-Control-Allow-Methods", "GET");
    soup_message_headers_append(headers, "X-Content-Type-Options", "nosniff");
    webkit_uri_scheme_response_set_http_headers(response, headers);
    webkit_uri_scheme_response_set_content_type(
        response,
        mime_type != NULL ? mime_type : "application/octet-stream"
    );
    webkit_uri_scheme_request_finish_with_response(request, response);
    g_object_unref(response);
}

static gboolean mime_is_stylesheet(const char *mime_type, const char *path) {
    if (g_strcmp0(mime_type, "text/css") == 0) {
        return TRUE;
    }
    if (path == NULL) {
        return FALSE;
    }
    size_t length = strlen(path);
    return length >= 4U && g_ascii_strcasecmp(path + length - 4U, ".css") == 0;
}

static gboolean scheme_request_is_cors(WebKitURISchemeRequest *request) {
    SoupMessageHeaders *headers = webkit_uri_scheme_request_get_http_headers(request);
    if (headers == NULL) {
        return FALSE;
    }
    const char *mode = soup_message_headers_get_one(headers, "Sec-Fetch-Mode");
    if (mode != NULL && g_strcmp0(mode, "cors") == 0) {
        return TRUE;
    }
    const char *origin = soup_message_headers_get_one(headers, "Origin");
    return origin != NULL && origin[0] != '\0';
}

static gboolean path_is_pack_module_css(const char *path) {
    return path != NULL
        && g_str_has_prefix(path, PACK_ROOT)
        && strstr(path, "/assets/") != NULL;
}

static void finish_scheme_bytes(
    WebKitURISchemeRequest *request,
    GInputStream *stream,
    gint64 length,
    const char *mime_type,
    const char *path
) {
    /* Every stylesheet uses simple finish(). Vite __vitePreload inserts
     * a stylesheet link for pack hashed CSS under assets and waits on
     * load. WebKitGTK ignores URISchemeResponse CSS, so CORS-finish
     * hangs that promise and the App chunk never loads. JS/HTML keep
     * CORS. */
    gboolean cors = scheme_request_is_cors(request);
    gboolean pack_css = path_is_pack_module_css(path);
    if (mime_is_stylesheet(mime_type, path)) {
        syslog(
            LOG_INFO,
            "scheme css finish path=%s mode=simple cors=%d pack=%d",
            path != NULL ? path : "",
            cors ? 1 : 0,
            pack_css ? 1 : 0
        );
        webkit_uri_scheme_request_finish(request, stream, length, "text/css");
        return;
    }
    finish_scheme_ok(request, stream, length, mime_type);
}

static void serve_shell_asset(WebKitURISchemeRequest *request, gpointer user_data) {
    (void)user_data;
    const char *uri = webkit_uri_scheme_request_get_uri(request);
    syslog(LOG_INFO, "Shell asset request uri=%s", uri != NULL ? uri : "(null)");
    GError *error = NULL;
    GUri *parsed = g_uri_parse(uri, G_URI_FLAGS_ENCODED_PATH, &error);
    if (parsed == NULL) {
        g_clear_error(&error);
        finish_scheme_error(request, G_IO_ERROR_INVALID_ARGUMENT, "invalid Shell URI");
        return;
    }

    const char *encoded_path = g_uri_get_path(parsed);
    gboolean trusted =
        g_strcmp0(g_uri_get_scheme(parsed), "empiier") == 0
        && g_strcmp0(g_uri_get_host(parsed), "shell") == 0
        && g_uri_get_userinfo(parsed) == NULL
        && g_uri_get_port(parsed) == -1
        && g_uri_get_query(parsed) == NULL
        && g_uri_get_fragment(parsed) == NULL
        && encoded_path != NULL;
    char *decoded_path = trusted ? g_uri_unescape_string(encoded_path, NULL) : NULL;
    if (decoded_path == NULL || !g_path_is_absolute(decoded_path)) {
        trusted = FALSE;
    }

    char *canonical_root = g_canonicalize_filename(SHELL_ROOT, "/");
    char *candidate = trusted
        ? g_canonicalize_filename(decoded_path + 1, canonical_root)
        : NULL;
    size_t root_length = strlen(canonical_root);
    if (candidate == NULL
        || !g_str_has_prefix(candidate, canonical_root)
        || candidate[root_length] != G_DIR_SEPARATOR) {
        trusted = FALSE;
    }

    struct stat metadata;
    if (!trusted
        || g_lstat(candidate, &metadata) != 0
        || !S_ISREG(metadata.st_mode)
        || metadata.st_uid != 0U
        || (metadata.st_mode & 0022U) != 0U
        || metadata.st_size < 0
        || (guint64)metadata.st_size > MAX_SHELL_ASSET_BYTES) {
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded_path);
        g_uri_unref(parsed);
        finish_scheme_error(request, G_IO_ERROR_PERMISSION_DENIED, "Shell asset is not trusted");
        return;
    }

    char *contents = NULL;
    gsize length = 0U;
    if (!g_file_get_contents(candidate, &contents, &length, &error)) {
        g_clear_error(&error);
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded_path);
        g_uri_unref(parsed);
        finish_scheme_error(request, G_IO_ERROR_FAILED, "Shell asset read failed");
        return;
    }

    gboolean uncertain = FALSE;
    char *content_type = g_content_type_guess(
        candidate,
        (const guchar *)contents,
        length,
        &uncertain
    );
    (void)uncertain;
    char *mime_type = content_type != NULL
        ? g_content_type_get_mime_type(content_type)
        : NULL;
    GInputStream *stream = g_memory_input_stream_new_from_data(contents, length, g_free);
    finish_scheme_bytes(
        request,
        stream,
        (gint64)length,
        mime_type != NULL ? mime_type : "application/octet-stream",
        candidate
    );
    syslog(
        LOG_INFO,
        "Shell asset served path=%s type=%s bytes=%zu",
        candidate,
        mime_type != NULL ? mime_type : "application/octet-stream",
        length
    );
    g_object_unref(stream);
    g_free(mime_type);
    g_free(content_type);
    g_free(candidate);
    g_free(canonical_root);
    g_free(decoded_path);
    g_uri_unref(parsed);
}

static gboolean on_delete_event(GtkWidget *widget, GdkEvent *event, gpointer user_data) {
    (void)widget;
    (void)event;
    (void)user_data;
    syslog(LOG_WARNING, "kiosk close ignored (delete-event)");
    return TRUE;
}

static gboolean on_key_press(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)widget;
    WebKitWebView *web_view = WEBKIT_WEB_VIEW(user_data);
    if ((event->state & GDK_MOD1_MASK) != 0U && event->keyval == GDK_KEY_F4) {
        syslog(LOG_WARNING, "kiosk close ignored (Alt+F4)");
        return TRUE;
    }
    if (event->keyval == GDK_KEY_F11) {
        if (!empiier_shell_window_mode_service_allowed()) {
            syslog(LOG_WARNING, "shell window mode toggle ignored (service gate)");
            return TRUE;
        }
        empiier_native_bridge_apply_window_mode(
            &bridge,
            bridge.windowed ? "kiosk" : "windowed"
        );
        return TRUE;
    }
    if (event->keyval == GDK_KEY_F9 || event->keyval == GDK_KEY_Escape) {
        if (event->keyval == GDK_KEY_Escape
            && empiier_native_bridge_seat_browser_key(&bridge, event)) {
            return TRUE;
        }
        if (empiier_native_bridge_escape_surface(&bridge)) {
            return TRUE;
        }
    }
    if (empiier_native_bridge_seat_browser_key(&bridge, event)) {
        return TRUE;
    }
    if (event->keyval == GDK_KEY_F5
        || ((event->state & GDK_CONTROL_MASK) != 0U
            && (event->keyval == GDK_KEY_r || event->keyval == GDK_KEY_R))) {
        webkit_web_view_reload_bypass_cache(web_view);
        return TRUE;
    }
    return FALSE;
}

static gboolean on_window_map(GtkWidget *widget, GdkEvent *event, gpointer user_data) {
    (void)event;
    (void)user_data;
    if (!bridge.windowed) {
        gtk_window_maximize(GTK_WINDOW(widget));
        gtk_window_fullscreen(GTK_WINDOW(widget));
    }
    empiier_native_bridge_publish_window_state(&bridge);
    return FALSE;
}

static gboolean on_window_state(
    GtkWidget *widget,
    GdkEventWindowState *event,
    gpointer user_data
) {
    (void)user_data;
    if (!bridge.windowed
        && (event->new_window_state & GDK_WINDOW_STATE_FULLSCREEN) == 0U) {
        gtk_window_fullscreen(GTK_WINDOW(widget));
    }
    return FALSE;
}

static void on_auth_load_changed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    gpointer user_data
) {
    const char *uri = webkit_web_view_get_uri(web_view);
    if (load_event == WEBKIT_LOAD_STARTED || load_event == WEBKIT_LOAD_FINISHED) {
        syslog(
            LOG_INFO,
            "auth load %s uri=%s",
            load_event == WEBKIT_LOAD_STARTED ? "started" : "finished",
            uri != NULL ? uri : ""
        );
    }
    if (load_event == WEBKIT_LOAD_FINISHED) {
        empiier_oidc_on_auth_page_finished(web_view, user_data);
    }
}

static gboolean on_auth_load_failed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    const char *failing_uri,
    GError *error,
    gpointer user_data
) {
    (void)web_view;
    (void)load_event;
    (void)user_data;
    syslog(
        LOG_ERR,
        "auth load failed uri=%s error=%s",
        failing_uri != NULL ? failing_uri : "(null)",
        error != NULL ? error->message : "unknown"
    );
    return TRUE;
}

static gboolean on_load_failed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    const char *failing_uri,
    GError *error,
    gpointer user_data
) {
    (void)load_event;
    (void)user_data;
    g_warning(
        "Shell load failed uri=%s error=%s",
        failing_uri != NULL ? failing_uri : "(null)",
        error != NULL ? error->message : "unknown"
    );
    syslog(
        LOG_ERR,
        "Shell load failed uri=%s error=%s",
        failing_uri != NULL ? failing_uri : "(null)",
        error != NULL ? error->message : "unknown"
    );
    const char *html =
        "<!doctype html><html lang='ru'><meta charset='utf-8'>"
        "<title>Ошибка загрузки EmpiierOS</title>"
        "<style>body{background:#080d14;color:#eef3f8;font:18px sans-serif;padding:3rem}"
        "code{color:#d0ae72}</style><h1>Ошибка загрузки EmpiierOS</h1>"
        "<p>Не удалось загрузить доверенный пакет Shell.</p>"
        "<p>Проверьте логи командой <code>journalctl -u empiieros-runtime</code> "
        "через режим обслуживания.</p></html>";
    webkit_web_view_load_html(web_view, html, "about:blank");
    return TRUE;
}

static void log_uri_no_query(int priority, const char *tag, const char *uri) {
    if (tag == NULL || uri == NULL || strstr(uri, "eyJ") != NULL || strstr(uri, "Bearer") != NULL) {
        return;
    }
    const char *query = strchr(uri, '?');
    if (query == NULL) {
        syslog(priority, "%s %s", tag, uri);
        return;
    }
    char *cut = g_strndup(uri, (gsize)(query - uri));
    syslog(priority, "%s %s", tag, cut);
    g_free(cut);
}

static gboolean uri_is_fofline_edge(const char *uri) {
    if (uri == NULL) {
        return FALSE;
    }
    return strstr(uri, "socket.io") != NULL
        || g_str_has_prefix(uri, "wss://")
        || g_str_has_prefix(uri, "ws://")
        || strstr(uri, "meet.eliteew.ru") != NULL
        || g_str_has_prefix(uri, "https://app.eliteew.ru/");
}

static void on_resource_failed(
    WebKitWebResource *resource,
    GError *error,
    gpointer user_data
) {
    (void)user_data;
    const char *uri = webkit_web_resource_get_uri(resource);
    if (uri_is_fofline_edge(uri)) {
        log_uri_no_query(LOG_ERR, "fofline edge failed", uri);
        syslog(LOG_ERR, "fofline edge failed detail=%s", error != NULL ? error->message : "unknown");
        return;
    }
    syslog(
        LOG_ERR,
        "Shell resource failed uri=%s error=%s",
        uri != NULL ? uri : "(null)",
        error != NULL ? error->message : "unknown"
    );
}

static void on_resource_load_started(
    WebKitWebView *web_view,
    WebKitWebResource *resource,
    WebKitURIRequest *request,
    gpointer user_data
) {
    (void)web_view;
    (void)request;
    (void)user_data;
    const char *uri = webkit_web_resource_get_uri(resource);
    if (uri_is_fofline_edge(uri)) {
        log_uri_no_query(LOG_INFO, "fofline edge start", uri);
    }
    g_signal_connect(resource, "failed", G_CALLBACK(on_resource_failed), NULL);
}

static gboolean pack_path_has_extension(const char *path) {
    const char *slash = strrchr(path, '/');
    const char *name = slash != NULL ? slash + 1 : path;
    return strchr(name, '.') != NULL;
}

static void serve_fofline_pack(WebKitURISchemeRequest *request, gpointer user_data) {
    (void)user_data;
    const char *uri = webkit_uri_scheme_request_get_uri(request);
    if (!empiier_native_bridge_fofline_uri(uri)) {
        finish_scheme_error(request, G_IO_ERROR_PERMISSION_DENIED, "FoFline pack URI denied");
        return;
    }
    GError *error = NULL;
    GUri *parsed = g_uri_parse(uri, G_URI_FLAGS_ENCODED_PATH, &error);
    if (parsed == NULL) {
        g_clear_error(&error);
        finish_scheme_error(request, G_IO_ERROR_INVALID_ARGUMENT, "invalid FoFline pack URI");
        return;
    }
    const char *encoded_path = g_uri_get_path(parsed);
    char *decoded_path = encoded_path != NULL ? g_uri_unescape_string(encoded_path, NULL) : NULL;
    if (decoded_path == NULL || strstr(decoded_path, "..") != NULL || strchr(decoded_path, '\\') != NULL) {
        g_free(decoded_path);
        g_uri_unref(parsed);
        finish_scheme_error(request, G_IO_ERROR_PERMISSION_DENIED, "FoFline pack path denied");
        return;
    }
    if (decoded_path[0] == '\0' || g_strcmp0(decoded_path, "/") == 0) {
        g_free(decoded_path);
        decoded_path = g_strdup("/index.html");
    }
    if (empiier_fofline_api_path(decoded_path)) {
        empiier_fofline_api_proxy(request, decoded_path, g_uri_get_query(parsed));
        g_free(decoded_path);
        g_uri_unref(parsed);
        return;
    }
    char *canonical_root = g_canonicalize_filename(PACK_ROOT, "/");
    const char *relative = decoded_path[0] == '/' ? decoded_path + 1 : decoded_path;
    char *candidate = g_canonicalize_filename(relative, canonical_root);
    size_t root_length = strlen(canonical_root);
    gboolean trusted = candidate != NULL
        && g_str_has_prefix(candidate, canonical_root)
        && (candidate[root_length] == '\0' || candidate[root_length] == G_DIR_SEPARATOR);
    struct stat metadata;
    if (trusted
        && (g_lstat(candidate, &metadata) != 0 || !S_ISREG(metadata.st_mode))
        && !pack_path_has_extension(decoded_path)) {
        g_free(candidate);
        candidate = g_canonicalize_filename("index.html", canonical_root);
        trusted = candidate != NULL
            && g_str_has_prefix(candidate, canonical_root)
            && g_lstat(candidate, &metadata) == 0
            && S_ISREG(metadata.st_mode);
    }
    if (!trusted
        || g_lstat(candidate, &metadata) != 0
        || !S_ISREG(metadata.st_mode)
        || metadata.st_uid != 0U
        || (metadata.st_mode & 0022U) != 0U
        || metadata.st_size < 0
        || (guint64)metadata.st_size > MAX_PACK_ASSET_BYTES) {
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded_path);
        g_uri_unref(parsed);
        finish_scheme_error(request, G_IO_ERROR_PERMISSION_DENIED, "FoFline pack asset is not trusted");
        return;
    }
    char *contents = NULL;
    gsize length = 0U;
    if (!g_file_get_contents(candidate, &contents, &length, &error)) {
        g_clear_error(&error);
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded_path);
        g_uri_unref(parsed);
        finish_scheme_error(request, G_IO_ERROR_FAILED, "FoFline pack read failed");
        return;
    }
    gboolean uncertain = FALSE;
    char *content_type = g_content_type_guess(
        candidate,
        (const guchar *)contents,
        length,
        &uncertain
    );
    (void)uncertain;
    char *mime_type = content_type != NULL ? g_content_type_get_mime_type(content_type) : NULL;
    GInputStream *stream = g_memory_input_stream_new_from_data(contents, length, g_free);
    finish_scheme_bytes(
        request,
        stream,
        (gint64)length,
        mime_type != NULL ? mime_type : "application/octet-stream",
        candidate
    );
    syslog(
        LOG_INFO,
        "FoFline pack served path=%s type=%s bytes=%zu",
        candidate,
        mime_type != NULL ? mime_type : "application/octet-stream",
        length
    );
    g_object_unref(stream);
    g_free(mime_type);
    g_free(content_type);
    g_free(candidate);
    g_free(canonical_root);
    g_free(decoded_path);
    g_uri_unref(parsed);
}

static void enable_webkit_sandbox(WebKitWebContext *context) {
    /* Product FoFline fetch() is HTTPS to app.eliteew.ru (VIP via hosts).
     * Lab NSS is resolve [!UNAVAIL=return] files. Sandbox hides host CA
     * and resolved dbus: getaddrinfo never reaches TCP 443 and the SPA
     * stays on the preloader. Same class as the auth white page.
     * decide_policy + custom schemes stay the boundary. */
    webkit_web_context_set_sandbox_enabled(context, FALSE);
    syslog(LOG_INFO, "WebKit sandbox off for host CA/NSS product HTTPS");
}

static void set_monitor_size(GtkWindow *window) {
    GdkDisplay *display = gdk_display_get_default();
    if (display == NULL) {
        return;
    }
    GdkMonitor *monitor = gdk_display_get_primary_monitor(display);
    if (monitor == NULL && gdk_display_get_n_monitors(display) > 0) {
        monitor = gdk_display_get_monitor(display, 0);
    }
    if (monitor != NULL) {
        GdkRectangle geometry;
        gdk_monitor_get_geometry(monitor, &geometry);
        gtk_window_set_default_size(window, geometry.width, geometry.height);
    }
}

int main(int argc, char **argv) {
    gboolean splash_up = FALSE;

    openlog("empiieros-shell-gui", LOG_PID, LOG_USER);
    if (!gtk_init_check(&argc, &argv)) {
        fputs("empiieros-shell-gui: GTK display initialization failed\n", stderr);
        syslog(LOG_ERR, "GTK display initialization failed");
        closelog();
        return 2;
    }
    (void)g_unix_signal_add(SIGINT, stop_main_loop, NULL);
    (void)g_unix_signal_add(SIGTERM, stop_main_loop, NULL);

    GtkWidget *window_widget = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    GtkWindow *window = GTK_WINDOW(window_widget);
    gtk_window_set_title(window, WINDOW_TITLE);
    gtk_window_set_role(window, WINDOW_ROLE);
    gtk_window_set_decorated(window, FALSE);
    set_monitor_size(window);

    WebKitSettings *settings = webkit_settings_new();
    webkit_settings_set_enable_javascript(settings, TRUE);
    webkit_settings_set_enable_developer_extras(settings, FALSE);
    webkit_settings_set_enable_webgl(settings, FALSE);
    webkit_settings_set_enable_page_cache(settings, FALSE);
    webkit_settings_set_enable_smooth_scrolling(settings, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(settings, FALSE);
    webkit_settings_set_allow_universal_access_from_file_urls(settings, FALSE);
    /* Same WebView later hosts fofline-spa. Iframe getUserMedia is reported
     * here; permission-request still deny-closes empiier://. */
    webkit_settings_set_enable_media(settings, TRUE);
    webkit_settings_set_enable_media_stream(settings, TRUE);
    webkit_settings_set_enable_mediasource(settings, TRUE);
    webkit_settings_set_enable_webrtc(settings, TRUE);

    WebKitUserContentManager *manager = webkit_user_content_manager_new();
    WebKitUserScript *fofline_boot_script = webkit_user_script_new(
        "(function(){try{"
        "if(String(location.protocol||'')!=='fofline-spa:')return;"
        "window.addEventListener('error',function(e){"
        "var t=e&&e.target;"
        "var src=String((t&&(t.src||t.href))||(e&&e.filename)||'');"
        "var m=String((e&&e.message)||'');"
        "if(!m)m=src?('load:'+src):'err';"
        "if(m.indexOf('eyJ')!==-1||m.indexOf('Bearer')!==-1)return;"
        "if(src.indexOf('eyJ')!==-1||src.indexOf('Bearer')!==-1)src='';"
        "window.__empiierBootErr=m.slice(0,96);"
        "if(src)window.__empiierBootSrc=src.slice(0,96);"
        "},true);"
        "}catch(x){}})();",
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
    webkit_user_content_manager_add_script(manager, fofline_boot_script);
    webkit_user_script_unref(fofline_boot_script);
    /* MusicPlayer sets <audio src> to fofline-spa content URLs. WebKit
     * cancels that media load even after a 200 body. Fetch a blob first. */
    WebKitUserScript *fofline_audio_script = webkit_user_script_new(
        "(function(){try{"
        "if(String(location.protocol||'')!=='fofline-spa:')return;"
        "if(window.__empiierosAudioBlob)return;"
        "window.__empiierosAudioBlob=true;"
        "function isContentDl(u){"
        "u=String(u||'');"
        "return u.indexOf('/api/v1/content/')!==-1&&u.indexOf('download')!==-1&&u.indexOf('blob:')!==0;"
        "}"
        "var rawLoad=HTMLMediaElement.prototype.load;"
        "HTMLMediaElement.prototype.load=function(){"
        "if(this.__empiierosBlobbing){this.__empiierosNeedLoad=true;return;}"
        "return rawLoad.apply(this,arguments);"
        "};"
        "function toBlob(el,url){"
        "if(!el||el.__empiierosBlobbing)return;"
        "el.__empiierosBlobbing=true;"
        "el.__empiierosNeedLoad=true;"
        "fetch(url,{credentials:'same-origin'}).then(function(r){"
        "if(!r.ok)throw new Error('audio_fetch');"
        "return r.blob();"
        "}).then(function(b){"
        "var t=b&&b.type&&b.type.indexOf('audio')===0?b:new Blob([b],{type:'audio/mpeg'});"
        "el.__empiierosBlobbing=false;"
        "el.src=URL.createObjectURL(t);"
        "if(el.__empiierosNeedLoad){el.__empiierosNeedLoad=false;try{rawLoad.call(el);}catch(e){}}"
        "}).catch(function(){el.__empiierosBlobbing=false;el.__empiierosNeedLoad=false;});"
        "}"
        "function patch(proto){"
        "if(!proto)return;"
        "var d=Object.getOwnPropertyDescriptor(proto,'src');"
        "if(!d||!d.set)return;"
        "var set=d.set;"
        "Object.defineProperty(proto,'src',{configurable:true,enumerable:d.enumerable,get:d.get,set:function(v){"
        "if(isContentDl(v)){toBlob(this,v);return;}set.call(this,v);"
        "}});"
        "}"
        "patch(HTMLMediaElement.prototype);"
        "if(window.HTMLSourceElement)patch(HTMLSourceElement.prototype);"
        "var sa=Element.prototype.setAttribute;"
        "HTMLMediaElement.prototype.setAttribute=function(n,v){"
        "if(String(n).toLowerCase()==='src'&&isContentDl(v)){toBlob(this,v);return;}"
        "return sa.apply(this,arguments);"
        "};"
        "}catch(x){}})();",
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
    webkit_user_content_manager_add_script(manager, fofline_audio_script);
    webkit_user_script_unref(fofline_audio_script);
    const char *maintenance = getenv("EMPIIEROS_SHELL_MAINTENANCE");
    if (maintenance != NULL && strcmp(maintenance, "1") == 0) {
    WebKitUserScript *diagnostics_script = webkit_user_script_new(
        "window.__empiierErrors=[];"
        "window.addEventListener('error',event=>{"
        "const target=event.target;"
        "const resource=target&&(target.src||target.href);"
        "window.__empiierErrors.push(String(resource||event.message||event.error||'error'));"
        "},true);"
        "window.addEventListener('unhandledrejection',event=>"
        "window.__empiierErrors.push(String(event.reason||'unhandled rejection')));"
        "window.addEventListener('securitypolicyviolation',event=>"
        "window.__empiierErrors.push(`CSP ${event.violatedDirective} ${event.blockedURI}`));",
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
    webkit_user_content_manager_add_script(manager, diagnostics_script);
    webkit_user_script_unref(diagnostics_script);
    syslog(LOG_INFO, "WebKit DOM diagnostics enabled");
    } else {
        syslog(LOG_INFO, "WebKit DOM diagnostics disabled");
    }
    WebKitWebContext *context = webkit_web_context_new_ephemeral();
    enable_webkit_sandbox(context);
    webkit_web_context_register_uri_scheme(
        context,
        "empiier",
        serve_shell_asset,
        NULL,
        NULL
    );
    WebKitSecurityManager *security_manager =
        webkit_web_context_get_security_manager(context);
    webkit_security_manager_register_uri_scheme_as_local(security_manager, "empiier");
    webkit_security_manager_register_uri_scheme_as_secure(security_manager, "empiier");
    webkit_security_manager_register_uri_scheme_as_cors_enabled(
        security_manager,
        "empiier"
    );
    webkit_web_context_register_uri_scheme(
        context,
        "fofline-spa",
        serve_fofline_pack,
        NULL,
        NULL
    );
    /* Not as_local: WebKit treats local schemes like file and ESM CSS
     * imports never settle even with ACAO. Keep secure plus CORS. */
    webkit_security_manager_register_uri_scheme_as_secure(security_manager, "fofline-spa");
    webkit_security_manager_register_uri_scheme_as_cors_enabled(
        security_manager,
        "fofline-spa"
    );
    GtkWidget *view_widget = g_object_new(
        WEBKIT_TYPE_WEB_VIEW,
        "web-context",
        context,
        "settings",
        settings,
        "user-content-manager",
        manager,
        NULL
    );
    WebKitWebView *web_view = WEBKIT_WEB_VIEW(view_widget);

    empiier_native_bridge_init(
        &bridge,
        manager,
        web_view,
        window,
        TRUSTED_URI_PREFIX,
        RUNTIME_SOCKET
    );

    GtkWidget *root = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *product_bar = empiier_product_chrome_new(&bridge);
    GtkWidget *overlay = gtk_overlay_new();
    gtk_container_add(GTK_CONTAINER(overlay), view_widget);
    bridge.overlay = overlay;

    WebKitSettings *auth_settings = webkit_settings_new();
    webkit_settings_set_enable_javascript(auth_settings, TRUE);
    webkit_settings_set_enable_developer_extras(auth_settings, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(auth_settings, FALSE);
    webkit_settings_set_allow_universal_access_from_file_urls(auth_settings, FALSE);
    WebKitUserContentManager *auth_manager = webkit_user_content_manager_new();
    WebKitUserScript *auth_guard = empiier_oidc_manual_login_script();
    webkit_user_content_manager_add_script(auth_manager, auth_guard);
    webkit_user_script_unref(auth_guard);
    WebKitWebContext *auth_context = webkit_web_context_new_ephemeral();
    /* Auth must use host CA store + /etc/hosts → VIP. Sandbox here yields a
     * silent white page: no certs, no hosts, HTTPS never paints. Host
     * allowlist in auth_decide_policy stays the boundary. */
    webkit_web_context_set_sandbox_enabled(auth_context, FALSE);
    webkit_cookie_manager_set_accept_policy(
        webkit_website_data_manager_get_cookie_manager(
            webkit_web_context_get_website_data_manager(auth_context)
        ),
        WEBKIT_COOKIE_POLICY_ACCEPT_ALWAYS
    );
    GtkWidget *auth_view_widget = g_object_new(
        WEBKIT_TYPE_WEB_VIEW,
        "web-context",
        auth_context,
        "settings",
        auth_settings,
        "user-content-manager",
        auth_manager,
        NULL
    );
    WebKitWebView *auth_view = WEBKIT_WEB_VIEW(auth_view_widget);
    GtkWidget *auth_box = empiier_auth_card_build(&bridge, auth_view);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), auth_box);
    gtk_widget_set_halign(auth_box, GTK_ALIGN_FILL);
    gtk_widget_set_valign(auth_box, GTK_ALIGN_FILL);
    gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(overlay), auth_box, TRUE);

    empiier_native_bridge_attach_chrome(&bridge, product_bar, auth_box, auth_view);
    g_signal_connect(
        auth_view_widget,
        "decide-policy",
        G_CALLBACK(empiier_native_bridge_auth_decide_policy),
        &bridge
    );

    /* Seat Browser overlay: ephemeral WebKit + Runtime decide-policy.
     * Sandbox off so host CA/hosts work for clearnet HTTPS; policy gate is
     * seat.browser.evaluate on every navigation (default-deny empty allowlist).
     * WebRTC must be on or Jitsi paints "WebRTC is not available in your browser". */
    WebKitSettings *browser_settings = webkit_settings_new();
    webkit_settings_set_enable_javascript(browser_settings, TRUE);
    webkit_settings_set_enable_developer_extras(browser_settings, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(browser_settings, FALSE);
    webkit_settings_set_allow_universal_access_from_file_urls(browser_settings, FALSE);
    webkit_settings_set_enable_media(browser_settings, TRUE);
    webkit_settings_set_enable_media_stream(browser_settings, TRUE);
    webkit_settings_set_enable_mediasource(browser_settings, TRUE);
    webkit_settings_set_enable_webrtc(browser_settings, TRUE);
    WebKitWebContext *browser_context = webkit_web_context_new_ephemeral();
    webkit_web_context_set_sandbox_enabled(browser_context, FALSE);
    webkit_cookie_manager_set_accept_policy(
        webkit_website_data_manager_get_cookie_manager(
            webkit_web_context_get_website_data_manager(browser_context)
        ),
        WEBKIT_COOKIE_POLICY_ACCEPT_NO_THIRD_PARTY
    );
    GtkWidget *browser_view_widget = g_object_new(
        WEBKIT_TYPE_WEB_VIEW,
        "web-context",
        browser_context,
        "settings",
        browser_settings,
        NULL
    );
    WebKitWebView *browser_view = WEBKIT_WEB_VIEW(browser_view_widget);
    GtkWidget *browser_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *browser_chrome = empiier_seat_browser_chrome_build(&bridge);
    gtk_widget_set_no_show_all(browser_box, TRUE);
    gtk_box_pack_start(GTK_BOX(browser_box), browser_chrome, FALSE, FALSE, 0);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), browser_box);
    gtk_widget_set_halign(browser_box, GTK_ALIGN_FILL);
    gtk_widget_set_valign(browser_box, GTK_ALIGN_FILL);
    gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(overlay), browser_box, TRUE);
    empiier_native_bridge_attach_seat_browser(&bridge, browser_box, browser_view);

    /* Meet stage: second WebKit, not a product iframe. Sandbox off so
     * /etc/hosts → VIP and host CA work. Camera/mic only on meet.eliteew.ru. */
    WebKitSettings *meet_settings = webkit_settings_new();
    webkit_settings_set_enable_javascript(meet_settings, TRUE);
    webkit_settings_set_enable_developer_extras(meet_settings, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(meet_settings, FALSE);
    webkit_settings_set_allow_universal_access_from_file_urls(meet_settings, FALSE);
    webkit_settings_set_enable_media(meet_settings, TRUE);
    webkit_settings_set_enable_media_stream(meet_settings, TRUE);
    webkit_settings_set_enable_mediasource(meet_settings, TRUE);
    webkit_settings_set_enable_webrtc(meet_settings, TRUE);
    WebKitWebContext *meet_context = webkit_web_context_new_ephemeral();
    webkit_web_context_set_sandbox_enabled(meet_context, FALSE);
    webkit_cookie_manager_set_accept_policy(
        webkit_website_data_manager_get_cookie_manager(
            webkit_web_context_get_website_data_manager(meet_context)
        ),
        WEBKIT_COOKIE_POLICY_ACCEPT_NO_THIRD_PARTY
    );
    WebKitUserContentManager *meet_ucm = webkit_user_content_manager_new();
    empiier_meet_register_diag(meet_ucm);
    GtkWidget *meet_view_widget = g_object_new(
        WEBKIT_TYPE_WEB_VIEW,
        "web-context",
        meet_context,
        "user-content-manager",
        meet_ucm,
        "settings",
        meet_settings,
        NULL
    );
    WebKitWebView *meet_view = WEBKIT_WEB_VIEW(meet_view_widget);
    GtkWidget *meet_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_set_no_show_all(meet_box, TRUE);
    gtk_box_pack_start(GTK_BOX(meet_box), meet_view_widget, TRUE, TRUE, 0);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), meet_box);
    gtk_widget_set_halign(meet_box, GTK_ALIGN_START);
    gtk_widget_set_valign(meet_box, GTK_ALIGN_START);
    gtk_widget_set_hexpand(meet_box, FALSE);
    gtk_widget_set_vexpand(meet_box, FALSE);
    gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(overlay), meet_box, TRUE);
    empiier_native_bridge_attach_meet(&bridge, meet_box, meet_view);
    empiier_fofline_api_proxy_bind(&bridge);
    g_signal_connect(
        meet_view_widget,
        "decide-policy",
        G_CALLBACK(empiier_native_bridge_meet_decide_policy),
        &bridge
    );
    g_signal_connect(
        meet_view_widget,
        "permission-request",
        G_CALLBACK(empiier_native_bridge_meet_permission),
        &bridge
    );
    g_signal_connect(meet_view_widget, "context-menu", G_CALLBACK(suppress_context_menu), NULL);
    g_signal_connect(meet_view_widget, "resource-load-started", G_CALLBACK(on_resource_load_started), NULL);
    g_signal_connect(
        meet_view_widget,
        "load-failed",
        G_CALLBACK(empiier_native_bridge_meet_load_failed),
        NULL
    );
    g_signal_connect(auth_view_widget, "context-menu", G_CALLBACK(suppress_context_menu), NULL);
    g_signal_connect(auth_view_widget, "permission-request", G_CALLBACK(deny_permission), NULL);
    g_signal_connect(auth_view_widget, "load-changed", G_CALLBACK(on_auth_load_changed), &bridge);
    g_signal_connect(auth_view_widget, "load-failed", G_CALLBACK(on_auth_load_failed), NULL);
    g_signal_connect(
        auth_view_widget,
        "submit-form",
        G_CALLBACK(empiier_oidc_on_auth_submit_form),
        &bridge
    );

    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), product_bar);
    gtk_widget_set_halign(product_bar, GTK_ALIGN_END);
    gtk_widget_set_valign(product_bar, GTK_ALIGN_CENTER);
    splash_up = empiier_boot_splash_attach(GTK_OVERLAY(overlay), &bridge);
    gtk_box_pack_start(GTK_BOX(root), overlay, TRUE, TRUE, 0);

    /* Do not connect scroll-event or add SCROLL masks: any GTK handler
     * (even return FALSE) has swallowed wheel delivery to WebKit on seat.
     * Overflow panels scroll via Secure Shell DOM wheel walker. */
    gtk_widget_add_events(window_widget, GDK_KEY_PRESS_MASK);
    gtk_widget_set_can_focus(view_widget, TRUE);

    g_signal_connect(window_widget, "delete-event", G_CALLBACK(on_delete_event), NULL);
    g_signal_connect(window_widget, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    g_signal_connect(window_widget, "map-event", G_CALLBACK(on_window_map), NULL);
    g_signal_connect(window_widget, "window-state-event", G_CALLBACK(on_window_state), NULL);
    g_signal_connect(window_widget, "key-press-event", G_CALLBACK(on_key_press), web_view);
    g_signal_connect(view_widget, "context-menu", G_CALLBACK(suppress_context_menu), NULL);
    g_signal_connect(
        view_widget,
        "permission-request",
        G_CALLBACK(empiier_native_bridge_product_meet_permission),
        &bridge
    );
    g_signal_connect(view_widget, "load-changed", G_CALLBACK(on_load_changed), NULL);
    g_signal_connect(view_widget, "load-failed", G_CALLBACK(on_load_failed), NULL);
    g_signal_connect(
        view_widget,
        "resource-load-started",
        G_CALLBACK(on_resource_load_started),
        NULL
    );
    g_signal_connect(
        view_widget,
        "decide-policy",
        G_CALLBACK(empiier_native_bridge_decide_policy),
        &bridge
    );

    gtk_container_add(GTK_CONTAINER(window_widget), root);
    webkit_web_view_load_uri(web_view, SHELL_URI);
    gtk_widget_show_all(window_widget);
    gtk_window_maximize(window);
    gtk_window_fullscreen(window);
    empiier_product_chrome_show(&bridge);
    if (splash_up) {
        gtk_widget_hide(product_bar);
    }
    empiier_native_bridge_publish_window_state(&bridge);
    gtk_widget_grab_focus(view_widget);
    gtk_main();

    webkit_user_content_manager_unregister_script_message_handler(manager, "empiieros");
    g_object_unref(auth_context);
    g_object_unref(auth_manager);
    g_object_unref(auth_settings);
    g_object_unref(browser_context);
    g_object_unref(browser_settings);
    g_object_unref(context);
    g_object_unref(manager);
    g_object_unref(settings);
    closelog();
    return 0;
}
