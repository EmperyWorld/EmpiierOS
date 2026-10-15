#include "auth_bounds.h"

#include <gdk/gdk.h>
#include <syslog.h>

#define AUTH_CARD_W 480
#define AUTH_CARD_H 640
#define AUTH_CARD_MARGIN 32

static const char AUTH_CARD_CSS[] =
    "#empiieros-auth-dimmer {"
    "  background-color: rgba(6, 8, 14, 0.58);"
    "}"
    "#empiieros-auth-card {"
    "  background-color: #10141c;"
    "  border-radius: 10px;"
    "  border: 1px solid rgba(255, 255, 255, 0.12);"
    "}";

static void auth_card_install_css(void) {
    static gboolean installed = FALSE;
    GtkCssProvider *css;
    GdkScreen *screen;

    if (installed) {
        return;
    }
    screen = gdk_screen_get_default();
    if (screen == NULL) {
        return;
    }
    css = gtk_css_provider_new();
    gtk_css_provider_load_from_data(css, AUTH_CARD_CSS, -1, NULL);
    gtk_style_context_add_provider_for_screen(
        screen,
        GTK_STYLE_PROVIDER(css),
        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
    );
    g_object_unref(css);
    installed = TRUE;
}

static int auth_card_fit(int preferred, int host, int margin) {
    int inner;

    if (host <= 0) {
        return preferred;
    }
    inner = host - (2 * margin);
    if (inner < 1) {
        return 1;
    }
    if (preferred > inner) {
        return inner;
    }
    return preferred;
}

void empiier_auth_card_apply(EmpiierNativeBridge *bridge) {
    GtkAllocation host;
    int iw;
    int ih;
    gint cur_w;
    gint cur_h;

    if (bridge == NULL || bridge->auth_card == NULL || bridge->auth_view == NULL) {
        return;
    }
    host.width = 0;
    host.height = 0;
    if (bridge->overlay != NULL) {
        gtk_widget_get_allocation(bridge->overlay, &host);
    } else if (bridge->web_view != NULL) {
        gtk_widget_get_allocation(GTK_WIDGET(bridge->web_view), &host);
    }
    iw = auth_card_fit(AUTH_CARD_W, host.width, AUTH_CARD_MARGIN);
    ih = auth_card_fit(AUTH_CARD_H, host.height, AUTH_CARD_MARGIN);
    gtk_widget_get_size_request(bridge->auth_card, &cur_w, &cur_h);
    if (cur_w == iw && cur_h == ih) {
        return;
    }
    gtk_widget_set_halign(bridge->auth_card, GTK_ALIGN_CENTER);
    gtk_widget_set_valign(bridge->auth_card, GTK_ALIGN_CENTER);
    gtk_widget_set_hexpand(bridge->auth_card, FALSE);
    gtk_widget_set_vexpand(bridge->auth_card, FALSE);
    gtk_widget_set_size_request(bridge->auth_card, iw, ih);
    gtk_widget_set_size_request(GTK_WIDGET(bridge->auth_view), iw, ih);
}

static gboolean auth_dimmer_press(
    GtkWidget *widget,
    GdkEventButton *event,
    gpointer user_data
) {
    (void)widget;
    (void)event;
    (void)user_data;
    /* Swallow. Do not hide — PAR/OIDC stays until Escape/F9 or complete. */
    return TRUE;
}

static void auth_card_on_allocate(
    GtkWidget *widget,
    GdkRectangle *allocation,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;

    (void)widget;
    (void)allocation;
    if (bridge != NULL && bridge->auth_visible) {
        empiier_auth_card_apply(bridge);
    }
}

GtkWidget *empiier_auth_card_build(
    EmpiierNativeBridge *bridge,
    WebKitWebView *auth_view
) {
    GtkWidget *auth_box;
    GtkWidget *dimmer;
    GtkWidget *card;
    GtkWidget *view_widget;

    if (bridge == NULL || auth_view == NULL) {
        return NULL;
    }
    view_widget = GTK_WIDGET(auth_view);
    auth_card_install_css();

    auth_box = gtk_overlay_new();
    gtk_widget_set_no_show_all(auth_box, TRUE);
    gtk_widget_set_hexpand(auth_box, TRUE);
    gtk_widget_set_vexpand(auth_box, TRUE);

    dimmer = gtk_event_box_new();
    gtk_widget_set_name(dimmer, "empiieros-auth-dimmer");
    gtk_event_box_set_visible_window(GTK_EVENT_BOX(dimmer), TRUE);
    gtk_widget_set_hexpand(dimmer, TRUE);
    gtk_widget_set_vexpand(dimmer, TRUE);
    gtk_widget_add_events(dimmer, GDK_BUTTON_PRESS_MASK);
    g_signal_connect(dimmer, "button-press-event", G_CALLBACK(auth_dimmer_press), bridge);
    gtk_container_add(GTK_CONTAINER(auth_box), dimmer);

    card = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_set_name(card, "empiieros-auth-card");
    gtk_box_pack_start(GTK_BOX(card), view_widget, TRUE, TRUE, 0);
    gtk_overlay_add_overlay(GTK_OVERLAY(auth_box), card);
    gtk_widget_set_halign(card, GTK_ALIGN_CENTER);
    gtk_widget_set_valign(card, GTK_ALIGN_CENTER);
    gtk_widget_set_hexpand(card, FALSE);
    gtk_widget_set_vexpand(card, FALSE);

    bridge->auth_box = auth_box;
    bridge->auth_dimmer = dimmer;
    bridge->auth_card = card;
    bridge->auth_view = auth_view;
    empiier_auth_card_apply(bridge);
    g_signal_connect(auth_box, "size-allocate", G_CALLBACK(auth_card_on_allocate), bridge);
    syslog(LOG_INFO, "auth surface card wired kind=card");
    return auth_box;
}
