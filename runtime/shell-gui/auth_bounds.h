#ifndef EMPIIEROS_AUTH_BOUNDS_H
#define EMPIIEROS_AUTH_BOUNDS_H

#include "native_bridge.h"

/* FILL modal host + centered Keycloak card. Not a second GtkWindow. */
GtkWidget *empiier_auth_card_build(
    EmpiierNativeBridge *bridge,
    WebKitWebView *auth_view
);

void empiier_auth_card_apply(EmpiierNativeBridge *bridge);

#endif
