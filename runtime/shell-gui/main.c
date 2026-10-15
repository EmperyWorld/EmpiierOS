/**
 * empiieros-shell-gui — Sovereign WebKitGTK Wayland Kiosk Browser for EmpiierOS.
 *
 * Kinship: `eliteew-ts-kiosk`, `session/SHELL-GUI.md`, and `runtime/src/shell_gui.rs`.
 * Plane: Runtime / Substrate kiosk runner.
 * Standard: GTK+ 3.0, WebKit2GTK 4.1, C99.
 */

#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <signal.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define DEFAULT_SHELL_URL "http://127.0.0.1:17420/"
#define FALLBACK_FILE_URL "file:///opt/empiieros/shell/index.html"
#define KIOSK_WINDOW_TITLE "Рабочий стол EliteEW"
#define KIOSK_WINDOW_ROLE  "empiieros-shell"

static WebKitWebView *g_web_view = NULL;
static gboolean g_is_windowed = FALSE;

static void on_sigint(int sig) {
    (void)sig;
    if (gtk_main_level() > 0) {
        gtk_main_quit();
    }
}

static gboolean on_context_menu(WebKitWebView *web_view,
                                WebKitContextMenu *context_menu,
                                GdkEvent *event,
                                WebKitHitTestResult *hit_test_result,
                                gpointer user_data) {
    (void)web_view;
    (void)context_menu;
    (void)event;
    (void)hit_test_result;
    (void)user_data;
    /* Hardened kiosk security: suppress context menu */
    return TRUE;
}

static gboolean on_delete_event(GtkWidget *widget, GdkEvent *event, gpointer user_data) {
    (void)widget;
    (void)event;
    (void)user_data;
    g_print("empiieros-shell-gui: kiosk close ignored (delete-event)\n");
    return TRUE;
}

static gboolean on_key_press(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)widget;
    WebKitWebView *view = WEBKIT_WEB_VIEW(user_data);
    if ((event->state & GDK_MOD1_MASK) != 0 && event->keyval == GDK_KEY_F4) {
        g_print("empiieros-shell-gui: kiosk close ignored (Alt+F4)\n");
        return TRUE;
    }
    /* F5 or Ctrl+R: reload kiosk shell */
    if (event->keyval == GDK_KEY_F5 ||
        ((event->state & GDK_CONTROL_MASK) && (event->keyval == GDK_KEY_r || event->keyval == GDK_KEY_R))) {
        g_print("empiieros-shell-gui: reload triggered via keypress\n");
        webkit_web_view_reload_bypass_cache(view);
        return TRUE;
    }
    return FALSE;
}

static int g_retry_count = 0;

static gboolean do_retry_load(gpointer user_data) {
    WebKitWebView *view = WEBKIT_WEB_VIEW(user_data);
    g_retry_count++;
    if (g_retry_count <= 5) {
        g_printerr("empiieros-shell-gui: retry attempt %d connecting to %s...\n", g_retry_count, DEFAULT_SHELL_URL);
        webkit_web_view_load_uri(view, DEFAULT_SHELL_URL);
    } else {
        g_printerr("empiieros-shell-gui: fallback to direct static file %s\n", FALLBACK_FILE_URL);
        webkit_web_view_load_uri(view, FALLBACK_FILE_URL);
    }
    return G_SOURCE_REMOVE;
}

static void on_load_changed(WebKitWebView *view, WebKitLoadEvent event, gpointer user_data) {
    (void)user_data;
    if (event == WEBKIT_LOAD_FINISHED) {
        g_retry_count = 0;
        g_print("empiieros-shell-gui: page load finished successfully: %s\n",
                webkit_web_view_get_uri(view));
    }
}

static void on_load_failed(WebKitWebView *view,
                           WebKitLoadEvent load_event,
                           char *failing_uri,
                           GError *error,
                           gpointer user_data) {
    (void)load_event;
    (void)user_data;
    g_printerr("empiieros-shell-gui: load failed for URI '%s': %s (code %d)\n",
               failing_uri, error->message, error->code);
    /* If runtime loopback is temporarily not ready, retry non-blocking or fallback to file */
    if (strncmp(failing_uri, DEFAULT_SHELL_URL, strlen(DEFAULT_SHELL_URL)) == 0) {
        g_timeout_add_seconds(1, do_retry_load, view);
    }
}

static gboolean on_window_map(GtkWidget *widget, GdkEvent *event, gpointer user_data) {
    (void)event;
    (void)user_data;
    gtk_window_maximize(GTK_WINDOW(widget));
    gtk_window_fullscreen(GTK_WINDOW(widget));
    return FALSE;
}

static gboolean on_window_state_event(GtkWidget *widget, GdkEventWindowState *event, gpointer user_data) {
    (void)user_data;
    if (!g_is_windowed) {
        if (!(event->new_window_state & GDK_WINDOW_STATE_FULLSCREEN)) {
            gtk_window_fullscreen(GTK_WINDOW(widget));
        }
        if (!(event->new_window_state & GDK_WINDOW_STATE_MAXIMIZED)) {
            gtk_window_maximize(GTK_WINDOW(widget));
        }
    }
    return FALSE;
}

static void on_title_changed(GObject *object, GParamSpec *pspec, gpointer user_data) {
    (void)pspec;
    WebKitWebView *view = WEBKIT_WEB_VIEW(object);
    GtkWindow *window = GTK_WINDOW(user_data);
    const char *title = webkit_web_view_get_title(view);
    if (title && strlen(title) > 0) {
        gtk_window_set_title(window, title);
        if (strstr(title, "[windowed]") != NULL) {
            if (!g_is_windowed) {
                g_is_windowed = TRUE;
                g_print("empiieros-shell-gui: switching to windowed mode\n");
                gtk_window_unfullscreen(window);
                gtk_window_set_decorated(window, TRUE);
                gtk_window_resize(window, 1280, 800);
            }
        } else {
            if (g_is_windowed) {
                g_is_windowed = FALSE;
                g_print("empiieros-shell-gui: switching to fullscreen kiosk mode\n");
                gtk_window_set_decorated(window, FALSE);
                gtk_window_fullscreen(window);
            }
        }
    }
}

static gboolean on_scroll_event(GtkWidget *widget, GdkEventScroll *event, gpointer user_data) {
    (void)widget;
    WebKitWebView *view = WEBKIT_WEB_VIEW(user_data);
    double dx = 0.0, dy = 0.0;
    if (gdk_event_get_scroll_deltas((GdkEvent *)event, &dx, &dy)) {
        char script[128];
        snprintf(script, sizeof(script), "window.scrollBy({top: %f * 50, behavior: 'auto'});", dy);
        webkit_web_view_run_javascript(view, script, NULL, NULL, NULL);
        return TRUE;
    } else if (event->direction == GDK_SCROLL_UP) {
        webkit_web_view_run_javascript(view, "window.scrollBy({top: -100, behavior: 'auto'});", NULL, NULL, NULL);
        return TRUE;
    } else if (event->direction == GDK_SCROLL_DOWN) {
        webkit_web_view_run_javascript(view, "window.scrollBy({top: 100, behavior: 'auto'});", NULL, NULL, NULL);
        return TRUE;
    }
    return FALSE;
}

int main(int argc, char *argv[]) {
    signal(SIGINT, on_sigint);
    signal(SIGTERM, on_sigint);

    gtk_init(&argc, &argv);

    const char *url = DEFAULT_SHELL_URL;
    const char *env_url = getenv("EMPIIEROS_SHELL_URL");
    if (env_url && strlen(env_url) > 0) {
        url = env_url;
    }
    if (argc > 1 && strlen(argv[1]) > 0) {
        url = argv[1];
    }

    g_print("empiieros-shell-gui: starting kiosk window, target URL: %s\n", url);

    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), KIOSK_WINDOW_TITLE);
    gtk_window_set_role(GTK_WINDOW(window), KIOSK_WINDOW_ROLE);
    gtk_window_set_decorated(GTK_WINDOW(window), FALSE);

    /* Set default size to full monitor resolution to prevent small square fallback */
    GdkDisplay *display = gdk_display_get_default();
    if (display) {
        GdkMonitor *monitor = gdk_display_get_primary_monitor(display);
        if (!monitor) {
            monitor = gdk_display_get_monitor(display, 0);
        }
        if (monitor) {
            GdkRectangle geom;
            gdk_monitor_get_geometry(monitor, &geom);
            gtk_window_set_default_size(GTK_WINDOW(window), geom.width, geom.height);
        }
    }

    gtk_window_maximize(GTK_WINDOW(window));
    gtk_window_fullscreen(GTK_WINDOW(window));

    g_signal_connect(window, "delete-event", G_CALLBACK(on_delete_event), NULL);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    g_signal_connect(window, "map-event", G_CALLBACK(on_window_map), NULL);
    g_signal_connect(window, "window-state-event", G_CALLBACK(on_window_state_event), NULL);

    /* Enable scroll events and focus for mouse wheel support */
    gtk_widget_add_events(window, GDK_SCROLL_MASK | GDK_SMOOTH_SCROLL_MASK);

    WebKitSettings *settings = webkit_settings_new();
    gboolean debug_mode = (getenv("EMPIIEROS_SHELL_DEBUG") != NULL);
    webkit_settings_set_enable_developer_extras(settings, debug_mode);
    webkit_settings_set_enable_webgl(settings, TRUE);
    webkit_settings_set_enable_page_cache(settings, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(settings, TRUE);
    webkit_settings_set_allow_universal_access_from_file_urls(settings, TRUE);
    webkit_settings_set_enable_smooth_scrolling(settings, FALSE);

    WebKitUserContentManager *content_mgr = webkit_user_content_manager_new();
    GtkWidget *view_widget = g_object_new(WEBKIT_TYPE_WEB_VIEW,
                                          "settings", settings,
                                          "user-content-manager", content_mgr,
                                          NULL);
    g_web_view = WEBKIT_WEB_VIEW(view_widget);
    gtk_widget_add_events(view_widget, GDK_SCROLL_MASK | GDK_SMOOTH_SCROLL_MASK);

    g_signal_connect(view_widget, "context-menu", G_CALLBACK(on_context_menu), NULL);
    g_signal_connect(window, "key-press-event", G_CALLBACK(on_key_press), g_web_view);
    g_signal_connect(window, "scroll-event", G_CALLBACK(on_scroll_event), g_web_view);
    g_signal_connect(view_widget, "scroll-event", G_CALLBACK(on_scroll_event), g_web_view);
    g_signal_connect(view_widget, "load-failed", G_CALLBACK(on_load_failed), NULL);
    g_signal_connect(view_widget, "load-changed", G_CALLBACK(on_load_changed), NULL);
    g_signal_connect(view_widget, "notify::title", G_CALLBACK(on_title_changed), window);

    gtk_container_add(GTK_CONTAINER(window), view_widget);
    webkit_web_view_load_uri(g_web_view, url);

    gtk_widget_show_all(window);
    gtk_widget_set_can_focus(view_widget, TRUE);
    gtk_widget_grab_focus(view_widget);
    gtk_main();

    g_print("empiieros-shell-gui: clean kiosk shutdown\n");
    return 0;
}
