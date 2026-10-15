#ifndef EMPIIEROS_NATIVE_BRIDGE_H
#define EMPIIEROS_NATIVE_BRIDGE_H

#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

#define EMPIIEROS_SHELL_URI "empiier://shell/index.html"
#define EMPIIEROS_FOFLINE_SPA_START \
    "fofline-spa://localhost/index.html?eliteewFoflineClient=1"
#define EMPIIEROS_FOFLINE_SPA_PREFIX "fofline-spa://localhost/"
#define EMPIIEROS_OIDC_AUTH \
    "https://auth.eliteew.ru/realms/empery/protocol/openid-connect/auth"
#define EMPIIEROS_OIDC_CALLBACK \
    "https://app.eliteew.ru/fofline-oidc-bridge.html"
#define EMPIIEROS_BROWSER_TAB_CAP 3
/** Service gate: windowed mode and F11 toggle require this root-owned plant. */
#define EMPIIEROS_SHELL_WINDOW_MODE_ALLOW \
    "/var/lib/empiieros-runtime/service/shell-window-mode.allow"

typedef struct {
    WebKitWebView *web_view;
    GtkWindow *window;
    const char *trusted_uri_prefix;
    const char *socket_path;
    gboolean windowed;
    GtkWidget *overlay;
    GtkWidget *product_bar;
    GtkWidget *auth_box;
    GtkWidget *auth_dimmer;
    GtkWidget *auth_card;
    WebKitWebView *auth_view;
    WebKitUserContentManager *manager;
    WebKitUserScript *oidc_script;
    gboolean product_visible;
    gboolean auth_visible;
    guint8 auth_fresh_budget;
    gboolean auth_fresh_busy;
    GtkWidget *browser_box;
    WebKitWebView *browser_view;
    GtkWidget *browser_stack;
    WebKitWebView *browser_views[EMPIIEROS_BROWSER_TAB_CAP];
    guint32 browser_tab_ids[EMPIIEROS_BROWSER_TAB_CAP];
    GtkWidget *browser_tab_strip;
    GtkWidget *browser_tab_buttons[EMPIIEROS_BROWSER_TAB_CAP];
    GtkWidget *browser_tab_new;
    gboolean browser_visible;
    GtkWidget *browser_chrome;
    GtkWidget *browser_address;
    GtkWidget *browser_find;
    GtkWidget *browser_progress;
    GtkWidget *browser_zoom_label;
    GtkWidget *browser_back;
    GtkWidget *browser_forward;
    GtkWidget *meet_box;
    WebKitWebView *meet_view;
    gboolean meet_visible;
    gboolean meet_has_bounds;
    gboolean oidc_product_busy;
} EmpiierNativeBridge;

void empiier_native_bridge_init(
    EmpiierNativeBridge *bridge,
    WebKitUserContentManager *manager,
    WebKitWebView *web_view,
    GtkWindow *window,
    const char *trusted_uri_prefix,
    const char *socket_path
);

void empiier_native_bridge_attach_chrome(
    EmpiierNativeBridge *bridge,
    GtkWidget *product_bar,
    GtkWidget *auth_box,
    WebKitWebView *auth_view
);

gboolean empiier_shell_window_mode_service_allowed(void);

gboolean empiier_native_bridge_apply_window_mode(
    EmpiierNativeBridge *bridge,
    const char *mode
);

void empiier_native_bridge_publish_window_state(EmpiierNativeBridge *bridge);

gboolean empiier_native_bridge_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
);

gboolean empiier_native_bridge_show_product(
    EmpiierNativeBridge *bridge,
    const char *entrypoint
);

void empiier_native_bridge_hide_product(EmpiierNativeBridge *bridge);

gboolean empiier_native_bridge_show_auth(
    EmpiierNativeBridge *bridge,
    const char *entrypoint
);

void empiier_native_bridge_hide_auth(EmpiierNativeBridge *bridge);

gboolean empiier_native_bridge_show_seat_browser(
    EmpiierNativeBridge *bridge,
    const char *url
);

void empiier_native_bridge_hide_seat_browser(EmpiierNativeBridge *bridge);

void empiier_native_bridge_attach_seat_browser(
    EmpiierNativeBridge *bridge,
    GtkWidget *browser_box,
    WebKitWebView *browser_view
);

gboolean empiier_native_bridge_seat_browser_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
);

gboolean empiier_native_bridge_seat_browser_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
);

GtkWidget *empiier_seat_browser_chrome_build(EmpiierNativeBridge *bridge);

void empiier_seat_browser_chrome_sync(EmpiierNativeBridge *bridge);

gboolean empiier_native_bridge_seat_browser_command(
    EmpiierNativeBridge *bridge,
    const char *action,
    const char *arg
);

gboolean empiier_native_bridge_seat_browser_tab(
    EmpiierNativeBridge *bridge,
    const char *action,
    guint32 tab_id,
    const char *url
);

gboolean empiier_native_bridge_seat_browser_key(
    EmpiierNativeBridge *bridge,
    GdkEventKey *event
);

gboolean empiier_native_bridge_show_meet(
    EmpiierNativeBridge *bridge,
    const char *url
);

void empiier_native_bridge_hide_meet(EmpiierNativeBridge *bridge);

void empiier_native_bridge_conceal_meet(EmpiierNativeBridge *bridge);

void empiier_native_bridge_attach_meet(
    EmpiierNativeBridge *bridge,
    GtkWidget *meet_box,
    WebKitWebView *meet_view
);

void empiier_meet_register_product_bridge(EmpiierNativeBridge *bridge);

void empiier_seat_browser_register_product_bridge(EmpiierNativeBridge *bridge);

void empiier_meet_register_diag(WebKitUserContentManager *manager);

gboolean empiier_native_bridge_meet_host_ok(const char *uri);

gboolean empiier_native_bridge_product_meet_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
);

gboolean empiier_native_bridge_meet_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
);

gboolean empiier_native_bridge_meet_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
);

gboolean empiier_native_bridge_meet_load_failed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    const char *failing_uri,
    GError *error,
    gpointer user_data
);

gboolean empiier_native_bridge_escape_surface(EmpiierNativeBridge *bridge);

gboolean empiier_native_bridge_fofline_uri(const char *uri);

gboolean empiier_native_bridge_auth_url(const char *uri);

gboolean empiier_native_bridge_auth_callback(const char *uri);

WebKitUserScript *empiier_oidc_manual_login_script(void);

gboolean empiier_native_bridge_auth_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
);

char *empiier_native_json_escape(const char *input);

char *empiier_native_runtime_exchange(
    EmpiierNativeBridge *bridge,
    const char *request,
    GError **error
);

void empiier_native_dispatch_custom_event(
    EmpiierNativeBridge *bridge,
    const char *name,
    const char *detail_json
);

GtkWidget *empiier_product_chrome_new(EmpiierNativeBridge *bridge);

void empiier_widget_show_deferred(GtkWidget *widget);
void empiier_widget_hide_deferred(GtkWidget *widget);
void empiier_overlay_park(EmpiierNativeBridge *bridge, GtkWidget *box, GtkWidget *view);
void empiier_overlay_raise(EmpiierNativeBridge *bridge, GtkWidget *box, GtkWidget *view);

void empiier_product_chrome_show(EmpiierNativeBridge *bridge);

void empiier_product_chrome_hide(EmpiierNativeBridge *bridge);

gboolean empiier_oidc_inject_product(EmpiierNativeBridge *bridge);

void empiier_oidc_clear_product(EmpiierNativeBridge *bridge);

void empiier_oidc_register_product_bridge(EmpiierNativeBridge *bridge);
void empiier_network_register_product_bridge(EmpiierNativeBridge *bridge);

void empiier_oidc_complete_from_callback(
    EmpiierNativeBridge *bridge,
    const char *callback_url
);

void empiier_oidc_on_auth_submit_form(
    WebKitWebView *web_view,
    WebKitFormSubmissionRequest *request,
    gpointer user_data
);

void empiier_oidc_on_auth_page_finished(
    WebKitWebView *web_view,
    EmpiierNativeBridge *bridge
);

#endif
