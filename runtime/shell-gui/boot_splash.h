#ifndef EMPIIEROS_BOOT_SPLASH_H
#define EMPIIEROS_BOOT_SPLASH_H

#include "native_bridge.h"

#include <gtk/gtk.h>

gboolean empiier_boot_splash_attach(GtkOverlay *overlay, EmpiierNativeBridge *bridge);

#endif
