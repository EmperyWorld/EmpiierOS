#define _POSIX_C_SOURCE 200809L

#include "boot_splash.h"

#include <gst/gst.h>
#include <syslog.h>

#define SPLASH_VOLUME 0.25
#define SPLASH_FAILSAFE_MS 20000
#define SPLASH_NAME "start_os.mp4"

typedef struct {
    GstElement *playbin;
    GtkWidget *host;
    GtkWidget *video;
    EmpiierNativeBridge *bridge;
    guint bus_watch;
    guint failsafe;
    guint start_watch;
    gboolean started;
    gboolean finished;
} BootSplash;

static const char *const k_media_roots[] = {
    "/opt/empiieros/shell/product-chrome",
    "/usr/share/empiieros/product-chrome",
};

static gboolean resolve_splash(char *out, size_t out_len) {
    size_t i;

    for (i = 0; i < sizeof(k_media_roots) / sizeof(k_media_roots[0]); i++) {
        if (g_snprintf(out, out_len, "%s/%s", k_media_roots[i], SPLASH_NAME) < 0) {
            continue;
        }
        if (g_file_test(out, G_FILE_TEST_IS_REGULAR)) {
            return TRUE;
        }
    }
    return FALSE;
}

static void splash_paint_black(GtkWidget *host) {
    GtkCssProvider *css = gtk_css_provider_new();

    gtk_css_provider_load_from_data(
        css,
        "#empiieros-boot-splash { background-color: #000000; }",
        -1,
        NULL
    );
    gtk_style_context_add_provider(
        gtk_widget_get_style_context(host),
        GTK_STYLE_PROVIDER(css),
        GTK_STYLE_PROVIDER_PRIORITY_USER
    );
    g_object_unref(css);
}

static void splash_finish(BootSplash *splash) {
    if (splash == NULL || splash->finished) {
        return;
    }
    splash->finished = TRUE;
    if (splash->failsafe != 0) {
        g_source_remove(splash->failsafe);
        splash->failsafe = 0;
    }
    if (splash->start_watch != 0) {
        g_source_remove(splash->start_watch);
        splash->start_watch = 0;
    }
    if (splash->bus_watch != 0) {
        g_source_remove(splash->bus_watch);
        splash->bus_watch = 0;
    }
    if (splash->playbin != NULL) {
        gst_element_set_state(splash->playbin, GST_STATE_NULL);
        gst_object_unref(splash->playbin);
        splash->playbin = NULL;
    }
    if (splash->host != NULL) {
        gtk_widget_hide(splash->host);
        gtk_widget_set_no_show_all(splash->host, TRUE);
    }
    if (splash->bridge != NULL) {
        empiier_product_chrome_show(splash->bridge);
        syslog(LOG_INFO, "boot splash revealed hud");
    }
    syslog(LOG_INFO, "boot splash finished volume=0.25 alsa released");
}

static gboolean on_failsafe(gpointer user_data) {
    BootSplash *splash = user_data;

    splash->failsafe = 0;
    syslog(LOG_INFO, "boot splash failsafe");
    splash_finish(splash);
    return FALSE;
}

static gboolean on_bus(GstBus *bus, GstMessage *message, gpointer user_data) {
    BootSplash *splash = user_data;
    GError *error = NULL;
    gchar *debug = NULL;
    GstState old_state = GST_STATE_VOID_PENDING;
    GstState new_state = GST_STATE_VOID_PENDING;
    GstState pending = GST_STATE_VOID_PENDING;

    (void)bus;
    switch (GST_MESSAGE_TYPE(message)) {
    case GST_MESSAGE_EOS:
        splash_finish(splash);
        break;
    case GST_MESSAGE_ERROR:
        gst_message_parse_error(message, &error, &debug);
        syslog(
            LOG_INFO,
            "boot splash error %s",
            error != NULL ? error->message : "unknown"
        );
        g_clear_error(&error);
        g_free(debug);
        splash_finish(splash);
        break;
    case GST_MESSAGE_STATE_CHANGED:
        if (GST_MESSAGE_SRC(message) != GST_OBJECT(splash->playbin)) {
            break;
        }
        gst_message_parse_state_changed(message, &old_state, &new_state, &pending);
        syslog(
            LOG_INFO,
            "boot splash state %s -> %s",
            gst_element_state_get_name(old_state),
            gst_element_state_get_name(new_state)
        );
        break;
    default:
        break;
    }
    return TRUE;
}

static GstElement *make_video_sink(GtkWidget **widget_out) {
    /* gtksink draws through GTK. gtkwaylandsink can preroll-hang on labwc. */
    const char *names[] = {"gtksink", "gtkwaylandsink", NULL};
    size_t i;

    *widget_out = NULL;
    for (i = 0; names[i] != NULL; i++) {
        GstElement *sink = gst_element_factory_make(names[i], "splash-video");
        GtkWidget *widget = NULL;

        if (sink == NULL) {
            continue;
        }
        if (g_object_class_find_property(G_OBJECT_GET_CLASS(sink), "widget") != NULL) {
            g_object_get(sink, "widget", &widget, NULL);
        }
        if (widget != NULL) {
            syslog(LOG_INFO, "boot splash video sink=%s", names[i]);
            *widget_out = widget;
            return sink;
        }
        gst_object_unref(sink);
    }
    return NULL;
}

static void splash_start(BootSplash *splash) {
    GstStateChangeReturn ret;

    if (splash == NULL || splash->started || splash->finished || splash->playbin == NULL) {
        return;
    }
    splash->started = TRUE;
    ret = gst_element_set_state(splash->playbin, GST_STATE_PLAYING);
    syslog(LOG_INFO, "boot splash started volume=0.25 ret=%d", (int)ret);
    if (ret == GST_STATE_CHANGE_FAILURE) {
        splash_finish(splash);
    }
}

static void on_host_mapped(GtkWidget *widget, gpointer user_data) {
    (void)widget;
    splash_start(user_data);
}

static gboolean on_try_start(gpointer user_data) {
    BootSplash *splash = user_data;

    if (splash->finished) {
        splash->start_watch = 0;
        return FALSE;
    }
    if (gtk_widget_get_realized(splash->video) || gtk_widget_get_mapped(splash->host)) {
        splash->start_watch = 0;
        splash_start(splash);
        return FALSE;
    }
    return TRUE;
}

static void on_host_destroy(GtkWidget *widget, gpointer user_data) {
    BootSplash *splash = user_data;

    (void)widget;
    splash_finish(splash);
    g_free(splash);
}

gboolean empiier_boot_splash_attach(GtkOverlay *overlay, EmpiierNativeBridge *bridge) {
    BootSplash *splash;
    GstElement *playbin;
    GstElement *audio;
    GstElement *video;
    GstBus *bus;
    GtkWidget *widget = NULL;
    char path[256];
    gchar *uri;
    GError *error = NULL;

    if (overlay == NULL) {
        return FALSE;
    }
    if (!resolve_splash(path, sizeof(path))) {
        syslog(LOG_INFO, "boot splash skipped missing %s", SPLASH_NAME);
        return FALSE;
    }
    if (!gst_init_check(NULL, NULL, &error)) {
        syslog(
            LOG_INFO,
            "boot splash gst init failed %s",
            error != NULL ? error->message : "unknown"
        );
        g_clear_error(&error);
        return FALSE;
    }
    playbin = gst_element_factory_make("playbin", "splash-playbin");
    audio = gst_element_factory_make("alsasink", "splash-alsa");
    video = make_video_sink(&widget);
    if (playbin == NULL || audio == NULL || video == NULL || widget == NULL) {
        syslog(LOG_INFO, "boot splash skipped playbin/alsa/video sink unavailable");
        if (playbin != NULL) {
            gst_object_unref(playbin);
        }
        if (audio != NULL) {
            gst_object_unref(audio);
        }
        if (video != NULL) {
            gst_object_unref(video);
        }
        return FALSE;
    }
    uri = g_filename_to_uri(path, NULL, NULL);
    if (uri == NULL) {
        gst_object_unref(playbin);
        gst_object_unref(audio);
        gst_object_unref(video);
        syslog(LOG_INFO, "boot splash uri failed path=%s", path);
        return FALSE;
    }
    splash = g_new0(BootSplash, 1);
    splash->playbin = playbin;
    splash->video = widget;
    splash->bridge = bridge;
    splash->host = gtk_event_box_new();
    gtk_widget_set_name(splash->host, "empiieros-boot-splash");
    gtk_widget_set_hexpand(splash->host, TRUE);
    gtk_widget_set_vexpand(splash->host, TRUE);
    gtk_widget_set_halign(splash->host, GTK_ALIGN_FILL);
    gtk_widget_set_valign(splash->host, GTK_ALIGN_FILL);
    splash_paint_black(splash->host);
    gtk_container_add(GTK_CONTAINER(splash->host), widget);
    gtk_overlay_add_overlay(overlay, splash->host);
    gtk_overlay_set_overlay_pass_through(overlay, splash->host, FALSE);
    gtk_widget_show_all(splash->host);
    if (bridge != NULL && bridge->product_bar != NULL) {
        gtk_widget_hide(bridge->product_bar);
    }
    /* VIDEO|AUDIO only. Soft-volume 0.25 on playbin, raw ALSA out. */
    g_object_set(
        playbin,
        "uri",
        uri,
        "volume",
        SPLASH_VOLUME,
        "flags",
        (1 << 0) | (1 << 1),
        "audio-sink",
        audio,
        "video-sink",
        video,
        NULL
    );
    g_free(uri);
    bus = gst_element_get_bus(playbin);
    splash->bus_watch = gst_bus_add_watch(bus, on_bus, splash);
    gst_object_unref(bus);
    splash->failsafe = g_timeout_add(SPLASH_FAILSAFE_MS, on_failsafe, splash);
    splash->start_watch = g_timeout_add(50, on_try_start, splash);
    g_signal_connect(splash->host, "map", G_CALLBACK(on_host_mapped), splash);
    g_signal_connect(splash->host, "destroy", G_CALLBACK(on_host_destroy), splash);
    syslog(LOG_INFO, "boot splash attached volume=0.25 path=%s", path);
    return TRUE;
}
