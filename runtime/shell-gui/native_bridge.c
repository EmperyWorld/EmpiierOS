#include "native_bridge.h"

#include <cairo.h>
#include <errno.h>
#include <gio/gio.h>
#include <jsc/jsc.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/un.h>
#include <syslog.h>
#include <unistd.h>

#define MAX_REQUEST_BYTES (64U * 1024U)
#define MAX_RESPONSE_BYTES (256U * 1024U)

void empiier_widget_show_deferred(GtkWidget *widget) {
    if (widget == NULL) {
        return;
    }
    /* GTK3 gtk_widget_show_all() returns immediately when no_show_all is set. */
    gtk_widget_set_no_show_all(widget, FALSE);
    gtk_widget_show_all(widget);
}

void empiier_widget_hide_deferred(GtkWidget *widget) {
    if (widget == NULL) {
        return;
    }
    gtk_widget_hide(widget);
    gtk_widget_set_no_show_all(widget, TRUE);
}

static void overlay_input_pass(GtkWidget *view, gboolean pass) {
    GdkWindow *win;
    cairo_region_t *empty;

    if (view == NULL || !gtk_widget_get_realized(view)) {
        return;
    }
    win = gtk_widget_get_window(view);
    if (win == NULL) {
        return;
    }
    if (pass) {
        empty = cairo_region_create();
        gdk_window_input_shape_combine_region(win, empty, 0, 0);
        cairo_region_destroy(empty);
        return;
    }
    gdk_window_input_shape_combine_region(win, NULL, 0, 0);
}

void empiier_overlay_park(EmpiierNativeBridge *bridge, GtkWidget *box, GtkWidget *view) {
    if (box == NULL) {
        return;
    }
    overlay_input_pass(view, TRUE);
    if (bridge != NULL && bridge->overlay != NULL) {
        gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(bridge->overlay), box, TRUE);
    }
    gtk_widget_set_sensitive(box, FALSE);
    empiier_widget_hide_deferred(box);
}

void empiier_overlay_raise(EmpiierNativeBridge *bridge, GtkWidget *box, GtkWidget *view) {
    if (box == NULL) {
        return;
    }
    gtk_widget_set_sensitive(box, TRUE);
    if (bridge != NULL && bridge->overlay != NULL) {
        gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(bridge->overlay), box, FALSE);
    }
    empiier_widget_show_deferred(box);
    overlay_input_pass(view, FALSE);
}

typedef struct {
    EmpiierNativeBridge *bridge;
    char *request_json;
    char *request_id;
} BridgeJob;

char *empiier_native_json_escape(const char *input) {
    GString *output = g_string_new("\"");
    const unsigned char *cursor = (const unsigned char *)(input != NULL ? input : "");
    while (*cursor != '\0') {
        switch (*cursor) {
            case '"':
                g_string_append(output, "\\\"");
                break;
            case '\\':
                g_string_append(output, "\\\\");
                break;
            case '\b':
                g_string_append(output, "\\b");
                break;
            case '\f':
                g_string_append(output, "\\f");
                break;
            case '\n':
                g_string_append(output, "\\n");
                break;
            case '\r':
                g_string_append(output, "\\r");
                break;
            case '\t':
                g_string_append(output, "\\t");
                break;
            default:
                if (*cursor < 0x20U) {
                    g_string_append_printf(output, "\\u%04x", *cursor);
                } else {
                    g_string_append_c(output, (char)*cursor);
                }
                break;
        }
        cursor++;
    }
    g_string_append_c(output, '"');
    return g_string_free(output, FALSE);
}

static char *failure_response(const char *request_id, const char *code, const char *message) {
    char *id_json = empiier_native_json_escape(request_id);
    char *code_json = empiier_native_json_escape(code);
    char *message_json = empiier_native_json_escape(message);
    char *response = g_strdup_printf(
        "{\"protocol_version\":1,\"id\":%s,\"ok\":false,\"status\":\"bridge_error\","
        "\"data\":null,\"error\":{\"code\":%s,\"message\":%s}}",
        id_json,
        code_json,
        message_json
    );
    g_free(id_json);
    g_free(code_json);
    g_free(message_json);
    return response;
}

static gboolean send_all(int fd, const char *data, size_t length) {
    size_t sent = 0U;
    while (sent < length) {
        ssize_t count = send(fd, data + sent, length - sent, MSG_NOSIGNAL);
        if (count < 0 && errno == EINTR) {
            continue;
        }
        if (count <= 0) {
            return FALSE;
        }
        sent += (size_t)count;
    }
    return TRUE;
}

static char *runtime_exchange(const char *socket_path, const char *request, GError **error) {
    if (strlen(request) > MAX_REQUEST_BYTES) {
        g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_INVALID_ARGUMENT, "request too large");
        return NULL;
    }
    if (strlen(socket_path) >= sizeof(((struct sockaddr_un *)0)->sun_path)) {
        g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_INVALID_ARGUMENT, "socket path too long");
        return NULL;
    }

    int fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) {
        g_set_error(error, G_IO_ERROR, g_io_error_from_errno(errno), "socket: %s", g_strerror(errno));
        return NULL;
    }

    struct timeval timeout = {.tv_sec = 180, .tv_usec = 0};
    (void)setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
    (void)setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));

    struct sockaddr_un address;
    memset(&address, 0, sizeof(address));
    address.sun_family = AF_UNIX;
    g_strlcpy(address.sun_path, socket_path, sizeof(address.sun_path));
    if (connect(fd, (struct sockaddr *)&address, sizeof(address)) != 0) {
        int saved_errno = errno;
        close(fd);
        g_set_error(
            error,
            G_IO_ERROR,
            g_io_error_from_errno(saved_errno),
            "connect: %s",
            g_strerror(saved_errno)
        );
        return NULL;
    }

    if (!send_all(fd, request, strlen(request)) || !send_all(fd, "\n", 1U)) {
        int saved_errno = errno;
        close(fd);
        g_set_error(
            error,
            G_IO_ERROR,
            g_io_error_from_errno(saved_errno),
            "send: %s",
            g_strerror(saved_errno)
        );
        return NULL;
    }

    GString *response = g_string_sized_new(4096U);
    char buffer[4096];
    gboolean complete = FALSE;
    while (response->len <= MAX_RESPONSE_BYTES) {
        ssize_t count = recv(fd, buffer, sizeof(buffer), 0);
        if (count < 0 && errno == EINTR) {
            continue;
        }
        if (count <= 0) {
            break;
        }
        char *newline = memchr(buffer, '\n', (size_t)count);
        size_t retain = newline != NULL ? (size_t)(newline - buffer) : (size_t)count;
        g_string_append_len(response, buffer, (gssize)retain);
        if (newline != NULL) {
            complete = TRUE;
            break;
        }
    }
    close(fd);

    if (!complete || response->len == 0U || response->len > MAX_RESPONSE_BYTES) {
        g_string_free(response, TRUE);
        g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_INVALID_DATA, "invalid runtime response framing");
        return NULL;
    }
    if (response->str[0] != '{') {
        g_string_free(response, TRUE);
        g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_INVALID_DATA, "runtime response is not JSON");
        return NULL;
    }
    return g_string_free(response, FALSE);
}

char *empiier_native_runtime_exchange(
    EmpiierNativeBridge *bridge,
    const char *request,
    GError **error
) {
    if (bridge == NULL || request == NULL) {
        g_set_error_literal(error, G_IO_ERROR, G_IO_ERROR_INVALID_ARGUMENT, "runtime exchange missing");
        return NULL;
    }
    return runtime_exchange(bridge->socket_path, request, error);
}

void empiier_native_dispatch_custom_event(
    EmpiierNativeBridge *bridge,
    const char *name,
    const char *detail_json
) {
    if (bridge == NULL || bridge->web_view == NULL || name == NULL || detail_json == NULL) {
        return;
    }
    if (strpbrk(name, "\"';\\") != NULL) {
        return;
    }
    char *script = g_strdup_printf(
        "window.dispatchEvent(new CustomEvent('%s',{detail:%s}));",
        name,
        detail_json
    );
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-native-event",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
}

static void bridge_job_free(BridgeJob *job) {
    if (job == NULL) {
        return;
    }
    g_free(job->request_json);
    g_free(job->request_id);
    g_free(job);
}

static void exchange_thread(GTask *task, gpointer source_object, gpointer task_data, GCancellable *cancellable) {
    (void)source_object;
    (void)cancellable;
    BridgeJob *job = task_data;
    GError *error = NULL;
    char *response = runtime_exchange(job->bridge->socket_path, job->request_json, &error);
    if (response == NULL) {
        g_task_return_error(task, error);
        return;
    }
    g_task_return_pointer(task, response, g_free);
}

static void dispatch_response(EmpiierNativeBridge *bridge, const char *response_json) {
    if (bridge->web_view == NULL || response_json == NULL) {
        return;
    }
    char *script = g_strdup_printf(
        "window.dispatchEvent(new CustomEvent('empiieros-response',{detail:%s}));",
        response_json
    );
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-native-bridge",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
}

static void dispatch_correlated_response(
    EmpiierNativeBridge *bridge,
    const char *response_json,
    const char *request_id
) {
    if (bridge->web_view == NULL || response_json == NULL) {
        return;
    }
    char *id_json = empiier_native_json_escape(request_id);
    char *script = g_strdup_printf(
        "(()=>{const detail=%s;if(typeof detail.id!=='string'||detail.id.length===0)"
        "{detail.id=%s;}window.dispatchEvent(new CustomEvent('empiieros-response',{detail}));})()",
        response_json,
        id_json
    );
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-native-bridge",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
    g_free(id_json);
}

static void exchange_finished(GObject *source_object, GAsyncResult *result, gpointer user_data) {
    (void)source_object;
    EmpiierNativeBridge *bridge = user_data;
    GTask *task = G_TASK(result);
    BridgeJob *job = g_task_get_task_data(task);
    GError *error = NULL;
    char *response = g_task_propagate_pointer(task, &error);
    if (response != NULL) {
        syslog(
            LOG_INFO,
            "runtime bridge response id=%s",
            job != NULL ? job->request_id : ""
        );
        dispatch_correlated_response(
            bridge,
            response,
            job != NULL ? job->request_id : ""
        );
        if (strstr(response, "\"events\"") != NULL) {
            char *event_script = g_strdup_printf(
                "window.dispatchEvent(new CustomEvent('empiieros-event',{detail:%s}));",
                response
            );
            webkit_web_view_evaluate_javascript(
                bridge->web_view,
                event_script,
                -1,
                NULL,
                "empiieros-native-event",
                NULL,
                NULL,
                NULL
            );
            g_free(event_script);
        }
        g_free(response);
        return;
    }
    syslog(
        LOG_ERR,
        "runtime bridge failure id=%s error=%s",
        job != NULL ? job->request_id : "",
        error != NULL ? error->message : "runtime exchange failed"
    );
    char *failure = failure_response(
        job != NULL ? job->request_id : "",
        "runtime_unreachable",
        error != NULL ? error->message : "runtime exchange failed"
    );
    dispatch_correlated_response(
        bridge,
        failure,
        job != NULL ? job->request_id : ""
    );
    g_free(failure);
    g_clear_error(&error);
}

static char *property_string(JSCValue *object, const char *name) {
    if (object == NULL || !jsc_value_is_object(object)) {
        return NULL;
    }
    JSCValue *property = jsc_value_object_get_property(object, name);
    if (property == NULL || !jsc_value_is_string(property)) {
        g_clear_object(&property);
        return NULL;
    }
    char *value = jsc_value_to_string(property);
    g_object_unref(property);
    return value;
}

static guint32 property_u32(JSCValue *object, const char *name) {
    JSCValue *property;
    guint32 out = 0;

    if (object == NULL || name == NULL || !jsc_value_is_object(object)) {
        return 0;
    }
    property = jsc_value_object_get_property(object, name);
    if (property == NULL) {
        return 0;
    }
    if (jsc_value_is_number(property)) {
        double raw = jsc_value_to_double(property);
        if (raw > 0.0 && raw < 4000000000.0) {
            out = (guint32)raw;
        }
    } else if (jsc_value_is_string(property)) {
        char *text = jsc_value_to_string(property);
        if (text != NULL) {
            out = (guint32)g_ascii_strtoull(text, NULL, 10);
            g_free(text);
        }
    }
    g_object_unref(property);
    return out;
}

static gboolean trusted_shell_uri(EmpiierNativeBridge *bridge, const char *uri) {
    if (uri == NULL || bridge == NULL || bridge->trusted_uri_prefix == NULL) {
        return FALSE;
    }
    GError *error = NULL;
    GUri *root_uri = g_uri_parse(
        bridge->trusted_uri_prefix,
        G_URI_FLAGS_ENCODED_PATH,
        &error
    );
    if (root_uri == NULL) {
        g_clear_error(&error);
        return FALSE;
    }
    GUri *candidate_uri = g_uri_parse(uri, G_URI_FLAGS_ENCODED_PATH, &error);
    if (candidate_uri == NULL) {
        g_uri_unref(root_uri);
        g_clear_error(&error);
        return FALSE;
    }

    gboolean trusted =
        g_strcmp0(g_uri_get_scheme(candidate_uri), g_uri_get_scheme(root_uri)) == 0
        && g_strcmp0(g_uri_get_host(candidate_uri), g_uri_get_host(root_uri)) == 0
        && g_uri_get_userinfo(candidate_uri) == NULL
        && g_uri_get_port(candidate_uri) == -1
        && g_uri_get_query(candidate_uri) == NULL
        && g_uri_get_fragment(candidate_uri) == NULL;
    const char *root_encoded_path = g_uri_get_path(root_uri);
    const char *candidate_encoded_path = g_uri_get_path(candidate_uri);
    char *root_path = root_encoded_path != NULL
        ? g_uri_unescape_string(root_encoded_path, NULL)
        : NULL;
    char *candidate_path = candidate_encoded_path != NULL
        ? g_uri_unescape_string(candidate_encoded_path, NULL)
        : NULL;
    if (root_path == NULL || candidate_path == NULL
        || !g_path_is_absolute(root_path)
        || !g_path_is_absolute(candidate_path)) {
        trusted = FALSE;
    }
    char *canonical_root = root_path != NULL
        ? g_canonicalize_filename(root_path, "/")
        : NULL;
    char *canonical_candidate = candidate_path != NULL
        ? g_canonicalize_filename(candidate_path, "/")
        : NULL;
    if (canonical_root == NULL || canonical_candidate == NULL
        || !g_str_has_prefix(canonical_candidate, canonical_root)) {
        trusted = FALSE;
    }

    size_t root_length = canonical_root != NULL ? strlen(canonical_root) : 0U;
    if (root_length == 0U) {
        trusted = FALSE;
    }
    if (trusted
        && canonical_root[root_length - 1U] != G_DIR_SEPARATOR
        && canonical_candidate[root_length] != '\0'
        && canonical_candidate[root_length] != G_DIR_SEPARATOR) {
        trusted = FALSE;
    }
    g_free(canonical_candidate);
    g_free(canonical_root);
    g_free(candidate_path);
    g_free(root_path);
    g_uri_unref(candidate_uri);
    g_uri_unref(root_uri);
    g_clear_error(&error);
    return trusted;
}

gboolean empiier_native_bridge_fofline_uri(const char *uri) {
    if (uri == NULL || !g_str_has_prefix(uri, EMPIIEROS_FOFLINE_SPA_PREFIX)) {
        return FALSE;
    }
    if (strstr(uri, "..") != NULL
        || strchr(uri, '\\') != NULL
        || g_strstr_len(uri, -1, "%2e") != NULL
        || g_strstr_len(uri, -1, "%2E") != NULL
        || g_strstr_len(uri, -1, "%2f") != NULL
        || g_strstr_len(uri, -1, "%2F") != NULL
        || g_strstr_len(uri, -1, "%00") != NULL) {
        return FALSE;
    }
    return TRUE;
}

gboolean empiier_native_bridge_show_product(
    EmpiierNativeBridge *bridge,
    const char *entrypoint
) {
    if (bridge == NULL || bridge->web_view == NULL) {
        return FALSE;
    }
    if (g_strcmp0(entrypoint, EMPIIEROS_FOFLINE_SPA_START) != 0) {
        return FALSE;
    }
    empiier_native_bridge_hide_seat_browser(bridge);
    empiier_native_bridge_hide_meet(bridge);
    if (!empiier_oidc_inject_product(bridge)) {
        return FALSE;
    }
    webkit_web_view_load_uri(bridge->web_view, EMPIIEROS_FOFLINE_SPA_START);
    empiier_product_chrome_show(bridge);
    bridge->product_visible = TRUE;
    syslog(LOG_INFO, "product surface show origin=fofline-spa");
    return TRUE;
}

void empiier_native_bridge_hide_product(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->web_view == NULL) {
        return;
    }
    empiier_oidc_clear_product(bridge);
    empiier_native_bridge_hide_auth(bridge);
    empiier_native_bridge_hide_seat_browser(bridge);
    empiier_native_bridge_hide_meet(bridge);
    bridge->product_visible = FALSE;
    webkit_web_view_load_uri(bridge->web_view, EMPIIEROS_SHELL_URI);
    syslog(LOG_INFO, "product surface hide origin=empiier-shell");
}

void empiier_native_bridge_attach_chrome(
    EmpiierNativeBridge *bridge,
    GtkWidget *product_bar,
    GtkWidget *auth_box,
    WebKitWebView *auth_view
) {
    if (bridge == NULL) {
        return;
    }
    bridge->product_bar = product_bar;
    bridge->auth_box = auth_box;
    bridge->auth_view = auth_view;
}

gboolean empiier_native_bridge_escape_surface(EmpiierNativeBridge *bridge) {
    if (bridge == NULL) {
        return FALSE;
    }
    if (bridge->meet_visible) {
        empiier_native_bridge_hide_meet(bridge);
        return TRUE;
    }
    if (bridge->browser_visible) {
        empiier_native_bridge_hide_seat_browser(bridge);
        return TRUE;
    }
    if (bridge->auth_visible) {
        empiier_native_bridge_hide_auth(bridge);
        return TRUE;
    }
    if (bridge->product_visible
        || empiier_native_bridge_fofline_uri(webkit_web_view_get_uri(bridge->web_view))) {
        empiier_native_bridge_hide_product(bridge);
        return TRUE;
    }
    return FALSE;
}

gboolean empiier_shell_window_mode_service_allowed(void) {
    return g_file_test(
        EMPIIEROS_SHELL_WINDOW_MODE_ALLOW,
        G_FILE_TEST_EXISTS
    );
}

gboolean empiier_native_bridge_apply_window_mode(
    EmpiierNativeBridge *bridge,
    const char *mode
) {
    if (bridge == NULL || bridge->window == NULL || mode == NULL) {
        return FALSE;
    }
    if (g_strcmp0(mode, "windowed") == 0
        && !empiier_shell_window_mode_service_allowed()) {
        return FALSE;
    }
    if (g_strcmp0(mode, "windowed") == 0) {
        bridge->windowed = TRUE;
        gtk_window_unfullscreen(bridge->window);
        gtk_window_unmaximize(bridge->window);
        gtk_window_set_decorated(bridge->window, TRUE);
        gtk_window_set_resizable(bridge->window, TRUE);
        gtk_window_resize(bridge->window, 1280, 800);
    } else if (g_strcmp0(mode, "kiosk") == 0) {
        bridge->windowed = FALSE;
        gtk_window_set_decorated(bridge->window, FALSE);
        gtk_window_maximize(bridge->window);
        gtk_window_fullscreen(bridge->window);
    } else {
        return FALSE;
    }

    if (bridge->web_view != NULL) {
        char *mode_json = empiier_native_json_escape(mode);
        char *script = g_strdup_printf(
            "window.dispatchEvent(new CustomEvent('empiieros-window-mode',{detail:{mode:%s}}));",
            mode_json
        );
        webkit_web_view_evaluate_javascript(
            bridge->web_view,
            script,
            -1,
            NULL,
            "empiieros-window-mode",
            NULL,
            NULL,
            NULL
        );
        g_free(script);
        g_free(mode_json);
    }
    syslog(LOG_INFO, "shell window mode=%s", mode);
    return TRUE;
}

void empiier_native_bridge_publish_window_state(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->window == NULL || bridge->web_view == NULL) {
        return;
    }
    GdkWindow *gdk_window = gtk_widget_get_window(GTK_WIDGET(bridge->window));
    if (gdk_window != NULL
        && (gdk_window_get_state(gdk_window) & GDK_WINDOW_STATE_FULLSCREEN) != 0) {
        bridge->windowed = FALSE;
    }
    const char *mode = bridge->windowed ? "windowed" : "kiosk";
    char *mode_json = empiier_native_json_escape(mode);
    char *script = g_strdup_printf(
        "window.dispatchEvent(new CustomEvent('empiieros-window-mode',{detail:{mode:%s}}));",
        mode_json
    );
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-window-state",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
    g_free(mode_json);
    syslog(LOG_INFO, "shell window state observed mode=%s", mode);
}

static gboolean handle_window_mode(EmpiierNativeBridge *bridge, JSCValue *value, const char *request_id) {
    char *operation = property_string(value, "op");
    if (g_strcmp0(operation, "shell.window_mode") != 0) {
        g_free(operation);
        return FALSE;
    }
    g_free(operation);

    JSCValue *params = jsc_value_object_get_property(value, "params");
    char *mode = property_string(params, "mode");
    g_clear_object(&params);
    if (g_strcmp0(mode, "windowed") == 0
        && !empiier_shell_window_mode_service_allowed()) {
        char *failure = failure_response(
            request_id,
            "window_mode_service_disabled",
            "windowed mode requires service shell-window-mode.allow on disk"
        );
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(mode);
        return TRUE;
    }
    if (!empiier_native_bridge_apply_window_mode(bridge, mode)) {
        char *failure = failure_response(request_id, "invalid_window_mode", "mode must be kiosk or windowed");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(mode);
        return TRUE;
    }

    char *id_json = empiier_native_json_escape(request_id);
    char *mode_json = empiier_native_json_escape(mode);
    char *response = g_strdup_printf(
        "{\"protocol_version\":1,\"id\":%s,\"ok\":true,\"status\":\"applied\","
        "\"data\":{\"mode\":%s},\"error\":null}",
        id_json,
        mode_json
    );
    dispatch_response(bridge, response);
    g_free(response);
    g_free(mode_json);
    g_free(id_json);
    g_free(mode);
    return TRUE;
}

static gboolean handle_product_surface(
    EmpiierNativeBridge *bridge,
    JSCValue *value,
    const char *request_id
) {
    char *operation = property_string(value, "op");
    if (g_strcmp0(operation, "shell.product_surface") != 0) {
        g_free(operation);
        return FALSE;
    }
    g_free(operation);

    JSCValue *params = jsc_value_object_get_property(value, "params");
    char *action = property_string(params, "action");
    char *entrypoint = property_string(params, "entrypoint");
    g_clear_object(&params);
    gboolean ok = FALSE;
    const char *code = "invalid_product_surface";
    if (g_strcmp0(action, "hide") == 0) {
        empiier_native_bridge_hide_product(bridge);
        ok = TRUE;
    } else if (g_strcmp0(action, "show") == 0
        && empiier_native_bridge_show_product(bridge, entrypoint)) {
        ok = TRUE;
    } else {
        code = "entrypoint_not_allowlisted";
    }
    if (!ok) {
        char *failure = failure_response(request_id, code, "product surface denied");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(action);
        g_free(entrypoint);
        return TRUE;
    }
    char *id_json = empiier_native_json_escape(request_id);
    char *action_json = empiier_native_json_escape(action != NULL ? action : "");
    char *response = g_strdup_printf(
        "{\"protocol_version\":1,\"id\":%s,\"ok\":true,\"status\":\"applied\","
        "\"data\":{\"action\":%s},\"error\":null}",
        id_json,
        action_json
    );
    dispatch_response(bridge, response);
    g_free(response);
    g_free(action_json);
    g_free(id_json);
    g_free(action);
    g_free(entrypoint);
    return TRUE;
}


static gboolean handle_seat_browser_surface(
    EmpiierNativeBridge *bridge,
    JSCValue *value,
    const char *request_id
) {
    char *operation = property_string(value, "op");
    if (g_strcmp0(operation, "shell.seat_browser_surface") != 0) {
        g_free(operation);
        return FALSE;
    }
    g_free(operation);

    JSCValue *params = jsc_value_object_get_property(value, "params");
    char *action = property_string(params, "action");
    char *url = property_string(params, "url");
    char *query = property_string(params, "query");
    char *value_arg = property_string(params, "value");
    guint32 tab_id = property_u32(params, "tab_id");
    if (url == NULL) {
        url = property_string(params, "entrypoint");
    }
    g_clear_object(&params);
    gboolean ok = FALSE;
    const char *code = "invalid_seat_browser_surface";
    const char *arg = NULL;
    if (query != NULL && query[0] != '\0') {
        arg = query;
    } else if (value_arg != NULL && value_arg[0] != '\0') {
        arg = value_arg;
    } else if (url != NULL) {
        arg = url;
    }
    if (g_strcmp0(action, "hide") == 0) {
        empiier_native_bridge_hide_seat_browser(bridge);
        ok = TRUE;
    } else if (g_strcmp0(action, "show") == 0 || g_strcmp0(action, "load") == 0) {
        if (tab_id != 0) {
            ok = empiier_native_bridge_seat_browser_tab(bridge, "load", tab_id, url);
        } else {
            ok = empiier_native_bridge_show_seat_browser(bridge, url);
        }
        if (!ok) {
            code = "navigation_denied";
        }
    } else if (g_strcmp0(action, "tab_open") == 0
        || g_strcmp0(action, "tab_select") == 0
        || g_strcmp0(action, "tab_close") == 0) {
        ok = empiier_native_bridge_seat_browser_tab(bridge, action, tab_id, url);
        if (!ok) {
            code = "tab_denied";
        }
    } else {
        ok = empiier_native_bridge_seat_browser_command(bridge, action, arg);
    }
    if (!ok) {
        char *failure = failure_response(request_id, code, "seat browser surface denied");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(action);
        g_free(url);
        g_free(query);
        g_free(value_arg);
        return TRUE;
    }
    char *id_json = empiier_native_json_escape(request_id);
    char *action_json = empiier_native_json_escape(action != NULL ? action : "");
    char *response = g_strdup_printf(
        "{\"protocol_version\":1,\"id\":%s,\"ok\":true,\"status\":\"applied\","
        "\"data\":{\"action\":%s},\"error\":null}",
        id_json,
        action_json
    );
    dispatch_response(bridge, response);
    g_free(response);
    g_free(action_json);
    g_free(id_json);
    g_free(action);
    g_free(url);
    g_free(query);
    g_free(value_arg);
    return TRUE;
}

static gboolean handle_auth_surface(
    EmpiierNativeBridge *bridge,
    JSCValue *value,
    const char *request_id
) {
    char *operation = property_string(value, "op");
    if (g_strcmp0(operation, "shell.auth_surface") != 0) {
        g_free(operation);
        return FALSE;
    }
    g_free(operation);

    JSCValue *params = jsc_value_object_get_property(value, "params");
    char *action = property_string(params, "action");
    char *entrypoint = property_string(params, "entrypoint");
    g_clear_object(&params);
    gboolean ok = FALSE;
    const char *code = "invalid_auth_surface";
    if (g_strcmp0(action, "hide") == 0) {
        empiier_native_bridge_hide_auth(bridge);
        ok = TRUE;
    } else if (g_strcmp0(action, "show") == 0) {
        bridge->auth_fresh_budget = 1;
        bridge->auth_fresh_busy = FALSE;
        ok = empiier_native_bridge_show_auth(bridge, entrypoint);
        if (!ok) {
            code = "entrypoint_not_allowlisted";
        }
    } else {
        code = "entrypoint_not_allowlisted";
    }
    if (!ok) {
        char *failure = failure_response(request_id, code, "auth surface denied");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(action);
        g_free(entrypoint);
        return TRUE;
    }
    char *id_json = empiier_native_json_escape(request_id);
    char *action_json = empiier_native_json_escape(action != NULL ? action : "");
    char *response = g_strdup_printf(
        "{\"protocol_version\":1,\"id\":%s,\"ok\":true,\"status\":\"applied\","
        "\"data\":{\"action\":%s},\"error\":null}",
        id_json,
        action_json
    );
    dispatch_response(bridge, response);
    g_free(response);
    g_free(action_json);
    g_free(id_json);
    g_free(action);
    g_free(entrypoint);
    return TRUE;
}

static void on_script_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *result,
    gpointer user_data
) {
    (void)manager;
    EmpiierNativeBridge *bridge = user_data;
    const char *uri = webkit_web_view_get_uri(bridge->web_view);
    if (!trusted_shell_uri(bridge, uri)) {
        return;
    }

    JSCValue *value = webkit_javascript_result_get_js_value(result);
    if (value == NULL || !jsc_value_is_object(value)) {
        dispatch_response(
            bridge,
            "{\"protocol_version\":1,\"id\":\"\",\"ok\":false,\"status\":\"bridge_error\","
            "\"data\":null,\"error\":{\"code\":\"invalid_message\",\"message\":\"message must be an object\"}}"
        );
        return;
    }
    char *request_id = property_string(value, "id");
    if (request_id == NULL) {
        request_id = g_strdup("");
    }
    if (handle_window_mode(bridge, value, request_id)) {
        g_free(request_id);
        return;
    }
    if (handle_product_surface(bridge, value, request_id)) {
        g_free(request_id);
        return;
    }
    if (handle_auth_surface(bridge, value, request_id)) {
        g_free(request_id);
        return;
    }
    if (handle_seat_browser_surface(bridge, value, request_id)) {
        g_free(request_id);
        return;
    }
    char *blocked = property_string(value, "op");
    if (g_strcmp0(blocked, "launcher.oidc_inject") == 0) {
        char *failure = failure_response(request_id, "token_ipc_forbidden", "oidc inject is native-only");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(blocked);
        g_free(request_id);
        return;
    }
    g_free(blocked);

    char *request_json = jsc_value_to_json(value, 0);
    if (request_json == NULL) {
        char *failure = failure_response(request_id, "serialization_failed", "cannot serialize bridge request");
        dispatch_response(bridge, failure);
        g_free(failure);
        g_free(request_id);
        return;
    }

    BridgeJob *job = g_new0(BridgeJob, 1);
    job->bridge = bridge;
    job->request_json = request_json;
    job->request_id = request_id;

    GTask *task = g_task_new(NULL, NULL, exchange_finished, bridge);
    g_task_set_task_data(task, job, (GDestroyNotify)bridge_job_free);
    g_task_run_in_thread(task, exchange_thread);
    g_object_unref(task);
}

void empiier_native_bridge_init(
    EmpiierNativeBridge *bridge,
    WebKitUserContentManager *manager,
    WebKitWebView *web_view,
    GtkWindow *window,
    const char *trusted_uri_prefix,
    const char *socket_path
) {
    g_return_if_fail(bridge != NULL);
    g_return_if_fail(WEBKIT_IS_USER_CONTENT_MANAGER(manager));
    g_return_if_fail(WEBKIT_IS_WEB_VIEW(web_view));
    g_return_if_fail(GTK_IS_WINDOW(window));

    bridge->web_view = web_view;
    bridge->window = window;
    bridge->trusted_uri_prefix = trusted_uri_prefix;
    bridge->socket_path = socket_path;
    bridge->windowed = FALSE;
    bridge->overlay = NULL;
    bridge->manager = manager;
    bridge->product_bar = NULL;
    bridge->auth_box = NULL;
    bridge->auth_dimmer = NULL;
    bridge->auth_card = NULL;
    bridge->auth_view = NULL;
    bridge->oidc_script = NULL;
    bridge->product_visible = FALSE;
    bridge->auth_visible = FALSE;
    bridge->auth_fresh_budget = 0;
    bridge->auth_fresh_busy = FALSE;
    bridge->browser_box = NULL;
    bridge->browser_view = NULL;
    bridge->browser_stack = NULL;
    memset(bridge->browser_views, 0, sizeof(bridge->browser_views));
    memset(bridge->browser_tab_ids, 0, sizeof(bridge->browser_tab_ids));
    bridge->browser_tab_strip = NULL;
    memset(bridge->browser_tab_buttons, 0, sizeof(bridge->browser_tab_buttons));
    bridge->browser_tab_new = NULL;
    bridge->browser_visible = FALSE;
    bridge->meet_box = NULL;
    bridge->meet_view = NULL;
    bridge->meet_visible = FALSE;
    bridge->meet_has_bounds = FALSE;
    bridge->oidc_product_busy = FALSE;

    g_signal_connect(
        manager,
        "script-message-received::empiieros",
        G_CALLBACK(on_script_message),
        bridge
    );
    if (!webkit_user_content_manager_register_script_message_handler(manager, "empiieros")) {
        g_error("failed to register empiieros script message handler");
    }
    empiier_oidc_register_product_bridge(bridge);
    empiier_network_register_product_bridge(bridge);
    empiier_meet_register_product_bridge(bridge);
    empiier_seat_browser_register_product_bridge(bridge);
}

gboolean empiier_native_bridge_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    WebKitNavigationPolicyDecision *nav;
    WebKitNavigationAction *action;
    WebKitURIRequest *request;
    const char *uri;
    const char *here;
    const char *frame;

    if (type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        webkit_policy_decision_ignore(decision);
        return TRUE;
    }
    if (type != WEBKIT_POLICY_DECISION_TYPE_NAVIGATION_ACTION) {
        return FALSE;
    }

    nav = WEBKIT_NAVIGATION_POLICY_DECISION(decision);
    action = webkit_navigation_policy_decision_get_navigation_action(nav);
    request = webkit_navigation_action_get_request(action);
    uri = webkit_uri_request_get_uri(request);
    if (trusted_shell_uri(bridge, uri)
        || empiier_native_bridge_fofline_uri(uri)
        || g_strcmp0(uri, "about:blank") == 0) {
        return FALSE;
    }
    if (empiier_native_bridge_meet_host_ok(uri)) {
        here = web_view != NULL ? webkit_web_view_get_uri(web_view) : NULL;
        frame = webkit_navigation_action_get_frame_name(action);
        if (!empiier_native_bridge_fofline_uri(here)) {
            webkit_policy_decision_ignore(decision);
            syslog(LOG_INFO, "fofline product navigation denied code=not_fofline_host");
            return TRUE;
        }
        if (frame == NULL && webkit_navigation_action_is_user_gesture(action)) {
            webkit_policy_decision_ignore(decision);
            syslog(LOG_INFO, "fofline product navigation denied code=main_frame");
            return TRUE;
        }
        webkit_policy_decision_use(decision);
        syslog(LOG_INFO, "fofline product meet navigation kind=iframe");
        return TRUE;
    }

    syslog(LOG_INFO, "fofline product navigation denied code=origin_denied");
    webkit_policy_decision_ignore(decision);
    return TRUE;
}
