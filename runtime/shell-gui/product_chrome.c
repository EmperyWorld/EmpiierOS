#define _POSIX_C_SOURCE 200809L

#include "native_bridge.h"

#include <cairo.h>
#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <syslog.h>

#define CHROME_CSS \
    "#empiieros-product-chrome," \
    "#empiieros-product-chrome scrolledwindow," \
    "#empiieros-product-chrome trough," \
    "#empiieros-product-chrome undershoot," \
    "#empiieros-product-chrome overshoot {" \
    "  background-color: transparent;" \
    "  background-image: none;" \
    "  border: none;" \
    "  box-shadow: none;" \
    "  overflow: hidden;" \
    "}" \
    "#empiieros-product-cluster {" \
    "  background-color: rgba(17, 24, 39, 0.82);" \
    "  border-radius: 28px;" \
    "  border: 1px solid rgba(56, 189, 248, 0.38);" \
    "  padding: 5px 8px;" \
    "}" \
    "#empiieros-product-cluster button {" \
    "  background-image: none;" \
    "  background-color: transparent;" \
    "  border: none;" \
    "  border-radius: 16px;" \
    "  padding: 2px;" \
    "  box-shadow: none;" \
    "  outline-width: 0;" \
    "  color: #e5e7eb;" \
    "  min-width: 60px;" \
    "  min-height: 60px;" \
    "}" \
    "#empiieros-product-cluster button:hover {" \
    "  background-color: rgba(56, 189, 248, 0.16);" \
    "}" \
    "#empiieros-product-cluster #empiieros-hud-caption {" \
    "  color: #e5e7eb;" \
    "  font-size: 9px;" \
    "}" \
    "#empiieros-hud-grip {" \
    "  min-height: 14px;" \
    "  background-color: rgba(56, 189, 248, 0.28);" \
    "  border-radius: 8px;" \
    "}" \
    "popover," \
    "popover.background," \
    "popover contents," \
    "#empiieros-seat-popover," \
    "#empiieros-seat-panel {" \
    "  background-image: none;" \
    "  background-color: rgba(17, 24, 39, 0.98);" \
    "  color: #e5e7eb;" \
    "  border: 1px solid rgba(56, 189, 248, 0.38);" \
    "  border-radius: 16px;" \
    "}" \
    "#empiieros-seat-panel {" \
    "  padding: 10px 12px;" \
    "  min-width: 360px;" \
    "}" \
    "#empiieros-seat-panel label {" \
    "  color: #e5e7eb;" \
    "  background-color: transparent;" \
    "}" \
    "#empiieros-seat-panel button," \
    "#empiieros-seat-panel combobox," \
    "#empiieros-seat-panel combobox > box," \
    "#empiieros-seat-panel combobox > .linked," \
    "#empiieros-seat-panel combobox button," \
    "#empiieros-seat-panel combobox cellview {" \
    "  background-image: none;" \
    "  background-color: #111827;" \
    "  color: #e5e7eb;" \
    "  border: 1px solid rgba(56, 189, 248, 0.28);" \
    "  box-shadow: none;" \
    "}" \
    "#empiieros-seat-panel menuitem," \
    "#empiieros-seat-panel menu {" \
    "  background-image: none;" \
    "  background-color: #111827;" \
    "  color: #e5e7eb;" \
    "}" \
    "#empiieros-seat-panel switch {" \
    "  background-image: none;" \
    "  background-color: #1f2937;" \
    "  color: #e5e7eb;" \
    "  border: 1px solid rgba(56, 189, 248, 0.40);" \
    "  border-radius: 12px;" \
    "  min-width: 46px;" \
    "  min-height: 24px;" \
    "  box-shadow: none;" \
    "}" \
    "#empiieros-seat-panel switch:checked {" \
    "  background-color: rgba(56, 189, 248, 0.34);" \
    "}" \
    "#empiieros-seat-panel switch slider {" \
    "  background-image: none;" \
    "  background-color: #7dd3fc;" \
    "  border: none;" \
    "  border-radius: 10px;" \
    "  min-width: 20px;" \
    "  min-height: 20px;" \
    "  box-shadow: none;" \
    "}" \
    "#empiieros-seat-panel row," \
    "#empiieros-seat-panel box {" \
    "  background-color: transparent;" \
    "}"

typedef struct {
    EmpiierNativeBridge *bridge;
    GtkWidget *chrome;
    GtkWidget *cluster;
    GtkWidget *flyout[6];
    GtkWidget *elite_btn;
    GtkWidget *panel_btn;
    GtkWidget *shield_btn;
    GtkWidget *shield_image;
    GtkWidget *ota_btn;
    GtkWidget *volume_btn;
    gboolean seat_licensed;
    GtkWidget *volume_level;
    GtkWidget *launcher_summary;
    GtkWidget *elite_summary;
    GtkWidget *shield_status;
    GtkWidget *shield_events;
    GtkWidget *ota_status;
    GtkWidget *mic_combo;
    GtkWidget *cam_combo;
    GtkWidget *mic_switch;
    GtkWidget *cam_switch;
    GtkWidget *gps_switch;
    GtkWidget *sensor_note;
    guint shield_tick;
    gboolean combo_busy;
    GtkWidget *grip;
    int pos_x;
    int pos_y;
    gboolean placed;
    gboolean dragging;
    int grab_root_x;
    int grab_root_y;
    int grab_pos_x;
    int grab_pos_y;
} ProductChromeHud;

typedef struct {
    EmpiierNativeBridge *bridge;
    char workspace[24];
    gboolean activator;
} PendingHostWorkspace;

static const char *const k_media_roots[] = {
    "/opt/empiieros/shell/product-chrome",
    "/usr/share/empiieros/product-chrome",
};

static gboolean resolve_media(const char *name, char *out, size_t out_len) {
    size_t i;

    for (i = 0; i < sizeof(k_media_roots) / sizeof(k_media_roots[0]); i++) {
        if (g_snprintf(out, out_len, "%s/%s", k_media_roots[i], name) < 0) {
            continue;
        }
        if (g_file_test(out, G_FILE_TEST_IS_REGULAR)) {
            return TRUE;
        }
    }
    return FALSE;
}

static GdkPixbuf *media_pixbuf(const char *name, int px) {
    char path[256];
    GdkPixbuf *pixbuf;
    GdkPixbuf *scaled;

    if (px < 16) {
        px = 16;
    }
    if (!resolve_media(name, path, sizeof(path))) {
        return NULL;
    }
    pixbuf = gdk_pixbuf_new_from_file(path, NULL);
    if (pixbuf == NULL) {
        return NULL;
    }
    scaled = gdk_pixbuf_scale_simple(pixbuf, px, px, GDK_INTERP_BILINEAR);
    g_object_unref(pixbuf);
    return scaled;
}

static GtkWidget *media_image_px(const char *name, int px) {
    GdkPixbuf *scaled = media_pixbuf(name, px);
    GtkWidget *image;

    if (scaled == NULL) {
        return gtk_image_new_from_icon_name("image-missing", GTK_ICON_SIZE_BUTTON);
    }
    image = gtk_image_new_from_pixbuf(scaled);
    g_object_unref(scaled);
    return image;
}

static GtkWidget *media_image(const char *name) {
    return media_image_px(name, 56);
}

static void hud_button_with_caption(GtkWidget *button, const char *icon, const char *caption) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *label = gtk_label_new(caption);

    gtk_widget_set_name(label, "empiieros-hud-caption");
    gtk_label_set_max_width_chars(GTK_LABEL(label), 12);
    gtk_container_add(GTK_CONTAINER(box), media_image_px(icon, 44));
    gtk_box_pack_start(GTK_BOX(box), label, FALSE, FALSE, 0);
    gtk_container_add(GTK_CONTAINER(button), box);
}

static void media_set_image(GtkWidget *image, const char *name, int px) {
    GdkPixbuf *scaled;

    if (image == NULL || !GTK_IS_IMAGE(image)) {
        return;
    }
    scaled = media_pixbuf(name, px);
    if (scaled == NULL) {
        return;
    }
    gtk_image_set_from_pixbuf(GTK_IMAGE(image), scaled);
    g_object_unref(scaled);
}

static gboolean spawn_click(gchar **argv) {
    return g_spawn_async(
        NULL,
        argv,
        NULL,
        G_SPAWN_SEARCH_PATH | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );
}

static void play_click(const char *name) {
    char path[256];
    gchar *argv[4];

    if (name == NULL || !resolve_media(name, path, sizeof(path))) {
        syslog(LOG_INFO, "seat chrome click missing name=%s", name != NULL ? name : "-");
        return;
    }
    /* Seat has raw ALSA. paplay/pw-play can spawn and stay silent. */
    argv[0] = "aplay";
    argv[1] = "-q";
    argv[2] = path;
    argv[3] = NULL;
    if (spawn_click(argv)) {
        syslog(LOG_INFO, "seat chrome click alsa name=%s", name);
        return;
    }
    argv[0] = "paplay";
    argv[1] = path;
    argv[2] = NULL;
    if (spawn_click(argv)) {
        return;
    }
    argv[0] = "pw-play";
    (void)spawn_click(argv);
}

#define HUD_FLY_LAUNCHER 0
#define HUD_FLY_ELITE 1
#define HUD_FLY_PANEL 2
#define HUD_FLY_SHIELD 3
#define HUD_FLY_VOLUME 4
#define HUD_FLY_OTA 5
#define HUD_FLYOUT_N 6

#define CHROME_PIN_END 10
#define HUD_POS_PATH "/var/lib/empiieros-runtime/hud-chrome-pos.json"

static void refresh_launcher(ProductChromeHud *hud);
static void refresh_ota(ProductChromeHud *hud);
static void hud_apply_pos(ProductChromeHud *hud);
static void hud_orient_flyouts(ProductChromeHud *hud);

static void chrome_hide_flyouts(ProductChromeHud *hud) {
    size_t i;

    if (hud == NULL) {
        return;
    }
    for (i = 0; i < HUD_FLYOUT_N; i++) {
        if (hud->flyout[i] != NULL && gtk_widget_is_visible(hud->flyout[i])) {
            gtk_popover_popdown(GTK_POPOVER(hud->flyout[i]));
        }
    }
}

static void chrome_input_shape(GtkWidget *chrome, GtkAllocation *alloc, gpointer user_data) {
    cairo_rectangle_int_t rect;
    cairo_region_t *region;
    GdkWindow *win;

    (void)user_data;
    if (chrome == NULL || !gtk_widget_get_realized(chrome) || alloc == NULL) {
        return;
    }
    win = gtk_widget_get_window(chrome);
    if (win == NULL) {
        return;
    }
    rect.x = 0;
    rect.y = 0;
    rect.width = alloc->width;
    rect.height = alloc->height;
    region = cairo_region_create_rectangle(&rect);
    gdk_window_input_shape_combine_region(win, region, 0, 0);
    cairo_region_destroy(region);
}

static void hud_orient_flyouts(ProductChromeHud *hud) {
    GtkPositionType side;
    size_t i;

    if (hud == NULL) {
        return;
    }
    side = hud->pos_x < 280 ? GTK_POS_RIGHT : GTK_POS_LEFT;
    for (i = 0; i < HUD_FLYOUT_N; i++) {
        if (hud->flyout[i] != NULL) {
            gtk_popover_set_position(GTK_POPOVER(hud->flyout[i]), side);
        }
    }
}

static void hud_clamp(ProductChromeHud *hud) {
    GtkWidget *parent;
    GtkAllocation parent_alloc;
    GtkAllocation chrome_alloc;
    int max_x;
    int max_y;

    if (hud == NULL || hud->chrome == NULL) {
        return;
    }
    parent = gtk_widget_get_parent(hud->chrome);
    if (parent == NULL) {
        return;
    }
    gtk_widget_get_allocation(parent, &parent_alloc);
    gtk_widget_get_allocation(hud->chrome, &chrome_alloc);
    max_x = parent_alloc.width - chrome_alloc.width;
    max_y = parent_alloc.height - chrome_alloc.height;
    if (max_x < 0) {
        max_x = 0;
    }
    if (max_y < 0) {
        max_y = 0;
    }
    if (hud->pos_x < 0) {
        hud->pos_x = 0;
    }
    if (hud->pos_y < 0) {
        hud->pos_y = 0;
    }
    if (hud->pos_x > max_x) {
        hud->pos_x = max_x;
    }
    if (hud->pos_y > max_y) {
        hud->pos_y = max_y;
    }
}

static void hud_apply_pos(ProductChromeHud *hud) {
    if (hud == NULL || hud->chrome == NULL) {
        return;
    }
    gtk_widget_set_halign(hud->chrome, GTK_ALIGN_START);
    gtk_widget_set_valign(hud->chrome, GTK_ALIGN_START);
    gtk_widget_set_margin_start(hud->chrome, hud->pos_x);
    gtk_widget_set_margin_top(hud->chrome, hud->pos_y);
    gtk_widget_set_margin_end(hud->chrome, 0);
    hud_orient_flyouts(hud);
}

static gboolean hud_load_pos(ProductChromeHud *hud) {
    gchar *raw = NULL;
    const char *px;
    const char *py;
    int x = 0;
    int y = 0;

    if (hud == NULL || !g_file_get_contents(HUD_POS_PATH, &raw, NULL, NULL) || raw == NULL) {
        return FALSE;
    }
    px = strstr(raw, "\"x\"");
    py = strstr(raw, "\"y\"");
    if (px == NULL || py == NULL || sscanf(px + 3, " :%d", &x) != 1 || sscanf(py + 3, " :%d", &y) != 1) {
        g_free(raw);
        return FALSE;
    }
    g_free(raw);
    if (x < 0 || y < 0 || x > 8000 || y > 8000) {
        return FALSE;
    }
    hud->pos_x = x;
    hud->pos_y = y;
    return TRUE;
}

static void hud_save_pos(const ProductChromeHud *hud) {
    gchar *body;

    if (hud == NULL) {
        return;
    }
    body = g_strdup_printf("{\"x\":%d,\"y\":%d}\n", hud->pos_x, hud->pos_y);
    (void)g_file_set_contents(HUD_POS_PATH, body, -1, NULL);
    g_free(body);
}

static void on_chrome_mapped(GtkWidget *chrome, gpointer user_data) {
    ProductChromeHud *hud = user_data;
    GtkWidget *parent;
    GtkAllocation parent_alloc;
    GtkAllocation chrome_alloc;

    (void)chrome;
    if (hud == NULL || hud->placed || hud->chrome == NULL) {
        return;
    }
    parent = gtk_widget_get_parent(hud->chrome);
    if (parent == NULL) {
        return;
    }
    gtk_widget_get_allocation(parent, &parent_alloc);
    gtk_widget_get_allocation(hud->chrome, &chrome_alloc);
    if (!hud_load_pos(hud)) {
        hud->pos_x = chrome_alloc.x;
        hud->pos_y = chrome_alloc.y;
        if (hud->pos_x < 0) {
            hud->pos_x = 0;
        }
        if (hud->pos_y < 0) {
            hud->pos_y = 0;
        }
    }
    hud_clamp(hud);
    hud_apply_pos(hud);
    hud->placed = TRUE;
}

static gboolean on_grip_press(GtkWidget *widget, GdkEventButton *event, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)widget;
    if (hud == NULL || event == NULL || event->button != 1) {
        return FALSE;
    }
    hud->dragging = TRUE;
    hud->grab_root_x = (int)event->x_root;
    hud->grab_root_y = (int)event->y_root;
    hud->grab_pos_x = hud->pos_x;
    hud->grab_pos_y = hud->pos_y;
    gtk_grab_add(widget);
    chrome_hide_flyouts(hud);
    return TRUE;
}

static gboolean on_grip_motion(GtkWidget *widget, GdkEventMotion *event, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)widget;
    if (hud == NULL || !hud->dragging || event == NULL) {
        return FALSE;
    }
    hud->pos_x = hud->grab_pos_x + (int)event->x_root - hud->grab_root_x;
    hud->pos_y = hud->grab_pos_y + (int)event->y_root - hud->grab_root_y;
    hud_clamp(hud);
    hud_apply_pos(hud);
    return TRUE;
}

static gboolean on_grip_release(GtkWidget *widget, GdkEventButton *event, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)widget;
    if (hud == NULL || event == NULL || event->button != 1 || !hud->dragging) {
        return FALSE;
    }
    hud->dragging = FALSE;
    gtk_grab_remove(widget);
    hud_clamp(hud);
    hud_apply_pos(hud);
    hud_save_pos(hud);
    return TRUE;
}

static GtkWidget *panel_heading(const char *text) {
    GtkWidget *label = gtk_label_new(text);
    gtk_widget_set_halign(label, GTK_ALIGN_START);
    gtk_label_set_xalign(GTK_LABEL(label), 0.0);
    return label;
}

static GtkWidget *panel_wrap(const char *text) {
    GtkWidget *label = gtk_label_new(text);
    gtk_widget_set_halign(label, GTK_ALIGN_START);
    gtk_label_set_xalign(GTK_LABEL(label), 0.0);
    gtk_label_set_line_wrap(GTK_LABEL(label), TRUE);
    gtk_label_set_max_width_chars(GTK_LABEL(label), 42);
    return label;
}

static gboolean json_extract_string(
    const char *body,
    const char *key,
    char *out,
    size_t out_len
) {
    char needle[96];
    const char *start;
    const char *end;
    size_t n;

    if (body == NULL || key == NULL || out == NULL || out_len == 0) {
        return FALSE;
    }
    if (g_snprintf(needle, sizeof(needle), "\"%s\":\"", key) < 0) {
        return FALSE;
    }
    start = strstr(body, needle);
    if (start == NULL) {
        return FALSE;
    }
    start += strlen(needle);
    end = strchr(start, '"');
    if (end == NULL) {
        return FALSE;
    }
    n = (size_t)(end - start);
    if (n >= out_len) {
        n = out_len - 1;
    }
    memcpy(out, start, n);
    out[n] = '\0';
    return TRUE;
}

static gboolean json_extract_int(const char *body, const char *key, long *out) {
    char needle[96];
    const char *start;
    char *end = NULL;

    if (body == NULL || key == NULL || out == NULL) {
        return FALSE;
    }
    if (g_snprintf(needle, sizeof(needle), "\"%s\":", key) < 0) {
        return FALSE;
    }
    start = strstr(body, needle);
    if (start == NULL) {
        return FALSE;
    }
    start += strlen(needle);
    *out = strtol(start, &end, 10);
    return end != start;
}

static char *runtime_op(ProductChromeHud *hud, const char *request) {
    GError *error = NULL;
    char *body;

    if (hud == NULL || hud->bridge == NULL || request == NULL) {
        return NULL;
    }
    body = empiier_native_runtime_exchange(hud->bridge, request, &error);
    if (error != NULL) {
        syslog(LOG_INFO, "seat chrome ipc fail %s", error->message);
        g_error_free(error);
    }
    return body;
}

static void fill_mic_combo(ProductChromeHud *hud) {
    FILE *cards;
    char line[256];
    int count = 0;

    hud->combo_busy = TRUE;
    gtk_combo_box_text_remove_all(GTK_COMBO_BOX_TEXT(hud->mic_combo));
    cards = fopen("/proc/asound/cards", "r");
    if (cards != NULL) {
        while (fgets(line, sizeof(line), cards) != NULL) {
            int id = -1;
            char short_name[64];

            if (sscanf(line, " %d [%63[^]]", &id, short_name) == 2) {
                gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(hud->mic_combo), short_name);
                count++;
            }
        }
        fclose(cards);
    }
    if (count == 0) {
        gtk_combo_box_text_append_text(
            GTK_COMBO_BOX_TEXT(hud->mic_combo),
            "карт ALSA не видно"
        );
    }
    gtk_combo_box_set_active(GTK_COMBO_BOX(hud->mic_combo), 0);
    hud->combo_busy = FALSE;
}

static int cam_name_is_metadata(const char *name) {
    return name != NULL
        && (strstr(name, "meta") != NULL || strstr(name, "Meta") != NULL);
}

static void fill_cam_combo(ProductChromeHud *hud) {
    DIR *dev;
    struct dirent *ent;
    int count = 0;

    hud->combo_busy = TRUE;
    gtk_combo_box_text_remove_all(GTK_COMBO_BOX_TEXT(hud->cam_combo));
    dev = opendir("/sys/class/video4linux");
    if (dev != NULL) {
        while ((ent = readdir(dev)) != NULL) {
            char sys_name[128];
            char label[192];
            FILE *name_file;

            if (ent->d_name[0] == '.' || !g_str_has_prefix(ent->d_name, "video")) {
                continue;
            }
            if (cam_name_is_metadata(ent->d_name)) {
                continue;
            }
            g_snprintf(sys_name, sizeof(sys_name), "/sys/class/video4linux/%s/name", ent->d_name);
            name_file = fopen(sys_name, "r");
            if (name_file != NULL) {
                char pretty[96];
                if (fgets(pretty, sizeof(pretty), name_file) != NULL) {
                    pretty[strcspn(pretty, "\r\n")] = '\0';
                    if (cam_name_is_metadata(pretty)) {
                        fclose(name_file);
                        continue;
                    }
                    g_snprintf(label, sizeof(label), "%s %s", ent->d_name, pretty);
                } else {
                    g_strlcpy(label, ent->d_name, sizeof(label));
                }
                fclose(name_file);
            } else {
                g_strlcpy(label, ent->d_name, sizeof(label));
            }
            gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(hud->cam_combo), label);
            count++;
        }
        closedir(dev);
    }
    if (count == 0) {
        gtk_combo_box_text_append_text(
            GTK_COMBO_BOX_TEXT(hud->cam_combo),
            "узлов /dev/video* нет"
        );
        syslog(LOG_INFO, "seat chrome camera none");
    } else {
        syslog(LOG_INFO, "seat chrome camera count=%d", count);
    }
    gtk_combo_box_set_active(GTK_COMBO_BOX(hud->cam_combo), 0);
    hud->combo_busy = FALSE;
}

static void collect_summaries(const char *body, GString *out) {
    const char *cursor = body;
    int n = 0;

    if (body == NULL) {
        g_string_assign(out, "События: Runtime не ответил");
        return;
    }
    while (n < 3) {
        const char *start = strstr(cursor, "\"summary\":\"");
        const char *end;

        if (start == NULL) {
            break;
        }
        start += 11;
        end = strchr(start, '"');
        if (end == NULL) {
            break;
        }
        if (out->len > 0) {
            g_string_append_c(out, '\n');
        }
        g_string_append_len(out, start, end - start);
        cursor = end + 1;
        n++;
    }
    if (n == 0) {
        g_string_assign(out, "Важных событий нет");
    }
}

static void refresh_shield(ProductChromeHud *hud) {
    char *status;
    char *events;
    char mode[48];
    char detail[160];
    long high = 0;
    GString *status_line;
    GString *event_line;
    char tip[192];

    if (hud == NULL) {
        return;
    }
    status = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-st\",\"op\":\"seat.shield.status\",\"params\":{}}"
    );
    events = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-ev\",\"op\":\"seat.shield.events\","
        "\"params\":{\"after_sequence\":0,\"limit\":3}}"
    );
    status_line = g_string_new(NULL);
    event_line = g_string_new(NULL);
    if (status == NULL || !json_extract_string(status, "mode", mode, sizeof(mode))) {
        g_strlcpy(tip, "Щит: нет ответа", sizeof(tip));
        g_string_assign(status_line, "Seat Shield: Runtime не ответил");
        media_set_image(hud->shield_image, "hud-shield.png", 56);
    } else {
        (void)json_extract_int(status, "open_high_events", &high);
        if (!json_extract_string(status, "detail", detail, sizeof(detail))) {
            g_strlcpy(detail, mode, sizeof(detail));
        }
        g_snprintf(tip, sizeof(tip), "Щит: %s · %ld high", mode, high);
        g_string_printf(
            status_line,
            "Режим %s. Открытых high/critical: %ld. %s",
            mode,
            high,
            detail
        );
        media_set_image(
            hud->shield_image,
            high > 0 ? "hud-shield-alert.png" : "hud-shield.png",
            56
        );
    }
    collect_summaries(events, event_line);
    if (hud->shield_btn != NULL) {
        gtk_widget_set_tooltip_text(hud->shield_btn, tip);
    }
    gtk_label_set_text(GTK_LABEL(hud->shield_status), status_line->str);
    gtk_label_set_text(GTK_LABEL(hud->shield_events), event_line->str);
    g_string_free(status_line, TRUE);
    g_string_free(event_line, TRUE);
    g_free(status);
    g_free(events);
}

static void refresh_volume(ProductChromeHud *hud) {
    char *body;
    long pct = -1;
    char tip[80];
    char line[80];

    if (hud == NULL) {
        return;
    }
    body = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-vol\",\"op\":\"system.snapshot\",\"params\":{}}"
    );
    if (body != NULL && json_extract_int(body, "observed_percent", &pct)
        && pct >= 0 && pct <= 100) {
        g_snprintf(tip, sizeof(tip), "Громкость ALSA %ld%%", pct);
        g_snprintf(line, sizeof(line), "%ld%% · audio.control", pct);
    } else {
        g_strlcpy(tip, "Громкость ALSA (observe)", sizeof(tip));
        g_strlcpy(line, "уровень не наблюдался", sizeof(line));
    }
    if (hud->volume_btn != NULL) {
        gtk_widget_set_tooltip_text(hud->volume_btn, tip);
    }
    if (hud->volume_level != NULL) {
        gtk_label_set_text(GTK_LABEL(hud->volume_level), line);
    }
    g_free(body);
}

static void refresh_panel(ProductChromeHud *hud) {
    fill_mic_combo(hud);
    fill_cam_combo(hud);
    gtk_switch_set_active(GTK_SWITCH(hud->gps_switch), FALSE);
    gtk_widget_set_sensitive(hud->gps_switch, FALSE);
    gtk_label_set_text(
        GTK_LABEL(hud->sensor_note),
        "Микрофон и камера только наблюдаются. audio.action not wired — "
        "выбор не переключает вход. Камера без системного kill-switch. "
        "GPS: gnss_not_on_seat."
    );
}

static void refresh_ota(ProductChromeHud *hud) {
    GString *line;
    char *body;
    char gate[64];
    char pk[32];
    char staged[16];

    if (hud == NULL || hud->ota_status == NULL) {
        return;
    }
    line = g_string_new(NULL);
    body = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-ota\",\"op\":\"launcher.ota_status\",\"params\":{}}"
    );
    if (body == NULL) {
        g_string_assign(line, "Обновления: Runtime не ответил.");
    } else {
        if (!json_extract_string(body, "ota_gate", gate, sizeof(gate))) {
            g_strlcpy(gate, "unknown", sizeof(gate));
        }
        if (!json_extract_string(body, "artifact_pk", pk, sizeof(pk))) {
            g_strlcpy(pk, "unknown", sizeof(pk));
        }
        if (strstr(body, "\"ota_staged\":true") != NULL) {
            g_strlcpy(staged, "staged", sizeof(staged));
        } else {
            g_strlcpy(staged, "idle", sizeof(staged));
        }
        g_string_printf(line, "Гейт %s. PK %s. %s.", gate, pk, staged);
        g_free(body);
    }
    gtk_label_set_text(GTK_LABEL(hud->ota_status), line->str);
    g_string_free(line, TRUE);
}

static gboolean on_shield_tick(gpointer user_data) {
    refresh_launcher(user_data);
    refresh_shield(user_data);
    refresh_ota(user_data);
    return G_SOURCE_CONTINUE;
}

static gboolean product_is_open(EmpiierNativeBridge *bridge) {
    const gchar *uri;

    if (bridge == NULL || bridge->web_view == NULL) {
        return FALSE;
    }
    if (bridge->product_visible) {
        return TRUE;
    }
    uri = webkit_web_view_get_uri(bridge->web_view);
    return empiier_native_bridge_fofline_uri(uri);
}

static void apply_seat_license_lock(ProductChromeHud *hud) {
    if (hud == NULL) {
        return;
    }
    if (hud->elite_btn != NULL) {
        gtk_widget_set_sensitive(hud->elite_btn, hud->seat_licensed);
    }
    if (hud->panel_btn != NULL) {
        gtk_widget_set_sensitive(hud->panel_btn, hud->seat_licensed);
    }
    if (hud->shield_btn != NULL) {
        gtk_widget_set_sensitive(hud->shield_btn, hud->seat_licensed);
    }
    if (hud->volume_btn != NULL) {
        gtk_widget_set_sensitive(hud->volume_btn, hud->seat_licensed);
    }
    if (hud->ota_btn != NULL) {
        gtk_widget_set_sensitive(hud->ota_btn, hud->seat_licensed);
    }
}

static void refresh_launcher(ProductChromeHud *hud) {
    GString *line;
    char *body;
    char spa[48];
    char oidc[32];

    if (hud == NULL || hud->launcher_summary == NULL) {
        return;
    }
    line = g_string_new(NULL);
    if (product_is_open(hud->bridge)) {
        g_string_assign(line, "EliteEW открыт. «Открыть лаунчер» вернёт на хост.");
    } else {
        body = runtime_op(
            hud,
            "{\"id\":\"seat-chrome-ln\",\"op\":\"launcher.status\",\"params\":{}}"
        );
        if (body == NULL) {
            g_string_assign(line, "Лаунчер: Runtime не ответил.");
            hud->seat_licensed = FALSE;
        } else {
            hud->seat_licensed =
                strstr(body, "\"capsule_imported\":true") != NULL
                && strstr(body, "\"entitlement_present\":true") != NULL;
            if (!json_extract_string(body, "spa_state", spa, sizeof(spa))) {
                g_strlcpy(spa, "unknown", sizeof(spa));
            }
            if (!json_extract_string(body, "oidc_session", oidc, sizeof(oidc))) {
                g_strlcpy(oidc, "unknown", sizeof(oidc));
            }
            if (!hud->seat_licensed) {
                g_string_assign(
                    line,
                    "Без лицензии FoFline доступен только путь активации."
                );
            } else {
                g_string_printf(
                    line,
                    "Хост. Пакет %s. OIDC %s. Иконка не открывает лаунчер сама.",
                    spa,
                    oidc
                );
            }
            g_free(body);
        }
    }
    gtk_label_set_text(GTK_LABEL(hud->launcher_summary), line->str);
    g_string_free(line, TRUE);
    apply_seat_license_lock(hud);
}

static void refresh_elite(ProductChromeHud *hud) {
    GString *line;
    char *body;
    char spa[48];
    char oidc[32];

    if (hud == NULL || hud->elite_summary == NULL) {
        return;
    }
    line = g_string_new(NULL);
    if (product_is_open(hud->bridge)) {
        g_string_assign(line, "EliteEW уже на экране.");
    } else {
        body = runtime_op(
            hud,
            "{\"id\":\"seat-chrome-el\",\"op\":\"launcher.status\",\"params\":{}}"
        );
        if (body == NULL) {
            g_string_assign(line, "EliteEW: Runtime не ответил.");
        } else {
            if (!json_extract_string(body, "spa_state", spa, sizeof(spa))) {
                g_strlcpy(spa, "unknown", sizeof(spa));
            }
            if (!json_extract_string(body, "oidc_session", oidc, sizeof(oidc))) {
                g_strlcpy(oidc, "unknown", sizeof(oidc));
            }
            g_string_printf(
                line,
                "Пакет %s. OIDC %s. Открыть идёт через product.open.",
                spa,
                oidc
            );
            g_free(body);
        }
    }
    gtk_label_set_text(GTK_LABEL(hud->elite_summary), line->str);
    g_string_free(line, TRUE);
}

static void chrome_refresh_flyout(ProductChromeHud *hud, guint kind) {
    if (hud == NULL) {
        return;
    }
    if (kind == HUD_FLY_LAUNCHER) {
        refresh_launcher(hud);
    } else if (kind == HUD_FLY_ELITE) {
        refresh_elite(hud);
    } else if (kind == HUD_FLY_PANEL) {
        refresh_panel(hud);
    } else if (kind == HUD_FLY_SHIELD) {
        refresh_shield(hud);
    } else if (kind == HUD_FLY_VOLUME) {
        refresh_volume(hud);
    } else if (kind == HUD_FLY_OTA) {
        refresh_ota(hud);
    }
}

static void chrome_toggle_flyout(
    ProductChromeHud *hud,
    guint kind,
    const char *wav,
    const char *name
) {
    GtkWidget *pop;
    gboolean was_open;

    if (hud == NULL || kind >= HUD_FLYOUT_N) {
        return;
    }
    play_click(wav);
    pop = hud->flyout[kind];
    if (pop == NULL) {
        return;
    }
    was_open = gtk_widget_is_visible(pop);
    chrome_hide_flyouts(hud);
    if (was_open) {
        syslog(LOG_INFO, "seat chrome flyout kind=%s closed", name);
        return;
    }
    chrome_refresh_flyout(hud, kind);
    gtk_popover_popup(GTK_POPOVER(pop));
    syslog(LOG_INFO, "seat chrome flyout kind=%s", name);
}

static void on_shell_workspace_load(
    WebKitWebView *view,
    WebKitLoadEvent event,
    gpointer user_data
) {
    PendingHostWorkspace *job = user_data;
    const gchar *uri;
    char detail[96];

    if (event != WEBKIT_LOAD_FINISHED) {
        return;
    }
    uri = webkit_web_view_get_uri(view);
    if (uri == NULL || !g_str_has_prefix(uri, "empiier://")) {
        return;
    }
    if (job->activator) {
        g_snprintf(
            detail,
            sizeof(detail),
            "{\"workspace\":\"%s\",\"activator\":true}",
            job->workspace
        );
    } else {
        g_snprintf(detail, sizeof(detail), "{\"workspace\":\"%s\"}", job->workspace);
    }
    empiier_native_dispatch_custom_event(job->bridge, "empiieros-host-workspace", detail);
    g_signal_handlers_disconnect_by_func(view, on_shell_workspace_load, job);
    g_free(job);
}

static void open_host_workspace(
    ProductChromeHud *hud,
    const char *workspace,
    gboolean activator
) {
    PendingHostWorkspace *job;
    char detail[96];

    if (hud == NULL || hud->bridge == NULL || workspace == NULL || hud->bridge->web_view == NULL) {
        return;
    }
    if (!product_is_open(hud->bridge)) {
        if (activator) {
            g_snprintf(
                detail,
                sizeof(detail),
                "{\"workspace\":\"%s\",\"activator\":true}",
                workspace
            );
        } else {
            g_snprintf(detail, sizeof(detail), "{\"workspace\":\"%s\"}", workspace);
        }
        empiier_native_dispatch_custom_event(
            hud->bridge,
            "empiieros-host-workspace",
            detail
        );
        return;
    }
    job = g_new0(PendingHostWorkspace, 1);
    job->bridge = hud->bridge;
    job->activator = activator;
    g_strlcpy(job->workspace, workspace, sizeof(job->workspace));
    g_signal_connect(
        hud->bridge->web_view,
        "load-changed",
        G_CALLBACK(on_shell_workspace_load),
        job
    );
    empiier_native_bridge_hide_product(hud->bridge);
}

static void on_back_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    chrome_toggle_flyout(user_data, HUD_FLY_LAUNCHER, "click-launcher.wav", "launcher");
}

static void on_eliteew_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    if (hud != NULL && !hud->seat_licensed) {
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    chrome_toggle_flyout(user_data, HUD_FLY_ELITE, "click-eliteew.wav", "eliteew");
}

static void on_panel_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    if (hud != NULL && !hud->seat_licensed) {
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    chrome_toggle_flyout(user_data, HUD_FLY_PANEL, "click-eliteew.wav", "panel");
}

static void on_volume_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    if (hud != NULL && !hud->seat_licensed) {
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    chrome_toggle_flyout(user_data, HUD_FLY_VOLUME, "click-eliteew.wav", "volume");
}

static void on_shield_icon_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    chrome_toggle_flyout(user_data, HUD_FLY_SHIELD, "click-eliteew.wav", "shield");
}

static void on_observe_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;
    char *body;

    (void)button;
    body = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-ob\",\"op\":\"seat.shield.observe\",\"params\":{}}"
    );
    if (body == NULL) {
        gtk_label_set_text(GTK_LABEL(hud->shield_status), "Observe: Runtime не ответил");
    } else if (strstr(body, "\"ok\":false") != NULL) {
        char code[64];
        if (!json_extract_string(body, "code", code, sizeof(code))) {
            g_strlcpy(code, "failed", sizeof(code));
        }
        syslog(LOG_INFO, "seat chrome shield observe code=%s", code);
        gtk_label_set_text(
            GTK_LABEL(hud->shield_status),
            "Observe недоступен (backend / Runtime)"
        );
    }
    g_free(body);
    refresh_shield(hud);
}

static void on_open_shield_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    play_click("click-eliteew.wav");
    chrome_hide_flyouts(hud);
    if (!hud->seat_licensed) {
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    open_host_workspace(hud, "shield", FALSE);
}

static void on_ota_icon_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    chrome_toggle_flyout(user_data, HUD_FLY_OTA, "click-eliteew.wav", "ota");
}

static void on_open_ota_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    play_click("click-eliteew.wav");
    chrome_hide_flyouts(hud);
    if (!hud->seat_licensed) {
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    open_host_workspace(hud, "ota", FALSE);
}

static void on_open_launcher_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)button;
    play_click("click-launcher.wav");
    chrome_hide_flyouts(hud);
    if (product_is_open(hud->bridge)) {
        empiier_native_bridge_hide_product(hud->bridge);
    }
    open_host_workspace(hud, "eliteew", TRUE);
}

static void on_open_elite_clicked(GtkButton *button, gpointer user_data) {
    ProductChromeHud *hud = user_data;
    char *body;
    char entrypoint[192];
    char code[64];
    char detail[160];

    (void)button;
    play_click("click-eliteew.wav");
    if (!hud->seat_licensed) {
        chrome_hide_flyouts(hud);
        open_host_workspace(hud, "eliteew", TRUE);
        return;
    }
    if (product_is_open(hud->bridge)) {
        chrome_hide_flyouts(hud);
        if (hud->bridge != NULL && hud->bridge->web_view != NULL) {
            gtk_widget_grab_focus(GTK_WIDGET(hud->bridge->web_view));
        }
        return;
    }
    body = runtime_op(
        hud,
        "{\"id\":\"seat-chrome-po\",\"op\":\"product.open\","
        "\"params\":{\"target\":\"fofline_local\"}}"
    );
    if (body != NULL
        && strstr(body, "\"ok\":false") == NULL
        && json_extract_string(body, "entrypoint_url", entrypoint, sizeof(entrypoint))
        && g_strcmp0(entrypoint, EMPIIEROS_FOFLINE_SPA_START) == 0
        && empiier_native_bridge_show_product(hud->bridge, entrypoint)) {
        chrome_hide_flyouts(hud);
        syslog(LOG_INFO, "seat chrome eliteew opened product.open");
        g_free(body);
        return;
    }
    if (!json_extract_string(body, "code", code, sizeof(code))) {
        g_strlcpy(code, body == NULL ? "runtime_silent" : "entrypoint_denied", sizeof(code));
    }
    if (!json_extract_string(body, "detail", detail, sizeof(detail))) {
        g_strlcpy(detail, "product.open не открыл пакет", sizeof(detail));
    }
    if (hud->elite_summary != NULL) {
        gchar *line = g_strdup_printf("Открыть нельзя: %s. %s", code, detail);
        gtk_label_set_text(GTK_LABEL(hud->elite_summary), line);
        g_free(line);
    }
    syslog(LOG_INFO, "seat chrome eliteew open denied code=%s", code);
    g_free(body);
}

static void on_combo_changed(GtkComboBox *combo, gpointer user_data) {
    ProductChromeHud *hud = user_data;
    gchar *value;

    if (hud->combo_busy) {
        return;
    }
    value = gtk_combo_box_text_get_active_text(GTK_COMBO_BOX_TEXT(combo));
    syslog(
        LOG_INFO,
        "seat chrome device observed name=%s (audio.action not wired)",
        value != NULL ? value : "-"
    );
    g_free(value);
}

static void on_switch_notify(GObject *object, GParamSpec *pspec, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)pspec;
    if (object == G_OBJECT(hud->gps_switch)) {
        gtk_switch_set_active(GTK_SWITCH(hud->gps_switch), FALSE);
        return;
    }
    syslog(
        LOG_INFO,
        "seat chrome preference only cam=%d mic=%d (no hardware kill-switch)",
        gtk_switch_get_active(GTK_SWITCH(hud->cam_switch)) ? 1 : 0,
        gtk_switch_get_active(GTK_SWITCH(hud->mic_switch)) ? 1 : 0
    );
}

static void apply_chrome_css(void) {
    GdkScreen *screen = gdk_screen_get_default();
    GtkCssProvider *provider;

    if (screen == NULL) {
        return;
    }
    provider = gtk_css_provider_new();
    gtk_css_provider_load_from_data(provider, CHROME_CSS, -1, NULL);
    gtk_style_context_add_provider_for_screen(
        screen,
        GTK_STYLE_PROVIDER(provider),
        GTK_STYLE_PROVIDER_PRIORITY_USER
    );
    g_object_unref(provider);
}

static GtkWidget *panel_icon_row(const char *icon, GtkWidget *middle, GtkWidget *end) {
    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);

    gtk_box_pack_start(GTK_BOX(row), media_image_px(icon, 28), FALSE, FALSE, 0);
    if (middle != NULL) {
        gtk_widget_set_hexpand(middle, TRUE);
        gtk_box_pack_start(GTK_BOX(row), middle, TRUE, TRUE, 0);
    }
    if (end != NULL) {
        gtk_box_pack_start(GTK_BOX(row), end, FALSE, FALSE, 0);
    }
    return row;
}

static GtkWidget *make_flyout(GtkWidget *relative, GtkWidget *box) {
    GtkWidget *popover = gtk_popover_new(relative);

    gtk_widget_set_name(popover, "empiieros-seat-popover");
    gtk_widget_set_name(box, "empiieros-seat-panel");
    gtk_popover_set_modal(GTK_POPOVER(popover), FALSE);
    gtk_popover_set_position(GTK_POPOVER(popover), GTK_POS_LEFT);
    gtk_container_add(GTK_CONTAINER(popover), box);
    gtk_widget_show_all(box);
    gtk_widget_hide(popover);
    return popover;
}

static GtkWidget *build_launcher_flyout(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    GtkWidget *open_btn;

    gtk_box_pack_start(GTK_BOX(box), panel_heading("Лаунчер"), FALSE, FALSE, 0);
    hud->launcher_summary = panel_wrap("сводка…");
    gtk_box_pack_start(GTK_BOX(box), hud->launcher_summary, FALSE, FALSE, 0);
    open_btn = gtk_button_new_with_label("Открыть лаунчер");
    gtk_box_pack_start(GTK_BOX(box), open_btn, FALSE, FALSE, 0);
    g_signal_connect(open_btn, "clicked", G_CALLBACK(on_open_launcher_clicked), hud);
    return make_flyout(relative, box);
}

static GtkWidget *build_elite_flyout(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    GtkWidget *open_btn;

    gtk_box_pack_start(GTK_BOX(box), panel_heading("EliteEW"), FALSE, FALSE, 0);
    hud->elite_summary = panel_wrap("сводка…");
    gtk_box_pack_start(GTK_BOX(box), hud->elite_summary, FALSE, FALSE, 0);
    open_btn = gtk_button_new_with_label("Открыть EliteEW");
    gtk_box_pack_start(GTK_BOX(box), open_btn, FALSE, FALSE, 0);
    g_signal_connect(open_btn, "clicked", G_CALLBACK(on_open_elite_clicked), hud);
    return make_flyout(relative, box);
}

static GtkWidget *build_panel(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);

    gtk_box_pack_start(GTK_BOX(box), panel_heading("Панель сиденья"), FALSE, FALSE, 0);
    hud->mic_switch = gtk_switch_new();
    hud->cam_switch = gtk_switch_new();
    hud->gps_switch = gtk_switch_new();
    hud->mic_combo = gtk_combo_box_text_new();
    hud->cam_combo = gtk_combo_box_text_new();
    gtk_widget_set_tooltip_text(hud->mic_switch, "Микрофон (предпочтение сессии)");
    gtk_widget_set_tooltip_text(hud->cam_switch, "Камера (предпочтение сессии)");
    gtk_widget_set_tooltip_text(hud->gps_switch, "GPS: gnss_not_on_seat");
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("panel-mic.png", hud->mic_combo, hud->mic_switch),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("panel-cam.png", hud->cam_combo, hud->cam_switch),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("panel-gps.png", gtk_label_new("gnss_not_on_seat"), hud->gps_switch),
        FALSE,
        FALSE,
        0
    );
    gtk_widget_set_sensitive(hud->gps_switch, FALSE);
    hud->sensor_note = panel_wrap(
        "Микрофон и камера только наблюдаются. audio.action not wired."
    );
    gtk_box_pack_start(GTK_BOX(box), hud->sensor_note, FALSE, FALSE, 0);
    {
        GtkWidget *open_ota = gtk_button_new_with_label("Открыть обновления");

        gtk_box_pack_start(GTK_BOX(box), open_ota, FALSE, FALSE, 0);
        g_signal_connect(open_ota, "clicked", G_CALLBACK(on_open_ota_clicked), hud);
    }
    g_signal_connect(hud->mic_combo, "changed", G_CALLBACK(on_combo_changed), hud);
    g_signal_connect(hud->cam_combo, "changed", G_CALLBACK(on_combo_changed), hud);
    g_signal_connect(hud->mic_switch, "notify::active", G_CALLBACK(on_switch_notify), hud);
    g_signal_connect(hud->cam_switch, "notify::active", G_CALLBACK(on_switch_notify), hud);
    g_signal_connect(hud->gps_switch, "notify::active", G_CALLBACK(on_switch_notify), hud);
    return make_flyout(relative, box);
}

static GtkWidget *build_shield_flyout(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    GtkWidget *actions = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *observe = gtk_button_new_with_label("Observe");
    GtkWidget *open_shield = gtk_button_new_with_label("Открыть щит");

    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("hud-shield.png", panel_heading("Seat Shield"), NULL),
        FALSE,
        FALSE,
        0
    );
    hud->shield_status = panel_wrap("Seat Shield: ожидание");
    hud->shield_events = panel_wrap("Важных событий нет");
    gtk_box_pack_start(GTK_BOX(actions), observe, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(actions), open_shield, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), hud->shield_status, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), hud->shield_events, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), actions, FALSE, FALSE, 0);
    g_signal_connect(observe, "clicked", G_CALLBACK(on_observe_clicked), hud);
    g_signal_connect(open_shield, "clicked", G_CALLBACK(on_open_shield_clicked), hud);
    return make_flyout(relative, box);
}

static GtkWidget *build_ota_flyout(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    GtkWidget *actions = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *open_ota = gtk_button_new_with_label("Открыть обновления");

    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("hud-ota.png", panel_heading("Обновления"), NULL),
        FALSE,
        FALSE,
        0
    );
    hud->ota_status = panel_wrap("Обновления: ожидание");
    gtk_box_pack_start(GTK_BOX(actions), open_ota, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), hud->ota_status, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), actions, FALSE, FALSE, 0);
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_wrap("Капсульный OTA. Не OS A/B. PK приходит с Air, не с установщика."),
        FALSE,
        FALSE,
        0
    );
    g_signal_connect(open_ota, "clicked", G_CALLBACK(on_open_ota_clicked), hud);
    return make_flyout(relative, box);
}

static GtkWidget *build_volume_flyout(ProductChromeHud *hud, GtkWidget *relative) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);

    gtk_box_pack_start(GTK_BOX(box), panel_heading("Громкость ALSA"), FALSE, FALSE, 0);
    hud->volume_level = gtk_label_new("уровень…");
    gtk_widget_set_halign(hud->volume_level, GTK_ALIGN_START);
    gtk_label_set_xalign(GTK_LABEL(hud->volume_level), 0.0);
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_icon_row("hud-volume.png", hud->volume_level, NULL),
        FALSE,
        FALSE,
        0
    );
    gtk_box_pack_start(
        GTK_BOX(box),
        panel_wrap(
            "Наблюдение audio.control. Ползунок на хосте. "
            "audio.action not wired — вход не переключается."
        ),
        FALSE,
        FALSE,
        0
    );
    return make_flyout(relative, box);
}

static void on_chrome_destroy(GtkWidget *widget, gpointer user_data) {
    ProductChromeHud *hud = user_data;

    (void)widget;
    if (hud->shield_tick != 0) {
        g_source_remove(hud->shield_tick);
        hud->shield_tick = 0;
    }
}

void empiier_product_chrome_show(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->product_bar == NULL) {
        return;
    }
    empiier_widget_show_deferred(bridge->product_bar);
}

void empiier_product_chrome_hide(EmpiierNativeBridge *bridge) {
    (void)bridge;
    /* Always-on seat chrome: hide_product must not tuck the HUD. */
}

GtkWidget *empiier_product_chrome_new(EmpiierNativeBridge *bridge) {
    ProductChromeHud *hud = g_new0(ProductChromeHud, 1);
    GtkWidget *chrome = gtk_event_box_new();
    GtkWidget *cluster = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    GtkWidget *back = gtk_button_new();
    GtkWidget *elite = gtk_button_new();
    GtkWidget *panel = gtk_button_new();
    GtkWidget *shield = gtk_button_new();
    GtkWidget *ota = gtk_button_new();
    GtkWidget *volume = gtk_button_new();
    GtkWidget *grip = gtk_event_box_new();

    hud->bridge = bridge;
    hud->chrome = chrome;
    hud->cluster = cluster;
    apply_chrome_css();
    {
        GdkScreen *screen = gdk_screen_get_default();
        GdkVisual *visual = screen != NULL ? gdk_screen_get_rgba_visual(screen) : NULL;

        if (visual != NULL) {
            gtk_widget_set_visual(chrome, visual);
        }
    }
    gtk_widget_set_name(chrome, "empiieros-product-chrome");
    gtk_widget_set_name(cluster, "empiieros-product-cluster");
    gtk_widget_set_hexpand(chrome, FALSE);
    gtk_widget_set_vexpand(chrome, FALSE);
    gtk_widget_set_halign(chrome, GTK_ALIGN_END);
    gtk_widget_set_valign(chrome, GTK_ALIGN_CENTER);
    gtk_widget_set_no_show_all(chrome, TRUE);
    gtk_widget_set_app_paintable(chrome, TRUE);
    gtk_event_box_set_visible_window(GTK_EVENT_BOX(chrome), TRUE);
    gtk_widget_set_margin_end(chrome, CHROME_PIN_END);
    g_signal_connect(chrome, "size-allocate", G_CALLBACK(chrome_input_shape), hud);
    gtk_button_set_relief(GTK_BUTTON(back), GTK_RELIEF_NONE);
    gtk_button_set_relief(GTK_BUTTON(elite), GTK_RELIEF_NONE);
    gtk_button_set_relief(GTK_BUTTON(panel), GTK_RELIEF_NONE);
    gtk_button_set_relief(GTK_BUTTON(shield), GTK_RELIEF_NONE);
    gtk_button_set_relief(GTK_BUTTON(ota), GTK_RELIEF_NONE);
    gtk_button_set_relief(GTK_BUTTON(volume), GTK_RELIEF_NONE);
    gtk_widget_set_tooltip_text(back, "Лаунчер");
    gtk_widget_set_tooltip_text(elite, "EliteEW");
    gtk_widget_set_tooltip_text(panel, "Панель сиденья");
    gtk_widget_set_tooltip_text(shield, "Seat Shield");
    gtk_widget_set_tooltip_text(ota, "Обновления");
    gtk_widget_set_tooltip_text(volume, "Громкость ALSA");
    gtk_container_add(GTK_CONTAINER(back), media_image("launcher.png"));
    gtk_container_add(GTK_CONTAINER(elite), media_image("eliteew.png"));
    gtk_container_add(GTK_CONTAINER(panel), media_image("hud-panel.png"));
    hud->shield_image = media_image("hud-shield.png");
    gtk_container_add(GTK_CONTAINER(shield), hud->shield_image);
    hud_button_with_caption(ota, "hud-ota.png", "Обновления");
    gtk_container_add(GTK_CONTAINER(volume), media_image("hud-volume.png"));
    hud->elite_btn = elite;
    hud->panel_btn = panel;
    hud->shield_btn = shield;
    hud->ota_btn = ota;
    hud->volume_btn = volume;
    hud->grip = grip;
    hud->seat_licensed = FALSE;
    gtk_widget_set_name(grip, "empiieros-hud-grip");
    gtk_event_box_set_visible_window(GTK_EVENT_BOX(grip), TRUE);
    gtk_widget_set_size_request(grip, -1, 14);
    gtk_widget_set_tooltip_text(grip, "Перетащить панель");
    gtk_widget_add_events(
        grip,
        GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_BUTTON1_MOTION_MASK
    );
    gtk_box_pack_start(GTK_BOX(cluster), grip, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), back, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), elite, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), panel, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), shield, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), ota, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(cluster), volume, FALSE, FALSE, 0);
    gtk_container_add(GTK_CONTAINER(chrome), cluster);
    hud->flyout[HUD_FLY_LAUNCHER] = build_launcher_flyout(hud, back);
    hud->flyout[HUD_FLY_ELITE] = build_elite_flyout(hud, elite);
    hud->flyout[HUD_FLY_PANEL] = build_panel(hud, panel);
    hud->flyout[HUD_FLY_SHIELD] = build_shield_flyout(hud, shield);
    hud->flyout[HUD_FLY_OTA] = build_ota_flyout(hud, ota);
    hud->flyout[HUD_FLY_VOLUME] = build_volume_flyout(hud, volume);
    g_signal_connect(back, "clicked", G_CALLBACK(on_back_clicked), hud);
    g_signal_connect(elite, "clicked", G_CALLBACK(on_eliteew_clicked), hud);
    g_signal_connect(panel, "clicked", G_CALLBACK(on_panel_clicked), hud);
    g_signal_connect(shield, "clicked", G_CALLBACK(on_shield_icon_clicked), hud);
    g_signal_connect(ota, "clicked", G_CALLBACK(on_ota_icon_clicked), hud);
    g_signal_connect(volume, "clicked", G_CALLBACK(on_volume_clicked), hud);
    g_signal_connect(grip, "button-press-event", G_CALLBACK(on_grip_press), hud);
    g_signal_connect(grip, "motion-notify-event", G_CALLBACK(on_grip_motion), hud);
    g_signal_connect(grip, "button-release-event", G_CALLBACK(on_grip_release), hud);
    g_signal_connect(chrome, "map", G_CALLBACK(on_chrome_mapped), hud);
    g_signal_connect(chrome, "destroy", G_CALLBACK(on_chrome_destroy), hud);
    hud->shield_tick = g_timeout_add_seconds(12, on_shield_tick, hud);
    apply_seat_license_lock(hud);
    refresh_launcher(hud);
    g_object_set_data_full(G_OBJECT(chrome), "empiieros-product-hud", hud, g_free);
    syslog(LOG_INFO, "product chrome attached always-on seat chrome movable no-peek");
    return chrome;
}
