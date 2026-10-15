#ifndef EMPIIEROS_FOFLINE_API_PROXY_H
#define EMPIIEROS_FOFLINE_API_PROXY_H

#include <webkit2/webkit2.h>

#include "native_bridge.h"

gboolean empiier_fofline_api_path(const char *path);
void empiier_fofline_api_proxy_bind(EmpiierNativeBridge *bridge);
gboolean empiier_fofline_video_call_end_path(const char *path);
void empiier_fofline_api_proxy(
    WebKitURISchemeRequest *request,
    const char *path,
    const char *query
);

#endif
