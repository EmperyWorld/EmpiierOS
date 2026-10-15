#ifndef EMPIIEROS_MEET_BOUNDS_H
#define EMPIIEROS_MEET_BOUNDS_H

#include "native_bridge.h"

gboolean empiier_meet_is_op(const char *text, const char *op);

gboolean empiier_meet_json_url(const char *text, char *out, size_t out_n);

/* Optional meetPost id. Digits only. Empty out if absent. */
gboolean empiier_meet_json_id(const char *text, char *out, size_t out_n);

gboolean empiier_meet_json_bounds(
    const char *text,
    double *x,
    double *y,
    double *w,
    double *h
);

/* -1 invalid, 0 hidden (too small), 1 ready to show. */
int empiier_meet_apply_bounds(
    EmpiierNativeBridge *bridge,
    double x,
    double y,
    double w,
    double h
);

#endif
