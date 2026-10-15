#include "meet_bounds.h"

#include <stdlib.h>
#include <string.h>
#include <syslog.h>

#define MEET_MIN 48
#define MEET_MAX 8192

static gboolean meet_finite(double v) {
    return v == v && v < 1.0e12 && v > -1.0e12;
}

static int meet_iround(double v) {
    if (v >= 0.0) {
        return (int)(v + 0.5);
    }
    return (int)(v - 0.5);
}

static gboolean json_num(const char *json, const char *key, double *out) {
    char needle[24];
    const char *p;
    char *end;

    if (json == NULL || key == NULL || out == NULL) {
        return FALSE;
    }
    if (g_snprintf(needle, sizeof(needle), "\"%s\":", key) < 0) {
        return FALSE;
    }
    p = strstr(json, needle);
    if (p == NULL) {
        return FALSE;
    }
    p += strlen(needle);
    while (*p == ' ' || *p == '\t') {
        p++;
    }
    *out = g_ascii_strtod(p, &end);
    return end != p && meet_finite(*out);
}

gboolean empiier_meet_is_op(const char *text, const char *op) {
    char needle[32];

    if (text == NULL || op == NULL || strchr(op, '"') != NULL) {
        return FALSE;
    }
    if (g_snprintf(needle, sizeof(needle), "\"op\":\"%s\"", op) < 0) {
        return FALSE;
    }
    return strstr(text, needle) != NULL;
}

gboolean empiier_meet_json_id(const char *text, char *out, size_t out_n) {
    const char *p;
    size_t i;

    if (text == NULL || out == NULL || out_n < 2) {
        return FALSE;
    }
    out[0] = '\0';
    p = strstr(text, "\"id\":\"");
    if (p == NULL) {
        return FALSE;
    }
    p += 6;
    i = 0;
    while (*p != '\0' && *p != '"' && i + 1 < out_n) {
        if (*p < '0' || *p > '9') {
            return FALSE;
        }
        out[i++] = *p++;
    }
    if (*p != '"' || i == 0) {
        return FALSE;
    }
    out[i] = '\0';
    return TRUE;
}

gboolean empiier_meet_json_url(const char *text, char *out, size_t out_n) {
    const char *p;
    size_t i;

    if (text == NULL || out == NULL || out_n < 8) {
        return FALSE;
    }
    p = strstr(text, "\"url\":\"");
    if (p == NULL) {
        return FALSE;
    }
    p += 7;
    i = 0;
    while (*p != '\0' && *p != '"' && i + 1 < out_n) {
        if (*p == '\\') {
            return FALSE;
        }
        out[i++] = *p++;
    }
    if (*p != '"' || i == 0) {
        return FALSE;
    }
    out[i] = '\0';
    return TRUE;
}

gboolean empiier_meet_json_bounds(
    const char *text,
    double *x,
    double *y,
    double *w,
    double *h
) {
    return json_num(text, "x", x)
        && json_num(text, "y", y)
        && json_num(text, "width", w)
        && json_num(text, "height", h);
}

int empiier_meet_apply_bounds(
    EmpiierNativeBridge *bridge,
    double x,
    double y,
    double w,
    double h
) {
    int ix;
    int iy;
    int iw;
    int ih;
    GtkAllocation host;

    if (bridge == NULL || bridge->meet_box == NULL || bridge->meet_view == NULL) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_unavailable");
        return -1;
    }
    if (!meet_finite(x) || !meet_finite(y) || !meet_finite(w) || !meet_finite(h)) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_invalid");
        return -1;
    }
    if (w < (double)MEET_MIN || h < (double)MEET_MIN) {
        syslog(LOG_INFO, "fofline meet stage request kind=bounds code=hidden");
        empiier_native_bridge_conceal_meet(bridge);
        return 0;
    }
    if (w > (double)MEET_MAX || h > (double)MEET_MAX
            || x < (double)-MEET_MAX || y < (double)-MEET_MAX) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_invalid");
        return -1;
    }
    ix = meet_iround(x);
    iy = meet_iround(y);
    iw = meet_iround(w);
    ih = meet_iround(h);
    if (bridge->web_view != NULL) {
        gtk_widget_get_allocation(GTK_WIDGET(bridge->web_view), &host);
        if (host.width > 0 && host.height > 0) {
            if (ix < 0) {
                iw += ix;
                ix = 0;
            }
            if (iy < 0) {
                ih += iy;
                iy = 0;
            }
            if (ix + iw > host.width) {
                iw = host.width - ix;
            }
            if (iy + ih > host.height) {
                ih = host.height - iy;
            }
        }
    }
    if (iw < MEET_MIN || ih < MEET_MIN) {
        syslog(LOG_INFO, "fofline meet stage request kind=bounds code=hidden");
        empiier_native_bridge_conceal_meet(bridge);
        return 0;
    }
    gtk_widget_set_halign(bridge->meet_box, GTK_ALIGN_START);
    gtk_widget_set_valign(bridge->meet_box, GTK_ALIGN_START);
    gtk_widget_set_hexpand(bridge->meet_box, FALSE);
    gtk_widget_set_vexpand(bridge->meet_box, FALSE);
    gtk_widget_set_margin_start(bridge->meet_box, ix);
    gtk_widget_set_margin_top(bridge->meet_box, iy);
    gtk_widget_set_margin_end(bridge->meet_box, 0);
    gtk_widget_set_margin_bottom(bridge->meet_box, 0);
    gtk_widget_set_size_request(bridge->meet_box, iw, ih);
    gtk_widget_set_size_request(GTK_WIDGET(bridge->meet_view), iw, ih);
    gtk_widget_queue_resize(bridge->meet_box);
    bridge->meet_has_bounds = TRUE;
    syslog(LOG_INFO, "fofline meet stage request kind=bounds");
    return 1;
}
