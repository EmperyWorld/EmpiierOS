#define _POSIX_C_SOURCE 200809L

/* LinkornEW seat window. Not the FoFline shell. A tile message may only
 * be console, browser, or command-post. The child is linkornew-host. */

#include <errno.h>
#include <gio/gio.h>
#include <glib.h>
#include <gtk/gtk.h>
#include <jsc/jsc.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>
#include <webkit2/webkit2.h>

#define HOST_BIN "/usr/lib/linkornew/linkornew-host"
#define DESK_URI "linkornew://desk/index.html"
#define LAUNCH_PREFIX "linkornew://launch/"
#define MAX_ASSET_BYTES (16U * 1024U * 1024U)

extern char **environ;

static char *shell_root = NULL;
static WebKitWebView *seat_web = NULL;
static gint64 last_launch_us = 0;
static char last_launch_action[32] = "";

static const unsigned char PIXEL_GIF[] = {
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00,
    0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
    0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b
};

static gboolean action_ok(const char *action) {
    return g_strcmp0(action, "console") == 0
        || g_strcmp0(action, "browser") == 0
        || g_strcmp0(action, "command-post") == 0;
}

static guint host_generation = 0;

static void tell_desk(const char *message) {
    if (seat_web == NULL || message == NULL) {
        return;
    }
    char *escaped = g_strescape(message, NULL);
    char *script = g_strdup_printf(
        "(function(){var n=document.getElementById('notice');if(n)n.textContent='%s';})();",
        escaped
    );
    webkit_web_view_evaluate_javascript(
        seat_web,
        script,
        -1,
        NULL,
        "linkornew-notice",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
    g_free(escaped);
}

typedef struct {
    char *message;
} DeskNotice;

static gboolean tell_desk_idle(gpointer data) {
    DeskNotice *notice = data;
    tell_desk(notice->message);
    g_free(notice->message);
    g_free(notice);
    return G_SOURCE_REMOVE;
}

static void tell_desk_soon(const char *message) {
    DeskNotice *notice = g_new0(DeskNotice, 1);
    notice->message = g_strdup(message);
    g_idle_add(tell_desk_idle, notice);
}

static gboolean tell_if_still_running(gpointer data) {
    guint *generation = data;
    if (*generation == host_generation) {
        tell_desk("waiting for program");
    }
    g_free(generation);
    return G_SOURCE_REMOVE;
}

static void on_host_exit(GPid pid, gint status, gpointer user_data) {
    char *action = user_data;
    char line[160];
    int code = -1;
    if (WIFEXITED(status)) {
        code = WEXITSTATUS(status);
    }
    gchar *detail = NULL;
    gsize length = 0;
    if (g_file_get_contents("/var/lib/empiieros-runtime/linkorn/chrome/host.last", &detail, &length, NULL)
        && detail != NULL
        && length > 0) {
        for (gsize i = 0; i < length; i++) {
            if (detail[i] == '\n' || detail[i] == '\r') {
                detail[i] = ' ';
            }
            if ((unsigned char)detail[i] < 0x20 || (unsigned char)detail[i] > 0x7e) {
                detail[i] = '?';
            }
        }
        g_snprintf(line, sizeof line, "%s exit %d %s", action != NULL ? action : "?", code, detail);
        g_free(detail);
    } else {
        g_snprintf(line, sizeof line, "%s host_exit %d", action != NULL ? action : "?", code);
    }
    g_warning("linkornew-seat: %s", line);
    host_generation++;
    tell_desk_soon(line);
    g_free(action);
    g_spawn_close_pid(pid);
}

static void launch_host(const char *action) {
    if (!action_ok(action)) {
        tell_desk_soon("refuse action");
        return;
    }
    gint64 now = g_get_monotonic_time();
    if (g_strcmp0(action, last_launch_action) == 0 && now - last_launch_us < 400000) {
        return;
    }
    last_launch_us = now;
    g_strlcpy(last_launch_action, action, sizeof last_launch_action);

    if (access(HOST_BIN, X_OK) != 0) {
        g_warning("linkornew-seat: missing %s", HOST_BIN);
        tell_desk_soon("missing linkornew-host");
        return;
    }
    char *argv[] = { HOST_BIN, (char *)action, NULL };
    pid_t pid = 0;
    int rc = posix_spawn(&pid, HOST_BIN, NULL, NULL, argv, environ);
    if (rc != 0) {
        g_warning("linkornew-seat: spawn %s %s: %s", HOST_BIN, action, g_strerror(rc));
        char line[96];
        g_snprintf(line, sizeof line, "spawn_failed %s", g_strerror(rc));
        tell_desk_soon(line);
        return;
    }
    host_generation++;
    guint *generation = g_new(guint, 1);
    *generation = host_generation;
    g_child_watch_add((GPid)pid, on_host_exit, g_strdup(action));
    g_timeout_add_seconds(3, tell_if_still_running, generation);
    char started[64];
    g_snprintf(started, sizeof started, "host running %s", action);
    tell_desk_soon(started);
}

static char *action_from_jsc(JSCValue *value) {
    if (value == NULL) {
        return NULL;
    }
    if (jsc_value_is_string(value)) {
        return jsc_value_to_string(value);
    }
    if (jsc_value_is_object(value)) {
        JSCValue *prop = jsc_value_object_get_property(value, "action");
        char *text = NULL;
        if (prop != NULL && jsc_value_is_string(prop)) {
            text = jsc_value_to_string(prop);
        }
        if (prop != NULL) {
            g_object_unref(prop);
        }
        return text;
    }
    return jsc_value_to_string(value);
}

static void on_seat_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *result,
    gpointer user_data
) {
    (void)manager;
    (void)user_data;
    JSCValue *value = webkit_javascript_result_get_js_value(result);
    char *action = action_from_jsc(value);
    if (action == NULL) {
        return;
    }
    launch_host(action);
    g_free(action);
}

static void finish_error(WebKitURISchemeRequest *request, const char *message) {
    GError *error = g_error_new_literal(G_IO_ERROR, G_IO_ERROR_FAILED, message);
    webkit_uri_scheme_request_finish_error(request, error);
    g_error_free(error);
}

static void finish_gif(WebKitURISchemeRequest *request) {
    gpointer copy = g_memdup2(PIXEL_GIF, sizeof PIXEL_GIF);
    GInputStream *stream = g_memory_input_stream_new_from_data(copy, sizeof PIXEL_GIF, g_free);
    webkit_uri_scheme_request_finish(request, stream, (gint64)sizeof PIXEL_GIF, "image/gif");
    g_object_unref(stream);
}

static void serve_desk(WebKitURISchemeRequest *request, gpointer user_data) {
    (void)user_data;
    const char *uri = webkit_uri_scheme_request_get_uri(request);
    GError *error = NULL;
    GUri *parsed = g_uri_parse(uri, G_URI_FLAGS_ENCODED_PATH, &error);
    if (parsed == NULL) {
        g_clear_error(&error);
        finish_error(request, "invalid desk URI");
        return;
    }
    if (g_str_has_prefix(uri, LAUNCH_PREFIX)
        && g_strcmp0(g_uri_get_scheme(parsed), "linkornew") == 0
        && g_strcmp0(g_uri_get_host(parsed), "launch") == 0
        && g_uri_get_query(parsed) == NULL
        && g_uri_get_fragment(parsed) == NULL) {
        const char *path = g_uri_get_path(parsed);
        const char *action = (path != NULL && path[0] == '/') ? path + 1 : path;
        if (!action_ok(action)) {
            g_uri_unref(parsed);
            finish_error(request, "launch refused");
            return;
        }
        launch_host(action);
        finish_gif(request);
        g_uri_unref(parsed);
        return;
    }

    if (g_strcmp0(g_uri_get_scheme(parsed), "linkornew") != 0
        || g_uri_get_query(parsed) != NULL
        || g_uri_get_fragment(parsed) != NULL) {
        g_uri_unref(parsed);
        finish_error(request, "desk asset refused");
        return;
    }

    const char *encoded = g_uri_get_path(parsed);
    gboolean trusted = g_strcmp0(g_uri_get_host(parsed), "desk") == 0
        && encoded != NULL
        && g_str_has_prefix(encoded, "/");
    char *decoded = trusted ? g_uri_unescape_string(encoded, NULL) : NULL;
    char *canonical_root = g_canonicalize_filename(shell_root, "/");
    char *candidate = decoded != NULL
        ? g_canonicalize_filename(decoded + 1, canonical_root)
        : NULL;
    size_t root_length = strlen(canonical_root);
    struct stat metadata;
    if (candidate == NULL
        || strstr(decoded, "..") != NULL
        || !g_str_has_prefix(candidate, canonical_root)
        || (candidate[root_length] != G_DIR_SEPARATOR && candidate[root_length] != '\0')
        || lstat(candidate, &metadata) != 0
        || !S_ISREG(metadata.st_mode)
        || metadata.st_size < 0
        || (guint64)metadata.st_size > MAX_ASSET_BYTES) {
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded);
        g_uri_unref(parsed);
        finish_error(request, "desk asset refused");
        return;
    }
    char *contents = NULL;
    gsize length = 0;
    if (!g_file_get_contents(candidate, &contents, &length, &error)) {
        g_clear_error(&error);
        g_free(candidate);
        g_free(canonical_root);
        g_free(decoded);
        g_uri_unref(parsed);
        finish_error(request, "desk asset read failed");
        return;
    }
    gboolean uncertain = FALSE;
    char *content_type = g_content_type_guess(candidate, (const guchar *)contents, length, &uncertain);
    (void)uncertain;
    char *guessed = content_type != NULL ? g_content_type_get_mime_type(content_type) : NULL;
    char *mime = NULL;
    if (g_str_has_suffix(candidate, ".js")) {
        mime = g_strdup("text/javascript;charset=utf-8");
    } else if (g_str_has_suffix(candidate, ".css")) {
        mime = g_strdup("text/css;charset=utf-8");
    } else if (g_str_has_suffix(candidate, ".html")) {
        mime = g_strdup("text/html;charset=utf-8");
    } else {
        mime = guessed != NULL ? g_strdup(guessed) : g_strdup("application/octet-stream");
    }
    GInputStream *stream = g_memory_input_stream_new_from_data(contents, length, g_free);
    webkit_uri_scheme_request_finish(
        request,
        stream,
        (gint64)length,
        mime
    );
    g_object_unref(stream);
    g_free(mime);
    g_free(guessed);
    g_free(content_type);
    g_free(candidate);
    g_free(canonical_root);
    g_free(decoded);
    g_uri_unref(parsed);
}

static void activate(GtkApplication *app, gpointer user_data) {
    (void)user_data;
    WebKitUserContentManager *manager = webkit_user_content_manager_new();
    g_signal_connect(manager, "script-message-received::linkornew", G_CALLBACK(on_seat_message), NULL);
    if (!webkit_user_content_manager_register_script_message_handler(manager, "linkornew")) {
        g_warning("linkornew-seat: message handler refused");
    }
    WebKitWebContext *context = webkit_web_context_new_ephemeral();
    webkit_web_context_register_uri_scheme(context, "linkornew", serve_desk, NULL, NULL);
    WebKitSecurityManager *security = webkit_web_context_get_security_manager(context);
    webkit_security_manager_register_uri_scheme_as_secure(security, "linkornew");
    webkit_security_manager_register_uri_scheme_as_cors_enabled(security, "linkornew");
    WebKitWebView *web = WEBKIT_WEB_VIEW(g_object_new(
        WEBKIT_TYPE_WEB_VIEW,
        "user-content-manager", manager,
        "web-context", context,
        NULL
    ));
    seat_web = web;
    GtkWidget *window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "LinkornEW");
    gtk_window_set_default_size(GTK_WINDOW(window), 1920, 1080);
    gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(web));
    webkit_web_view_load_uri(web, DESK_URI);
    gtk_widget_show_all(window);
    gtk_window_fullscreen(GTK_WINDOW(window));
    g_object_unref(manager);
}

int main(int argc, char **argv) {
    const char *root = argc > 1 ? argv[1] : "/opt/linkornew/shell";
    shell_root = g_canonicalize_filename(root, "/");
    GtkApplication *app = gtk_application_new("ew.linkorn.seat", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int status = g_application_run(G_APPLICATION(app), 1, argv);
    g_object_unref(app);
    g_free(shell_root);
    return status;
}
