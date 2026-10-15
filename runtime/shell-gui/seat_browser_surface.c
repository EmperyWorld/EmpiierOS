#include "meet_bounds.h"
#include "native_bridge.h"

#include <gdk/gdkkeysyms.h>
#include <string.h>
#include <syslog.h>

#define BROWSER_ZOOM_MIN 0.5
#define BROWSER_ZOOM_MAX 3.0
#define BROWSER_ZOOM_STEP 0.1

#define BROWSER_CHROME_CSS \
    "#empiieros-seat-browser-chrome {" \
    "  background-color: rgba(17, 24, 39, 0.94);" \
    "  border-bottom: 1px solid rgba(56, 189, 248, 0.38);" \
    "  padding: 6px 8px;" \
    "}" \
    "#empiieros-seat-browser-chrome button {" \
    "  background-image: none;" \
    "  background-color: rgba(15, 23, 42, 0.72);" \
    "  border: 1px solid rgba(56, 189, 248, 0.28);" \
    "  border-radius: 8px;" \
    "  color: #e2e8f0;" \
    "  padding: 4px 10px;" \
    "  min-height: 28px;" \
    "}" \
    "#empiieros-seat-browser-chrome button:hover {" \
    "  background-color: rgba(56, 189, 248, 0.18);" \
    "}" \
    "#empiieros-seat-browser-chrome button:disabled {" \
    "  opacity: 0.45;" \
    "}" \
    "#empiieros-seat-browser-address, #empiieros-seat-browser-find {" \
    "  background-color: rgba(15, 23, 42, 0.92);" \
    "  color: #e2e8f0;" \
    "  border: 1px solid rgba(56, 189, 248, 0.28);" \
    "  border-radius: 8px;" \
    "  padding: 4px 8px;" \
    "  caret-color: #7dd3fc;" \
    "}" \
    "#empiieros-seat-browser-zoom {" \
    "  color: #94a3b8;" \
    "  min-width: 40px;" \
    "}" \
    "#empiieros-seat-browser-progress {" \
    "  min-height: 3px;" \
    "}" \
    "#empiieros-seat-browser-tabs {" \
    "  padding: 0 8px 4px 8px;" \
    "}" \
    "#empiieros-seat-browser-tabs button {" \
    "  min-width: 72px;" \
    "  max-width: 160px;" \
    "}" \
    "#empiieros-seat-browser-tabs button.tab-active {" \
    "  background-color: rgba(56, 189, 248, 0.28);" \
    "}"

static gboolean browser_url_scheme_ok(const char *uri) {
    if (uri == NULL) {
        return FALSE;
    }
    if (g_strcmp0(uri, "about:blank") == 0 || g_str_has_prefix(uri, "about:blank?")) {
        return TRUE;
    }
    return g_str_has_prefix(uri, "https://");
}

static gboolean runtime_evaluate_allowed(EmpiierNativeBridge *bridge, const char *uri) {
    if (bridge == NULL || uri == NULL) {
        return FALSE;
    }
    char *url_json = empiier_native_json_escape(uri);
    char *request = g_strdup_printf(
        "{\"id\":\"seat-browser-nav\",\"op\":\"seat.browser.evaluate\","
        "\"params\":{\"url\":%s}}",
        url_json
    );
    g_free(url_json);
    GError *error = NULL;
    char *response = empiier_native_runtime_exchange(bridge, request, &error);
    g_free(request);
    if (response == NULL) {
        if (error != NULL) {
            syslog(LOG_WARNING, "seat browser evaluate failed: %s", error->message);
            g_error_free(error);
        }
        return FALSE;
    }
    gboolean allowed = strstr(response, "\"allowed\":true") != NULL
        || strstr(response, "\"allowed\": true") != NULL;
    g_free(response);
    return allowed;
}

static gboolean json_bool_true(const char *json, const char *key) {
    char *needle_true;
    char *needle_spaced;
    gboolean hit;

    if (json == NULL || key == NULL) {
        return FALSE;
    }
    needle_true = g_strdup_printf("\"%s\":true", key);
    needle_spaced = g_strdup_printf("\"%s\": true", key);
    hit = strstr(json, needle_true) != NULL || strstr(json, needle_spaced) != NULL;
    g_free(needle_true);
    g_free(needle_spaced);
    return hit;
}

static gboolean runtime_media_policy(
    EmpiierNativeBridge *bridge,
    gboolean *camera_out,
    gboolean *microphone_out
) {
    GError *error = NULL;
    char *response;
    gboolean camera;
    gboolean microphone;

    if (bridge == NULL || camera_out == NULL || microphone_out == NULL) {
        return FALSE;
    }
    *camera_out = FALSE;
    *microphone_out = FALSE;
    response = empiier_native_runtime_exchange(
        bridge,
        "{\"id\":\"seat-browser-media\",\"op\":\"seat.browser.status\",\"params\":{}}",
        &error
    );
    if (response == NULL) {
        if (error != NULL) {
            syslog(LOG_WARNING, "seat browser media status failed: %s", error->message);
            g_error_free(error);
        }
        return FALSE;
    }
    camera = json_bool_true(response, "camera");
    microphone = json_bool_true(response, "microphone");
    g_free(response);
    *camera_out = camera;
    *microphone_out = microphone;
    return TRUE;
}

static guint32 json_u32_field(const char *json, const char *key) {
    char *needle;
    const char *hit;
    gchar *end = NULL;
    guint64 raw;

    if (json == NULL || key == NULL) {
        return 0;
    }
    needle = g_strdup_printf("\"%s\":", key);
    hit = strstr(json, needle);
    g_free(needle);
    if (hit == NULL) {
        return 0;
    }
    hit = strchr(hit, ':');
    if (hit == NULL) {
        return 0;
    }
    hit++;
    while (*hit == ' ' || *hit == '\t') {
        hit++;
    }
    if (*hit < '0' || *hit > '9') {
        return 0;
    }
    raw = g_ascii_strtoull(hit, &end, 10);
    if (end == hit || raw == 0 || raw > G_MAXUINT32) {
        return 0;
    }
    return (guint32)raw;
}

static char *runtime_exchange_ok(EmpiierNativeBridge *bridge, const char *request) {
    GError *error = NULL;
    char *response;

    if (bridge == NULL || request == NULL) {
        return NULL;
    }
    response = empiier_native_runtime_exchange(bridge, request, &error);
    if (response == NULL) {
        if (error != NULL) {
            syslog(LOG_WARNING, "seat browser runtime failed: %s", error->message);
            g_error_free(error);
        }
        return NULL;
    }
    if (!json_bool_true(response, "ok")) {
        syslog(LOG_INFO, "seat browser runtime denied request");
        g_free(response);
        return NULL;
    }
    return response;
}

static gboolean runtime_navigate(EmpiierNativeBridge *bridge, const char *url, guint32 *tab_id_out) {
    char *url_json;
    char *request;
    char *response;

    if (url == NULL) {
        return FALSE;
    }
    url_json = empiier_native_json_escape(url);
    request = g_strdup_printf(
        "{\"id\":\"seat-browser-nav-act\",\"op\":\"seat.browser.navigate\","
        "\"params\":{\"url\":%s}}",
        url_json
    );
    g_free(url_json);
    response = runtime_exchange_ok(bridge, request);
    g_free(request);
    if (response == NULL) {
        return FALSE;
    }
    if (tab_id_out != NULL) {
        *tab_id_out = json_u32_field(response, "tab_id");
    }
    g_free(response);
    return TRUE;
}

static gboolean runtime_tab_open(EmpiierNativeBridge *bridge, const char *url, guint32 *tab_id_out) {
    char *url_json;
    char *request;
    char *response;
    const char *use = (url != NULL && url[0] != '\0') ? url : "about:blank";

    url_json = empiier_native_json_escape(use);
    request = g_strdup_printf(
        "{\"id\":\"seat-browser-tab-open\",\"op\":\"seat.browser.tab.open\","
        "\"params\":{\"url\":%s}}",
        url_json
    );
    g_free(url_json);
    response = runtime_exchange_ok(bridge, request);
    g_free(request);
    if (response == NULL) {
        return FALSE;
    }
    if (tab_id_out != NULL) {
        *tab_id_out = json_u32_field(response, "tab_id");
    }
    g_free(response);
    return TRUE;
}

static gboolean runtime_tab_select(EmpiierNativeBridge *bridge, guint32 tab_id) {
    char *request;
    char *response;

    if (tab_id == 0) {
        return FALSE;
    }
    request = g_strdup_printf(
        "{\"id\":\"seat-browser-tab-select\",\"op\":\"seat.browser.tab.select\","
        "\"params\":{\"id\":%u}}",
        tab_id
    );
    response = runtime_exchange_ok(bridge, request);
    g_free(request);
    if (response == NULL) {
        return FALSE;
    }
    g_free(response);
    return TRUE;
}

static gboolean runtime_tab_close(
    EmpiierNativeBridge *bridge,
    guint32 tab_id,
    guint32 *remain_out,
    gboolean *hide_out
) {
    char *request;
    char *response;
    gboolean hide;

    if (tab_id == 0) {
        return FALSE;
    }
    request = g_strdup_printf(
        "{\"id\":\"seat-browser-tab-close\",\"op\":\"seat.browser.tab.close\","
        "\"params\":{\"id\":%u}}",
        tab_id
    );
    response = runtime_exchange_ok(bridge, request);
    g_free(request);
    if (response == NULL) {
        return FALSE;
    }
    hide = strstr(response, "\"host_action\":\"hide\"") != NULL
        || strstr(response, "\"host_action\": \"hide\"") != NULL;
    if (remain_out != NULL) {
        *remain_out = json_u32_field(response, "active_tab");
    }
    if (hide_out != NULL) {
        *hide_out = hide;
    }
    g_free(response);
    return TRUE;
}

static int browser_slot_for_id(const EmpiierNativeBridge *bridge, guint32 tab_id) {
    int i;

    if (bridge == NULL || tab_id == 0) {
        return -1;
    }
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        if (bridge->browser_tab_ids[i] == tab_id) {
            return i;
        }
    }
    return -1;
}

static int browser_alloc_slot(EmpiierNativeBridge *bridge, guint32 tab_id) {
    int existing;
    int i;

    if (bridge == NULL || tab_id == 0) {
        return -1;
    }
    existing = browser_slot_for_id(bridge, tab_id);
    if (existing >= 0) {
        return existing;
    }
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        if (bridge->browser_tab_ids[i] == 0 && bridge->browser_views[i] != NULL) {
            bridge->browser_tab_ids[i] = tab_id;
            return i;
        }
    }
    return -1;
}

static int browser_used_count(const EmpiierNativeBridge *bridge) {
    int i;
    int n = 0;

    if (bridge == NULL) {
        return 0;
    }
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        if (bridge->browser_tab_ids[i] != 0) {
            n++;
        }
    }
    return n;
}

static guint32 browser_active_tab_id(const EmpiierNativeBridge *bridge) {
    int i;

    if (bridge == NULL || bridge->browser_view == NULL) {
        return 0;
    }
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        if (bridge->browser_views[i] == bridge->browser_view) {
            return bridge->browser_tab_ids[i];
        }
    }
    return 0;
}

static GtkWidget *browser_input_widget(EmpiierNativeBridge *bridge) {
    if (bridge == NULL) {
        return NULL;
    }
    if (bridge->browser_stack != NULL) {
        return bridge->browser_stack;
    }
    return GTK_WIDGET(bridge->browser_view);
}

static void browser_activate_slot(EmpiierNativeBridge *bridge, int slot) {
    if (bridge == NULL || slot < 0 || slot >= EMPIIEROS_BROWSER_TAB_CAP) {
        return;
    }
    if (bridge->browser_views[slot] == NULL) {
        return;
    }
    bridge->browser_view = bridge->browser_views[slot];
    if (bridge->browser_stack != NULL) {
        gtk_stack_set_visible_child(
            GTK_STACK(bridge->browser_stack),
            GTK_WIDGET(bridge->browser_views[slot])
        );
    }
}

static void browser_clear_slot(EmpiierNativeBridge *bridge, int slot) {
    if (bridge == NULL || slot < 0 || slot >= EMPIIEROS_BROWSER_TAB_CAP) {
        return;
    }
    if (bridge->browser_views[slot] != NULL) {
        webkit_web_view_load_uri(bridge->browser_views[slot], "about:blank");
    }
    bridge->browser_tab_ids[slot] = 0;
}

static void tab_button_label(WebKitWebView *view, guint32 id, char *buf, size_t n) {
    const char *title;
    const char *uri;
    const char *host;
    const char *slash;

    if (buf == NULL || n == 0) {
        return;
    }
    title = view != NULL ? webkit_web_view_get_title(view) : NULL;
    if (title != NULL && title[0] != '\0') {
        g_snprintf(buf, n, "%s", title);
        return;
    }
    uri = view != NULL ? webkit_web_view_get_uri(view) : NULL;
    if (uri != NULL && g_str_has_prefix(uri, "https://")) {
        host = uri + 8;
        slash = strchr(host, '/');
        if (slash != NULL && slash > host) {
            g_snprintf(buf, n, "%.*s", (int)(slash - host), host);
            return;
        }
        g_snprintf(buf, n, "%s", host);
        return;
    }
    if (uri != NULL
        && (g_strcmp0(uri, "about:blank") == 0 || g_str_has_prefix(uri, "about:blank?"))) {
        g_snprintf(buf, n, "blank");
        return;
    }
    g_snprintf(buf, n, "Tab %u", id);
}

static void browser_sync_tab_strip(EmpiierNativeBridge *bridge) {
    int i;
    char label[48];
    GtkStyleContext *ctx;

    if (bridge == NULL) {
        return;
    }
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        GtkWidget *button = bridge->browser_tab_buttons[i];
        if (button == NULL) {
            continue;
        }
        if (bridge->browser_tab_ids[i] == 0 || bridge->browser_views[i] == NULL) {
            gtk_widget_hide(button);
            continue;
        }
        tab_button_label(bridge->browser_views[i], bridge->browser_tab_ids[i], label, sizeof(label));
        gtk_button_set_label(GTK_BUTTON(button), label);
        gtk_widget_set_no_show_all(button, FALSE);
        gtk_widget_show(button);
        ctx = gtk_widget_get_style_context(button);
        if (bridge->browser_views[i] == bridge->browser_view) {
            gtk_style_context_add_class(ctx, "tab-active");
        } else {
            gtk_style_context_remove_class(ctx, "tab-active");
        }
    }
    if (bridge->browser_tab_new != NULL) {
        gtk_widget_set_sensitive(
            bridge->browser_tab_new,
            browser_used_count(bridge) < EMPIIEROS_BROWSER_TAB_CAP
        );
    }
}

static gboolean browser_raise_surface(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->browser_box == NULL || bridge->browser_view == NULL) {
        return FALSE;
    }
    if (bridge->auth_visible) {
        empiier_native_bridge_hide_auth(bridge);
    }
    empiier_overlay_raise(bridge, bridge->browser_box, browser_input_widget(bridge));
    bridge->browser_visible = TRUE;
    empiier_seat_browser_chrome_sync(bridge);
    return TRUE;
}

static gboolean browser_use_tab(
    EmpiierNativeBridge *bridge,
    guint32 tab_id,
    const char *url,
    gboolean load
) {
    int slot;

    if (bridge == NULL) {
        return FALSE;
    }
    if (tab_id == 0) {
        tab_id = browser_active_tab_id(bridge);
        if (tab_id == 0) {
            tab_id = 1;
        }
    }
    slot = browser_alloc_slot(bridge, tab_id);
    if (slot < 0) {
        syslog(LOG_INFO, "seat browser tab slot denied id=%u", tab_id);
        return FALSE;
    }
    browser_activate_slot(bridge, slot);
    if (load && url != NULL && url[0] != '\0') {
        if (!browser_url_scheme_ok(url) || !runtime_evaluate_allowed(bridge, url)) {
            syslog(LOG_INFO, "seat browser tab load denied");
            return FALSE;
        }
        webkit_web_view_load_uri(bridge->browser_views[slot], url);
        if (bridge->browser_address != NULL) {
            gtk_entry_set_text(GTK_ENTRY(bridge->browser_address), url);
        }
    }
    return browser_raise_surface(bridge);
}

static char *normalize_address(const char *raw) {
    if (raw == NULL) {
        return NULL;
    }
    while (*raw == ' ' || *raw == '\t') {
        raw++;
    }
    if (*raw == '\0') {
        return NULL;
    }
    if (g_str_has_prefix(raw, "https://")
        || g_str_has_prefix(raw, "http://")
        || g_str_has_prefix(raw, "about:")) {
        return g_strdup(raw);
    }
    return g_strdup_printf("https://%s", raw);
}

static void apply_browser_chrome_css(void) {
    GtkCssProvider *provider = gtk_css_provider_new();
    GdkScreen *screen = gdk_screen_get_default();

    gtk_css_provider_load_from_data(provider, BROWSER_CHROME_CSS, -1, NULL);
    if (screen == NULL) {
        g_object_unref(provider);
        return;
    }
    gtk_style_context_add_provider_for_screen(
        screen,
        GTK_STYLE_PROVIDER(provider),
        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
    );
    g_object_unref(provider);
}

static void clamp_zoom(WebKitWebView *view, gdouble zoom) {
    if (zoom < BROWSER_ZOOM_MIN) {
        zoom = BROWSER_ZOOM_MIN;
    }
    if (zoom > BROWSER_ZOOM_MAX) {
        zoom = BROWSER_ZOOM_MAX;
    }
    webkit_web_view_set_zoom_level(view, zoom);
}

static void finish_find(EmpiierNativeBridge *bridge) {
    WebKitFindController *finder;

    if (bridge == NULL || bridge->browser_view == NULL) {
        return;
    }
    finder = webkit_web_view_get_find_controller(bridge->browser_view);
    webkit_find_controller_search_finish(finder);
    if (bridge->browser_find != NULL) {
        gtk_entry_set_text(GTK_ENTRY(bridge->browser_find), "");
    }
}

static void search_in_page(EmpiierNativeBridge *bridge, const char *query, gboolean next) {
    WebKitFindController *finder;
    const char *text;

    if (bridge == NULL || bridge->browser_view == NULL) {
        return;
    }
    finder = webkit_web_view_get_find_controller(bridge->browser_view);
    text = query;
    if ((text == NULL || text[0] == '\0') && bridge->browser_find != NULL) {
        text = gtk_entry_get_text(GTK_ENTRY(bridge->browser_find));
    }
    if (text == NULL || text[0] == '\0') {
        webkit_find_controller_search_finish(finder);
        return;
    }
    if (next && webkit_find_controller_get_search_text(finder) != NULL) {
        webkit_find_controller_search_next(finder);
        return;
    }
    webkit_find_controller_search(
        finder,
        text,
        WEBKIT_FIND_OPTIONS_CASE_INSENSITIVE | WEBKIT_FIND_OPTIONS_WRAP_AROUND,
        G_MAXUINT
    );
}

void empiier_seat_browser_chrome_sync(EmpiierNativeBridge *bridge) {
    const char *uri;
    gdouble progress;
    gdouble zoom;
    char zoom_label[16];
    gboolean loading;

    if (bridge == NULL || bridge->browser_view == NULL) {
        return;
    }
    uri = webkit_web_view_get_uri(bridge->browser_view);
    if (bridge->browser_address != NULL
        && !gtk_widget_has_focus(bridge->browser_address)
        && uri != NULL) {
        gtk_entry_set_text(GTK_ENTRY(bridge->browser_address), uri);
    }
    if (bridge->browser_back != NULL) {
        gtk_widget_set_sensitive(
            bridge->browser_back,
            webkit_web_view_can_go_back(bridge->browser_view)
        );
    }
    if (bridge->browser_forward != NULL) {
        gtk_widget_set_sensitive(
            bridge->browser_forward,
            webkit_web_view_can_go_forward(bridge->browser_view)
        );
    }
    progress = webkit_web_view_get_estimated_load_progress(bridge->browser_view);
    loading = webkit_web_view_is_loading(bridge->browser_view);
    if (bridge->browser_progress != NULL) {
        gtk_progress_bar_set_fraction(
            GTK_PROGRESS_BAR(bridge->browser_progress),
            loading ? progress : 0.0
        );
        if (loading && progress < 1.0) {
            gtk_widget_show(bridge->browser_progress);
        } else {
            gtk_widget_hide(bridge->browser_progress);
        }
    }
    zoom = webkit_web_view_get_zoom_level(bridge->browser_view);
    if (bridge->browser_zoom_label != NULL) {
        g_snprintf(zoom_label, sizeof(zoom_label), "%d%%", (int)(zoom * 100.0 + 0.5));
        gtk_label_set_text(GTK_LABEL(bridge->browser_zoom_label), zoom_label);
    }
    browser_sync_tab_strip(bridge);
}

gboolean empiier_native_bridge_seat_browser_command(
    EmpiierNativeBridge *bridge,
    const char *action,
    const char *arg
) {
    char *url;
    gdouble zoom;
    gchar *end = NULL;
    guint32 tab_id = 0;
    gboolean ok;

    if (bridge == NULL || bridge->browser_view == NULL || action == NULL) {
        return FALSE;
    }
    if (!bridge->browser_visible) {
        return FALSE;
    }
    if (g_strcmp0(action, "reload") == 0) {
        webkit_web_view_reload(bridge->browser_view);
        empiier_seat_browser_chrome_sync(bridge);
        syslog(LOG_INFO, "seat browser reload");
        return TRUE;
    }
    if (g_strcmp0(action, "stop") == 0) {
        webkit_web_view_stop_loading(bridge->browser_view);
        empiier_seat_browser_chrome_sync(bridge);
        return TRUE;
    }
    if (g_strcmp0(action, "back") == 0) {
        if (webkit_web_view_can_go_back(bridge->browser_view)) {
            webkit_web_view_go_back(bridge->browser_view);
        }
        empiier_seat_browser_chrome_sync(bridge);
        syslog(LOG_INFO, "seat browser history back");
        return TRUE;
    }
    if (g_strcmp0(action, "forward") == 0) {
        if (webkit_web_view_can_go_forward(bridge->browser_view)) {
            webkit_web_view_go_forward(bridge->browser_view);
        }
        empiier_seat_browser_chrome_sync(bridge);
        syslog(LOG_INFO, "seat browser history forward");
        return TRUE;
    }
    if (g_strcmp0(action, "find") == 0) {
        if (arg != NULL && bridge->browser_find != NULL) {
            gtk_entry_set_text(GTK_ENTRY(bridge->browser_find), arg);
        }
        search_in_page(bridge, arg, FALSE);
        if (bridge->browser_find != NULL) {
            gtk_widget_grab_focus(bridge->browser_find);
        }
        return TRUE;
    }
    if (g_strcmp0(action, "find_next") == 0) {
        search_in_page(bridge, arg, TRUE);
        return TRUE;
    }
    if (g_strcmp0(action, "find_prev") == 0) {
        if (bridge->browser_view != NULL) {
            webkit_find_controller_search_previous(
                webkit_web_view_get_find_controller(bridge->browser_view)
            );
        }
        return TRUE;
    }
    if (g_strcmp0(action, "zoom") == 0) {
        zoom = webkit_web_view_get_zoom_level(bridge->browser_view);
        if (arg == NULL || g_strcmp0(arg, "in") == 0) {
            clamp_zoom(bridge->browser_view, zoom + BROWSER_ZOOM_STEP);
        } else if (g_strcmp0(arg, "out") == 0) {
            clamp_zoom(bridge->browser_view, zoom - BROWSER_ZOOM_STEP);
        } else if (g_strcmp0(arg, "reset") == 0) {
            clamp_zoom(bridge->browser_view, 1.0);
        } else {
            zoom = g_ascii_strtod(arg, &end);
            if (end == arg) {
                return FALSE;
            }
            clamp_zoom(bridge->browser_view, zoom);
        }
        empiier_seat_browser_chrome_sync(bridge);
        return TRUE;
    }
    if (g_strcmp0(action, "go") == 0) {
        url = normalize_address(
            arg != NULL && arg[0] != '\0'
                ? arg
                : (bridge->browser_address != NULL
                    ? gtk_entry_get_text(GTK_ENTRY(bridge->browser_address))
                    : NULL)
        );
        if (url == NULL || !browser_url_scheme_ok(url)) {
            g_free(url);
            syslog(LOG_INFO, "seat browser go denied scheme");
            return FALSE;
        }
        if (!runtime_navigate(bridge, url, &tab_id)) {
            syslog(LOG_INFO, "seat browser go denied by policy");
            g_free(url);
            return FALSE;
        }
        ok = browser_use_tab(bridge, tab_id, url, TRUE);
        g_free(url);
        return ok;
    }
    return FALSE;
}

gboolean empiier_native_bridge_seat_browser_key(
    EmpiierNativeBridge *bridge,
    GdkEventKey *event
) {
    guint mods;
    const char *find_text;

    if (bridge == NULL || event == NULL || !bridge->browser_visible) {
        return FALSE;
    }
    mods = event->state & (GDK_CONTROL_MASK | GDK_MOD1_MASK | GDK_SHIFT_MASK);
    if (event->keyval == GDK_KEY_Escape) {
        find_text = bridge->browser_find != NULL
            ? gtk_entry_get_text(GTK_ENTRY(bridge->browser_find))
            : "";
        if (find_text != NULL && find_text[0] != '\0') {
            finish_find(bridge);
            return TRUE;
        }
        return FALSE;
    }
    if (event->keyval == GDK_KEY_F5
        || ((mods & GDK_CONTROL_MASK) != 0U
            && (event->keyval == GDK_KEY_r || event->keyval == GDK_KEY_R))) {
        return empiier_native_bridge_seat_browser_command(bridge, "reload", NULL);
    }
    if ((mods & GDK_MOD1_MASK) != 0U && event->keyval == GDK_KEY_Left) {
        return empiier_native_bridge_seat_browser_command(bridge, "back", NULL);
    }
    if ((mods & GDK_MOD1_MASK) != 0U && event->keyval == GDK_KEY_Right) {
        return empiier_native_bridge_seat_browser_command(bridge, "forward", NULL);
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_l || event->keyval == GDK_KEY_L)) {
        if (bridge->browser_address != NULL) {
            gtk_widget_grab_focus(bridge->browser_address);
            gtk_editable_select_region(GTK_EDITABLE(bridge->browser_address), 0, -1);
        }
        return TRUE;
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_f || event->keyval == GDK_KEY_F)) {
        return empiier_native_bridge_seat_browser_command(bridge, "find", NULL);
    }
    if (event->keyval == GDK_KEY_F3
        || ((mods & GDK_CONTROL_MASK) != 0U
            && (event->keyval == GDK_KEY_g || event->keyval == GDK_KEY_G))) {
        if ((mods & GDK_SHIFT_MASK) != 0U) {
            return empiier_native_bridge_seat_browser_command(bridge, "find_prev", NULL);
        }
        return empiier_native_bridge_seat_browser_command(bridge, "find_next", NULL);
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_plus
            || event->keyval == GDK_KEY_equal
            || event->keyval == GDK_KEY_KP_Add)) {
        return empiier_native_bridge_seat_browser_command(bridge, "zoom", "in");
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_minus || event->keyval == GDK_KEY_KP_Subtract)) {
        return empiier_native_bridge_seat_browser_command(bridge, "zoom", "out");
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_0 || event->keyval == GDK_KEY_KP_0)) {
        return empiier_native_bridge_seat_browser_command(bridge, "zoom", "reset");
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (mods & GDK_MOD1_MASK) == 0U
        && (event->keyval == GDK_KEY_t || event->keyval == GDK_KEY_T)) {
        return empiier_native_bridge_seat_browser_tab(bridge, "tab_open", 0, "about:blank");
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (mods & GDK_MOD1_MASK) == 0U
        && (event->keyval == GDK_KEY_w || event->keyval == GDK_KEY_W)) {
        return empiier_native_bridge_seat_browser_tab(
            bridge,
            "tab_close",
            browser_active_tab_id(bridge),
            NULL
        );
    }
    if ((mods & GDK_CONTROL_MASK) != 0U
        && (event->keyval == GDK_KEY_Tab
            || event->keyval == GDK_KEY_ISO_Left_Tab
            || event->keyval == GDK_KEY_KP_Tab)) {
        int i;
        int start = -1;
        int next = -1;
        int first = -1;
        int last = -1;
        int dir = ((mods & GDK_SHIFT_MASK) != 0U
            || event->keyval == GDK_KEY_ISO_Left_Tab)
            ? -1
            : 1;
        for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
            if (bridge->browser_tab_ids[i] == 0) {
                continue;
            }
            if (first < 0) {
                first = i;
            }
            last = i;
            if (bridge->browser_views[i] == bridge->browser_view) {
                start = i;
            }
        }
        if (first < 0) {
            return TRUE;
        }
        if (dir > 0) {
            for (i = start + 1; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
                if (bridge->browser_tab_ids[i] != 0) {
                    next = i;
                    break;
                }
            }
            if (next < 0) {
                next = first;
            }
        } else {
            for (i = start - 1; i >= 0; i--) {
                if (bridge->browser_tab_ids[i] != 0) {
                    next = i;
                    break;
                }
            }
            if (next < 0) {
                next = last;
            }
        }
        return empiier_native_bridge_seat_browser_tab(
            bridge,
            "tab_select",
            bridge->browser_tab_ids[next],
            NULL
        );
    }
    return FALSE;
}

static void on_chrome_command(GtkButton *button, gpointer user_data) {
    EmpiierNativeBridge *bridge = user_data;
    const char *action = g_object_get_data(G_OBJECT(button), "seat-browser-action");
    const char *arg = g_object_get_data(G_OBJECT(button), "seat-browser-arg");

    (void)empiier_native_bridge_seat_browser_command(bridge, action, arg);
}

static void on_chrome_hide(GtkButton *button, gpointer user_data) {
    (void)button;
    empiier_native_bridge_hide_seat_browser(user_data);
}

static void on_tab_strip_clicked(GtkButton *button, gpointer user_data) {
    EmpiierNativeBridge *bridge = user_data;
    int slot;
    guint32 tab_id;

    if (bridge == NULL || button == NULL) {
        return;
    }
    slot = GPOINTER_TO_INT(g_object_get_data(G_OBJECT(button), "tab-slot"));
    if (slot < 0 || slot >= EMPIIEROS_BROWSER_TAB_CAP) {
        return;
    }
    tab_id = bridge->browser_tab_ids[slot];
    if (tab_id == 0) {
        return;
    }
    (void)empiier_native_bridge_seat_browser_tab(bridge, "tab_select", tab_id, NULL);
}

static void on_tab_new_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    (void)empiier_native_bridge_seat_browser_tab(user_data, "tab_open", 0, "about:blank");
}

static void on_address_activate(GtkEntry *entry, gpointer user_data) {
    empiier_native_bridge_seat_browser_command(
        user_data,
        "go",
        gtk_entry_get_text(entry)
    );
}

static void on_find_changed(GtkEditable *editable, gpointer user_data) {
    EmpiierNativeBridge *bridge = user_data;

    search_in_page(bridge, gtk_entry_get_text(GTK_ENTRY(editable)), FALSE);
}

static void on_find_activate(GtkEntry *entry, gpointer user_data) {
    empiier_native_bridge_seat_browser_command(
        user_data,
        "find_next",
        gtk_entry_get_text(entry)
    );
}

static GtkWidget *chrome_button(
    const char *label,
    const char *tooltip,
    const char *action,
    const char *arg,
    EmpiierNativeBridge *bridge
) {
    GtkWidget *button = gtk_button_new_with_label(label);

    gtk_widget_set_tooltip_text(button, tooltip);
    gtk_button_set_relief(GTK_BUTTON(button), GTK_RELIEF_NONE);
    g_object_set_data(G_OBJECT(button), "seat-browser-action", (gpointer)action);
    g_object_set_data(G_OBJECT(button), "seat-browser-arg", (gpointer)arg);
    g_signal_connect(button, "clicked", G_CALLBACK(on_chrome_command), bridge);
    return button;
}

static void on_browser_load_changed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    gpointer user_data
) {
    (void)web_view;
    (void)load_event;
    empiier_seat_browser_chrome_sync(user_data);
}

static void on_browser_notify(GObject *object, GParamSpec *pspec, gpointer user_data) {
    (void)object;
    (void)pspec;
    empiier_seat_browser_chrome_sync(user_data);
}

static char *json_string_field(const char *json, const char *key) {
    char *needle;
    const char *hit;
    const char *start;
    const char *end;
    char *out;

    if (json == NULL || key == NULL) {
        return NULL;
    }
    needle = g_strdup_printf("\"%s\":\"", key);
    hit = strstr(json, needle);
    g_free(needle);
    if (hit == NULL) {
        needle = g_strdup_printf("\"%s\": \"", key);
        hit = strstr(json, needle);
        g_free(needle);
    }
    if (hit == NULL) {
        return NULL;
    }
    start = strchr(hit + 1, '"');
    if (start == NULL) {
        return NULL;
    }
    start = strchr(start + 1, '"');
    if (start == NULL) {
        return NULL;
    }
    start++;
    end = strchr(start, '"');
    if (end == NULL || end <= start) {
        return NULL;
    }
    out = g_strndup(start, (gsize)(end - start));
    return out;
}

static gboolean dest_uri_ok(const char *uri, const char *name) {
    if (uri == NULL || name == NULL || name[0] == '\0') {
        return FALSE;
    }
    if (!g_str_has_prefix(uri, "file://")) {
        return FALSE;
    }
    if (strstr(uri, "..") != NULL) {
        return FALSE;
    }
    if (strstr(uri, "/seat_browser/downloads/") == NULL) {
        return FALSE;
    }
    return g_str_has_suffix(uri, name);
}

static void runtime_download_finish(
    EmpiierNativeBridge *bridge,
    guint32 download_id,
    gboolean ok,
    guint64 bytes
) {
    char *request;
    char *response;

    if (bridge == NULL || download_id == 0) {
        return;
    }
    request = g_strdup_printf(
        "{\"id\":\"seat-browser-dl-fin\",\"op\":\"seat.browser.download.finish\","
        "\"params\":{\"id\":%u,\"ok\":%s,\"bytes\":%" G_GUINT64_FORMAT "}}",
        download_id,
        ok ? "true" : "false",
        bytes
    );
    response = runtime_exchange_ok(bridge, request);
    g_free(request);
    g_free(response);
}

static void on_download_finished(WebKitDownload *download, gpointer user_data) {
    guint32 download_id;
    guint64 bytes;

    if (download == NULL) {
        return;
    }
    download_id = GPOINTER_TO_UINT(g_object_get_data(G_OBJECT(download), "seat-dl-id"));
    bytes = webkit_download_get_received_data_length(download);
    runtime_download_finish(user_data, download_id, TRUE, bytes);
    syslog(LOG_INFO, "seat browser download finished id=%u bytes=%" G_GUINT64_FORMAT, download_id, bytes);
}

static gboolean on_download_failed(
    WebKitDownload *download,
    GError *error,
    gpointer user_data
) {
    guint32 download_id;
    guint64 bytes;

    download_id = download != NULL
        ? GPOINTER_TO_UINT(g_object_get_data(G_OBJECT(download), "seat-dl-id"))
        : 0;
    bytes = download != NULL ? webkit_download_get_received_data_length(download) : 0;
    runtime_download_finish(user_data, download_id, FALSE, bytes);
    syslog(
        LOG_INFO,
        "seat browser download failed id=%u err=%s",
        download_id,
        error != NULL ? error->message : ""
    );
    return TRUE;
}

static void on_download_received(WebKitDownload *download, guint64 data_length, gpointer user_data) {
    guint max_bytes;
    guint64 received;

    (void)data_length;
    (void)user_data;
    if (download == NULL) {
        return;
    }
    max_bytes = GPOINTER_TO_UINT(g_object_get_data(G_OBJECT(download), "seat-dl-max"));
    received = webkit_download_get_received_data_length(download);
    if (max_bytes > 0 && received > (guint64)max_bytes) {
        syslog(LOG_INFO, "seat browser download over cap bytes=%" G_GUINT64_FORMAT, received);
        webkit_download_cancel(download);
    }
}

static void on_browser_download_started(
    WebKitWebContext *context,
    WebKitDownload *download,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    WebKitURIRequest *request;
    const char *uri;
    char *suggested = NULL;
    char *name_json;
    char *url_json;
    char *rpc;
    char *response;
    char *dest_uri;
    char *dest_name;
    guint32 download_id;
    guint32 max_bytes;

    (void)context;
    if (bridge == NULL || download == NULL) {
        if (download != NULL) {
            webkit_download_cancel(download);
        }
        return;
    }
    request = webkit_download_get_request(download);
    uri = request != NULL ? webkit_uri_request_get_uri(request) : NULL;
    if (uri == NULL || !g_str_has_prefix(uri, "https://")) {
        syslog(LOG_INFO, "seat browser download denied scheme");
        webkit_download_cancel(download);
        return;
    }
    g_object_get(G_OBJECT(download), "suggested-filename", &suggested, NULL);
    if (suggested == NULL || suggested[0] == '\0') {
        g_free(suggested);
        suggested = g_path_get_basename(uri);
    }
    url_json = empiier_native_json_escape(uri);
    name_json = empiier_native_json_escape(suggested != NULL ? suggested : "download.bin");
    rpc = g_strdup_printf(
        "{\"id\":\"seat-browser-dl-begin\",\"op\":\"seat.browser.download.begin\","
        "\"params\":{\"url\":%s,\"name\":%s}}",
        url_json,
        name_json
    );
    g_free(url_json);
    g_free(name_json);
    g_free(suggested);
    response = runtime_exchange_ok(bridge, rpc);
    g_free(rpc);
    if (response == NULL) {
        syslog(LOG_INFO, "seat browser download denied by policy");
        webkit_download_cancel(download);
        return;
    }
    dest_uri = json_string_field(response, "dest_uri");
    dest_name = json_string_field(response, "name");
    download_id = json_u32_field(response, "download_id");
    max_bytes = json_u32_field(response, "max_bytes");
    g_free(response);
    if (download_id == 0 || dest_uri == NULL || !dest_uri_ok(dest_uri, dest_name)) {
        syslog(LOG_INFO, "seat browser download dest rejected");
        g_free(dest_uri);
        g_free(dest_name);
        webkit_download_cancel(download);
        return;
    }
    webkit_download_set_destination(download, dest_uri);
    g_object_set_data(G_OBJECT(download), "seat-dl-id", GUINT_TO_POINTER(download_id));
    g_object_set_data(G_OBJECT(download), "seat-dl-max", GUINT_TO_POINTER(max_bytes));
    g_signal_connect(download, "finished", G_CALLBACK(on_download_finished), bridge);
    g_signal_connect(download, "failed", G_CALLBACK(on_download_failed), bridge);
    g_signal_connect(download, "received-data", G_CALLBACK(on_download_received), bridge);
    syslog(LOG_INFO, "seat browser download started id=%u name=%s", download_id, dest_name);
    g_free(dest_uri);
    g_free(dest_name);
}

GtkWidget *empiier_seat_browser_chrome_build(EmpiierNativeBridge *bridge) {
    GtkWidget *wrap;
    GtkWidget *row;
    GtkWidget *tabs;
    GtkWidget *address;
    GtkWidget *find;
    GtkWidget *progress;
    GtkWidget *zoom;
    GtkWidget *hide;
    int i;

    apply_browser_chrome_css();
    wrap = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    tabs = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
    gtk_widget_set_name(wrap, "empiieros-seat-browser-chrome");
    gtk_widget_set_name(tabs, "empiieros-seat-browser-tabs");
    gtk_widget_set_hexpand(row, TRUE);
    address = gtk_entry_new();
    gtk_widget_set_name(address, "empiieros-seat-browser-address");
    gtk_widget_set_hexpand(address, TRUE);
    gtk_entry_set_placeholder_text(GTK_ENTRY(address), "https://");
    gtk_widget_set_tooltip_text(address, "Адрес (Ctrl+L)");
    find = gtk_entry_new();
    gtk_widget_set_name(find, "empiieros-seat-browser-find");
    gtk_entry_set_width_chars(GTK_ENTRY(find), 16);
    gtk_entry_set_placeholder_text(GTK_ENTRY(find), "Найти");
    gtk_widget_set_tooltip_text(find, "Найти на странице (Ctrl+F)");
    zoom = gtk_label_new("100%");
    gtk_widget_set_name(zoom, "empiieros-seat-browser-zoom");
    gtk_widget_set_tooltip_text(zoom, "Масштаб");
    progress = gtk_progress_bar_new();
    gtk_widget_set_name(progress, "empiieros-seat-browser-progress");
    gtk_widget_set_no_show_all(progress, TRUE);
    hide = gtk_button_new_with_label("Esc");
    gtk_widget_set_tooltip_text(hide, "Скрыть оверлей");
    gtk_button_set_relief(GTK_BUTTON(hide), GTK_RELIEF_NONE);
    if (bridge != NULL) {
        bridge->browser_chrome = wrap;
        bridge->browser_address = address;
        bridge->browser_find = find;
        bridge->browser_progress = progress;
        bridge->browser_zoom_label = zoom;
        bridge->browser_back = chrome_button("←", "Назад (Alt+Left)", "back", NULL, bridge);
        bridge->browser_forward = chrome_button("→", "Вперёд (Alt+Right)", "forward", NULL, bridge);
        gtk_widget_set_sensitive(bridge->browser_back, FALSE);
        gtk_widget_set_sensitive(bridge->browser_forward, FALSE);
        gtk_box_pack_start(GTK_BOX(row), bridge->browser_back, FALSE, FALSE, 0);
        gtk_box_pack_start(GTK_BOX(row), bridge->browser_forward, FALSE, FALSE, 0);
        gtk_box_pack_start(
            GTK_BOX(row),
            chrome_button("↻", "Обновить (F5)", "reload", NULL, bridge),
            FALSE,
            FALSE,
            0
        );
        gtk_box_pack_start(
            GTK_BOX(row),
            chrome_button("■", "Стоп", "stop", NULL, bridge),
            FALSE,
            FALSE,
            0
        );
        g_signal_connect(address, "activate", G_CALLBACK(on_address_activate), bridge);
        g_signal_connect(find, "changed", G_CALLBACK(on_find_changed), bridge);
        g_signal_connect(find, "activate", G_CALLBACK(on_find_activate), bridge);
        g_signal_connect(hide, "clicked", G_CALLBACK(on_chrome_hide), bridge);
        bridge->browser_tab_strip = tabs;
        for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
            GtkWidget *tab_btn = gtk_button_new_with_label("");
            gtk_button_set_relief(GTK_BUTTON(tab_btn), GTK_RELIEF_NONE);
            gtk_widget_set_no_show_all(tab_btn, TRUE);
            gtk_widget_hide(tab_btn);
            g_object_set_data(G_OBJECT(tab_btn), "tab-slot", GINT_TO_POINTER(i));
            g_signal_connect(tab_btn, "clicked", G_CALLBACK(on_tab_strip_clicked), bridge);
            bridge->browser_tab_buttons[i] = tab_btn;
            gtk_box_pack_start(GTK_BOX(tabs), tab_btn, FALSE, FALSE, 0);
        }
        bridge->browser_tab_new = gtk_button_new_with_label("+");
        gtk_widget_set_tooltip_text(bridge->browser_tab_new, "Новая вкладка (Ctrl+T)");
        gtk_button_set_relief(GTK_BUTTON(bridge->browser_tab_new), GTK_RELIEF_NONE);
        g_signal_connect(bridge->browser_tab_new, "clicked", G_CALLBACK(on_tab_new_clicked), bridge);
        gtk_box_pack_start(GTK_BOX(tabs), bridge->browser_tab_new, FALSE, FALSE, 0);
    }
    gtk_box_pack_start(GTK_BOX(row), address, TRUE, TRUE, 0);
    gtk_box_pack_start(
        GTK_BOX(row),
        chrome_button("→|", "Перейти", "go", NULL, bridge),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(
        GTK_BOX(row),
        chrome_button("−", "Мельче (Ctrl+-)", "zoom", "out", bridge),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(GTK_BOX(row), zoom, FALSE, FALSE, 0);
    gtk_box_pack_start(
        GTK_BOX(row),
        chrome_button("+", "Крупнее (Ctrl++)", "zoom", "in", bridge),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(GTK_BOX(row), find, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(row), hide, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(wrap), tabs, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(wrap), row, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(wrap), progress, FALSE, FALSE, 0);
    gtk_widget_show_all(wrap);
    syslog(LOG_INFO, "seat browser chrome attached");
    return wrap;
}

gboolean empiier_native_bridge_show_seat_browser(
    EmpiierNativeBridge *bridge,
    const char *url
) {
    return empiier_native_bridge_seat_browser_tab(bridge, "load", 0, url);
}

void empiier_native_bridge_hide_seat_browser(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->browser_box == NULL) {
        return;
    }
    finish_find(bridge);
    empiier_overlay_park(bridge, bridge->browser_box, browser_input_widget(bridge));
    bridge->browser_visible = FALSE;
    syslog(LOG_INFO, "seat browser hidden");
}

gboolean empiier_native_bridge_seat_browser_tab(
    EmpiierNativeBridge *bridge,
    const char *action,
    guint32 tab_id,
    const char *url
) {
    guint32 remain = 0;
    gboolean hide = FALSE;
    int slot;

    if (bridge == NULL || action == NULL) {
        return FALSE;
    }
    if (g_strcmp0(action, "tab_open") == 0) {
        if (tab_id == 0) {
            if (!runtime_tab_open(bridge, url, &tab_id) || tab_id == 0) {
                return FALSE;
            }
        }
        if (!browser_use_tab(bridge, tab_id, url != NULL ? url : "about:blank", TRUE)) {
            return FALSE;
        }
        syslog(LOG_INFO, "seat browser tab opened id=%u", tab_id);
        return TRUE;
    }
    if (g_strcmp0(action, "tab_select") == 0) {
        if (tab_id == 0) {
            return FALSE;
        }
        if (!runtime_tab_select(bridge, tab_id)) {
            return FALSE;
        }
        if (!browser_use_tab(bridge, tab_id, url, url != NULL && url[0] != '\0')) {
            return FALSE;
        }
        syslog(LOG_INFO, "seat browser tab selected id=%u", tab_id);
        return TRUE;
    }
    if (g_strcmp0(action, "tab_close") == 0) {
        if (tab_id == 0) {
            tab_id = browser_active_tab_id(bridge);
        }
        if (tab_id == 0) {
            return FALSE;
        }
        if (!runtime_tab_close(bridge, tab_id, &remain, &hide)) {
            /* Shell may have already closed the Runtime row. */
            slot = browser_slot_for_id(bridge, tab_id);
            if (slot < 0) {
                return FALSE;
            }
            hide = browser_used_count(bridge) <= 1;
        }
        slot = browser_slot_for_id(bridge, tab_id);
        browser_clear_slot(bridge, slot);
        if (hide || browser_used_count(bridge) == 0) {
            empiier_native_bridge_hide_seat_browser(bridge);
            syslog(LOG_INFO, "seat browser last tab closed id=%u", tab_id);
            return TRUE;
        }
        if (remain != 0) {
            (void)browser_use_tab(bridge, remain, NULL, FALSE);
        }
        empiier_seat_browser_chrome_sync(bridge);
        syslog(LOG_INFO, "seat browser tab closed id=%u", tab_id);
        return TRUE;
    }
    if (g_strcmp0(action, "load") == 0) {
        if (url == NULL || !browser_url_scheme_ok(url)) {
            return FALSE;
        }
        return browser_use_tab(bridge, tab_id, url, TRUE);
    }
    return FALSE;
}

gboolean empiier_native_bridge_seat_browser_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    (void)web_view;
    if (type != WEBKIT_POLICY_DECISION_TYPE_NAVIGATION_ACTION
        && type != WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        return FALSE;
    }
    WebKitNavigationPolicyDecision *nav = WEBKIT_NAVIGATION_POLICY_DECISION(decision);
    WebKitNavigationAction *action = webkit_navigation_policy_decision_get_navigation_action(nav);
    WebKitURIRequest *request = webkit_navigation_action_get_request(action);
    const char *uri = webkit_uri_request_get_uri(request);
    if (!browser_url_scheme_ok(uri) || !runtime_evaluate_allowed(bridge, uri)) {
        webkit_policy_decision_ignore(decision);
        syslog(LOG_INFO, "seat browser navigation denied uri=%s", uri != NULL ? uri : "");
        return TRUE;
    }
    if (type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        webkit_policy_decision_ignore(decision);
        if (uri != NULL
            && empiier_native_bridge_seat_browser_tab(bridge, "tab_open", 0, uri)) {
            return TRUE;
        }
        syslog(LOG_INFO, "seat browser extra window denied (tab cap or policy)");
        return TRUE;
    }
    if (uri != NULL) {
        (void)runtime_navigate(bridge, uri, NULL);
    }
    webkit_policy_decision_use(decision);
    return TRUE;
}

gboolean empiier_native_bridge_seat_browser_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    const char *uri;
    gboolean camera = FALSE;
    gboolean microphone = FALSE;
    gboolean want_video = FALSE;
    gboolean want_audio = FALSE;
    gboolean allowed;

    uri = webkit_web_view_get_uri(web_view);
    if (!browser_url_scheme_ok(uri) || !runtime_media_policy(bridge, &camera, &microphone)) {
        webkit_permission_request_deny(request);
        return TRUE;
    }
    if (WEBKIT_IS_DEVICE_INFO_PERMISSION_REQUEST(request)) {
        if (camera || microphone) {
            webkit_permission_request_allow(request);
        } else {
            webkit_permission_request_deny(request);
        }
        return TRUE;
    }
    if (!WEBKIT_IS_USER_MEDIA_PERMISSION_REQUEST(request)) {
        webkit_permission_request_deny(request);
        return TRUE;
    }
    if (webkit_user_media_permission_is_for_display_device(
            WEBKIT_USER_MEDIA_PERMISSION_REQUEST(request)
        )) {
        webkit_permission_request_deny(request);
        syslog(LOG_INFO, "seat browser display capture denied");
        return TRUE;
    }
    want_video = webkit_user_media_permission_is_for_video_device(
        WEBKIT_USER_MEDIA_PERMISSION_REQUEST(request)
    );
    want_audio = webkit_user_media_permission_is_for_audio_device(
        WEBKIT_USER_MEDIA_PERMISSION_REQUEST(request)
    );
    allowed = (!want_video || camera) && (!want_audio || microphone);
    if (allowed) {
        webkit_permission_request_allow(request);
        syslog(LOG_INFO, "seat browser media permission allowed video=%d audio=%d", want_video, want_audio);
    } else {
        webkit_permission_request_deny(request);
        syslog(LOG_INFO, "seat browser media permission denied video=%d audio=%d", want_video, want_audio);
    }
    return TRUE;
}

void empiier_native_bridge_attach_seat_browser(
    EmpiierNativeBridge *bridge,
    GtkWidget *browser_box,
    WebKitWebView *browser_view
) {
    WebKitWebContext *context;
    WebKitSettings *settings;
    int i;
    char name[8];

    if (bridge == NULL) {
        return;
    }
    bridge->browser_box = browser_box;
    bridge->browser_visible = FALSE;
    bridge->browser_stack = NULL;
    memset(bridge->browser_views, 0, sizeof(bridge->browser_views));
    memset(bridge->browser_tab_ids, 0, sizeof(bridge->browser_tab_ids));
    if (browser_view == NULL) {
        return;
    }
    settings = webkit_web_view_get_settings(browser_view);
    context = webkit_web_view_get_context(browser_view);
    bridge->browser_stack = gtk_stack_new();
    gtk_stack_set_transition_type(GTK_STACK(bridge->browser_stack), GTK_STACK_TRANSITION_TYPE_NONE);
    gtk_widget_set_hexpand(bridge->browser_stack, TRUE);
    gtk_widget_set_vexpand(bridge->browser_stack, TRUE);
    for (i = 0; i < EMPIIEROS_BROWSER_TAB_CAP; i++) {
        WebKitWebView *view;
        if (i == 0) {
            view = browser_view;
        } else {
            view = WEBKIT_WEB_VIEW(g_object_new(
                WEBKIT_TYPE_WEB_VIEW,
                "related-view",
                browser_view,
                "settings",
                settings,
                NULL
            ));
        }
        gtk_widget_set_hexpand(GTK_WIDGET(view), TRUE);
        gtk_widget_set_vexpand(GTK_WIDGET(view), TRUE);
        g_snprintf(name, sizeof(name), "tab%d", i);
        gtk_stack_add_named(GTK_STACK(bridge->browser_stack), GTK_WIDGET(view), name);
        g_signal_connect(view, "load-changed", G_CALLBACK(on_browser_load_changed), bridge);
        g_signal_connect(view, "notify::uri", G_CALLBACK(on_browser_notify), bridge);
        g_signal_connect(view, "notify::title", G_CALLBACK(on_browser_notify), bridge);
        g_signal_connect(
            view,
            "notify::estimated-load-progress",
            G_CALLBACK(on_browser_notify),
            bridge
        );
        g_signal_connect(
            view,
            "decide-policy",
            G_CALLBACK(empiier_native_bridge_seat_browser_decide_policy),
            bridge
        );
        g_signal_connect(
            view,
            "permission-request",
            G_CALLBACK(empiier_native_bridge_seat_browser_permission),
            bridge
        );
        gtk_widget_show(GTK_WIDGET(view));
        bridge->browser_views[i] = view;
        bridge->browser_tab_ids[i] = 0;
    }
    bridge->browser_view = bridge->browser_views[0];
    if (browser_box != NULL) {
        gtk_box_pack_start(GTK_BOX(browser_box), bridge->browser_stack, TRUE, TRUE, 0);
    }
    if (context != NULL) {
        g_signal_connect(
            context,
            "download-started",
            G_CALLBACK(on_browser_download_started),
            bridge
        );
    }
}

#define SEAT_BROWSER_HOST_URL_MAX 2048

static char seat_browser_reply_id[16];

static void seat_browser_host_resolve(EmpiierNativeBridge *bridge, const char *json) {
    char *script;

    if (bridge == NULL || bridge->web_view == NULL || json == NULL) {
        return;
    }
    if (seat_browser_reply_id[0] != '\0') {
        script = g_strdup_printf(
            "if(window.__empiierosSeatBrowserDone)window.__empiierosSeatBrowserDone("
            "Object.assign(%s,{id:'%s'}));",
            json,
            seat_browser_reply_id
        );
    } else {
        script = g_strdup_printf(
            "if(window.__empiierosSeatBrowserDone)window.__empiierosSeatBrowserDone(%s);",
            json
        );
    }
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-seat-browser-host",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
}

static gboolean seat_browser_host_show_existing(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->browser_box == NULL) {
        return FALSE;
    }
    if (browser_used_count(bridge) == 0) {
        return empiier_native_bridge_seat_browser_tab(bridge, "tab_open", 0, "about:blank");
    }
    empiier_overlay_raise(bridge, bridge->browser_box, browser_input_widget(bridge));
    bridge->browser_visible = TRUE;
    empiier_seat_browser_chrome_sync(bridge);
    return TRUE;
}

static const char *seat_browser_host_deny_code(const char *response) {
    if (response == NULL) {
        return "denied";
    }
    if (strstr(response, "allowlist_miss") != NULL) {
        return "allowlist_miss";
    }
    if (strstr(response, "\"scheme\"") != NULL || strstr(response, "scheme") != NULL) {
        return "scheme";
    }
    if (strstr(response, "tab_limit") != NULL) {
        return "tab_limit";
    }
    return "denied";
}

static void on_seat_browser_host_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *js_result,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    JSCValue *value;
    char *text;
    char url[SEAT_BROWSER_HOST_URL_MAX];
    char *request;
    char *url_json;
    char *response;
    char reply[96];

    (void)manager;
    if (bridge == NULL || bridge->web_view == NULL) {
        return;
    }
    if (!empiier_native_bridge_fofline_uri(webkit_web_view_get_uri(bridge->web_view))) {
        syslog(LOG_ERR, "fofline seat browser host denied origin");
        return;
    }
    value = webkit_javascript_result_get_js_value(js_result);
    text = value != NULL && jsc_value_is_string(value) ? jsc_value_to_string(value) : NULL;
    if (text == NULL) {
        seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"invalid_params\"}");
        return;
    }
    seat_browser_reply_id[0] = '\0';
    (void)empiier_meet_json_id(text, seat_browser_reply_id, sizeof seat_browser_reply_id);
    if (strstr(text, "accessToken") != NULL
        || strstr(text, "refreshToken") != NULL
        || strstr(text, "\"token\"") != NULL) {
        syslog(LOG_ERR, "fofline seat browser host denied code=token_forbidden");
        seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"token_forbidden\"}");
        g_free(text);
        return;
    }
    if (empiier_meet_is_op(text, "close")) {
        syslog(LOG_INFO, "fofline seat browser host request kind=close");
        empiier_native_bridge_hide_seat_browser(bridge);
        seat_browser_host_resolve(bridge, "{\"ok\":true}");
        g_free(text);
        return;
    }
    if (!empiier_meet_is_op(text, "open")) {
        syslog(LOG_ERR, "fofline seat browser host denied code=invalid_params");
        seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"invalid_params\"}");
        g_free(text);
        return;
    }
    syslog(LOG_INFO, "fofline seat browser host request kind=open");
    if (empiier_meet_json_url(text, url, sizeof(url))) {
        if (!g_str_has_prefix(url, "https://")
            || strchr(url, ' ') != NULL
            || strchr(url, '\n') != NULL
            || strchr(url, '\r') != NULL) {
            syslog(LOG_ERR, "fofline seat browser host denied code=scheme");
            seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"scheme\"}");
            g_free(text);
            return;
        }
        url_json = empiier_native_json_escape(url);
        request = g_strdup_printf(
            "{\"id\":\"seat-browser-host-nav\",\"op\":\"seat.browser.navigate\","
            "\"params\":{\"url\":%s}}",
            url_json
        );
        g_free(url_json);
        response = empiier_native_runtime_exchange(bridge, request, NULL);
        g_free(request);
        if (response == NULL || !json_bool_true(response, "ok")) {
            g_snprintf(
                reply,
                sizeof(reply),
                "{\"ok\":false,\"code\":\"%s\"}",
                seat_browser_host_deny_code(response)
            );
            syslog(LOG_INFO, "fofline seat browser host navigate denied");
            seat_browser_host_resolve(bridge, reply);
            g_free(response);
            g_free(text);
            return;
        }
        g_free(response);
        if (!empiier_native_bridge_show_seat_browser(bridge, url)) {
            seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"seat_browser_unavailable\"}");
            g_free(text);
            return;
        }
    } else if (!seat_browser_host_show_existing(bridge)) {
        seat_browser_host_resolve(bridge, "{\"ok\":false,\"code\":\"seat_browser_unavailable\"}");
        g_free(text);
        return;
    }
    seat_browser_host_resolve(bridge, "{\"ok\":true}");
    g_free(text);
}

void empiier_seat_browser_register_product_bridge(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return;
    }
    g_signal_connect(
        bridge->manager,
        "script-message-received::empiierosSeatBrowser",
        G_CALLBACK(on_seat_browser_host_message),
        bridge
    );
    if (!webkit_user_content_manager_register_script_message_handler(
            bridge->manager,
            "empiierosSeatBrowser"
        )) {
        g_error("failed to register empiierosSeatBrowser script message handler");
    }
}
