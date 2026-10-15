#include "meet_bounds.h"
#include "native_bridge.h"

#include <stdio.h>
#include <string.h>
#include <syslog.h>

#define MEET_HOST "meet.eliteew.ru"
#define MEET_HOST_LEN 15
_Static_assert(sizeof(MEET_HOST) - 1 == MEET_HOST_LEN, "meet host length");
#define MEET_URL_MAX 4096
#define HOSTS_PATH "/etc/hosts"

gboolean empiier_native_bridge_meet_host_ok(const char *uri) {
    const char *rest;
    const char *end;
    size_t n;

    if (uri == NULL || !g_str_has_prefix(uri, "https://")) {
        return FALSE;
    }
    rest = uri + 8;
    end = rest;
    while (*end != '\0' && *end != '/' && *end != '?' && *end != '#') {
        end++;
    }
    n = (size_t)(end - rest);
    if (n == MEET_HOST_LEN && strncmp(rest, MEET_HOST, MEET_HOST_LEN) == 0) {
        return TRUE;
    }
    return n > (MEET_HOST_LEN + 1)
        && rest[n - MEET_HOST_LEN - 1] == '.'
        && strncmp(rest + n - MEET_HOST_LEN, MEET_HOST, MEET_HOST_LEN) == 0;
}

static gboolean meet_jwt_ok(const char *uri) {
    const char *query;
    const char *jwt;
    size_t n;

    if (uri == NULL) {
        return FALSE;
    }
    query = strchr(uri, '?');
    if (query == NULL) {
        return FALSE;
    }
    jwt = strstr(query, "jwt=");
    if (jwt == NULL) {
        return FALSE;
    }
    jwt += 4;
    if (*jwt == '\0' || *jwt == '&' || *jwt == '#') {
        return FALSE;
    }
    /* Hash may contain `&config.`; do not count the fragment as JWT. */
    n = 0;
    while (jwt[n] != '\0' && jwt[n] != '&' && jwt[n] != '#') {
        n++;
    }
    return n >= 16 && n < 2048;
}

static gboolean meet_join_ok(const char *uri) {
    if (uri == NULL || strlen(uri) > MEET_URL_MAX) {
        return FALSE;
    }
    if (strchr(uri, ' ') != NULL || strchr(uri, '\n') != NULL || strchr(uri, '\r') != NULL) {
        return FALSE;
    }
    return empiier_native_bridge_meet_host_ok(uri) && meet_jwt_ok(uri);
}

static gboolean meet_hostmap_ready(void) {
    FILE *file;
    char line[256];
    int saw_vip = 0;
    int saw_meet = 0;

    file = fopen(HOSTS_PATH, "r");
    if (file == NULL) {
        return FALSE;
    }
    while (fgets(line, sizeof(line), file) != NULL) {
        if (line[0] == '#') {
            continue;
        }
        if (strstr(line, "10.88.89.1") != NULL && strstr(line, MEET_HOST) != NULL) {
            fclose(file);
            return TRUE;
        }
        if (strstr(line, "10.88.89.1") != NULL) {
            saw_vip = 1;
        }
        if (strstr(line, MEET_HOST) != NULL) {
            saw_meet = 1;
        }
    }
    fclose(file);
    return saw_vip && saw_meet;
}

static char meet_reply_id[16];

static void meet_resolve(EmpiierNativeBridge *bridge, const char *json) {
    char *script;

    if (bridge == NULL || bridge->web_view == NULL || json == NULL) {
        return;
    }
    if (meet_reply_id[0] != '\0') {
        script = g_strdup_printf(
            "if(window.__empiierosMeetDone)window.__empiierosMeetDone("
            "Object.assign(%s,{id:'%s'}));",
            json,
            meet_reply_id
        );
    } else {
        script = g_strdup_printf(
            "if(window.__empiierosMeetDone)window.__empiierosMeetDone(%s);",
            json
        );
    }
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-meet-stage",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
}

void empiier_native_bridge_conceal_meet(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->meet_box == NULL) {
        return;
    }
    empiier_overlay_park(bridge, bridge->meet_box, GTK_WIDGET(bridge->meet_view));
    bridge->meet_visible = FALSE;
    syslog(LOG_INFO, "fofline meet stage concealed");
}

void empiier_native_bridge_hide_meet(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->meet_box == NULL) {
        return;
    }
    empiier_native_bridge_conceal_meet(bridge);
    if (bridge->meet_view != NULL) {
        /* about:blank keeps the WebProcess; ICE/XMPP stay. Terminate
         * first so the next join cannot reuse Jicofo session-invalid. */
        webkit_web_view_terminate_web_process(bridge->meet_view);
        webkit_web_view_load_uri(bridge->meet_view, "about:blank");
        syslog(LOG_INFO, "fofline meet stage process recycled");
    }
    bridge->meet_has_bounds = FALSE;
    syslog(LOG_INFO, "fofline meet stage hidden");
}

gboolean empiier_native_bridge_show_meet(EmpiierNativeBridge *bridge, const char *url) {
    if (bridge == NULL || bridge->meet_view == NULL || bridge->meet_box == NULL) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_unavailable");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_unavailable\"}");
        return FALSE;
    }
    if (!bridge->meet_has_bounds) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_required");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_required\"}");
        return FALSE;
    }
    if (!meet_join_ok(url)) {
        syslog(LOG_ERR, "fofline meet stage denied code=join_url_denied");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"join_url_denied\"}");
        return FALSE;
    }
    if (!meet_hostmap_ready()) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_hostmap_unavailable");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_hostmap_unavailable\"}");
        return FALSE;
    }
    if (bridge->browser_visible) {
        empiier_native_bridge_hide_seat_browser(bridge);
    }
    if (bridge->auth_visible) {
        empiier_native_bridge_hide_auth(bridge);
    }
    webkit_web_view_load_uri(bridge->meet_view, url);
    empiier_overlay_raise(bridge, bridge->meet_box, GTK_WIDGET(bridge->meet_view));
    bridge->meet_visible = TRUE;
    syslog(LOG_INFO, "fofline meet stage shown host=meet.eliteew.ru");
    meet_resolve(bridge, "{\"ok\":true}");
    return TRUE;
}

gboolean empiier_native_bridge_meet_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
) {
    WebKitNavigationPolicyDecision *nav;
    WebKitNavigationAction *action;
    WebKitURIRequest *request;
    const char *uri;

    (void)user_data;
    (void)web_view;
    if (type != WEBKIT_POLICY_DECISION_TYPE_NAVIGATION_ACTION
        && type != WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        return FALSE;
    }
    nav = WEBKIT_NAVIGATION_POLICY_DECISION(decision);
    action = webkit_navigation_policy_decision_get_navigation_action(nav);
    request = webkit_navigation_action_get_request(action);
    uri = webkit_uri_request_get_uri(request);
    if (g_strcmp0(uri, "about:blank") == 0 || g_str_has_prefix(uri, "about:blank?")) {
        webkit_policy_decision_use(decision);
        return TRUE;
    }
    if (!empiier_native_bridge_meet_host_ok(uri)) {
        webkit_policy_decision_ignore(decision);
        syslog(LOG_INFO, "fofline meet navigation denied");
        return TRUE;
    }
    if (type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        webkit_policy_decision_ignore(decision);
        syslog(LOG_INFO, "fofline meet new-window ignored");
        return TRUE;
    }
    webkit_policy_decision_use(decision);
    return TRUE;
}

gboolean empiier_native_bridge_meet_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
) {
    const char *uri;

    (void)user_data;
    uri = webkit_web_view_get_uri(web_view);
    if (empiier_native_bridge_meet_host_ok(uri)
        && (WEBKIT_IS_USER_MEDIA_PERMISSION_REQUEST(request)
            || WEBKIT_IS_DEVICE_INFO_PERMISSION_REQUEST(request))) {
        webkit_permission_request_allow(request);
        syslog(LOG_INFO, "fofline meet media permission allowed");
        return TRUE;
    }
    webkit_permission_request_deny(request);
    return TRUE;
}

gboolean empiier_native_bridge_product_meet_permission(
    WebKitWebView *web_view,
    WebKitPermissionRequest *request,
    gpointer user_data
) {
    const char *uri;

    (void)user_data;
    uri = webkit_web_view_get_uri(web_view);
    if (!(WEBKIT_IS_USER_MEDIA_PERMISSION_REQUEST(request)
            || WEBKIT_IS_DEVICE_INFO_PERMISSION_REQUEST(request))) {
        webkit_permission_request_deny(request);
        return TRUE;
    }
    if (empiier_native_bridge_meet_host_ok(uri)) {
        webkit_permission_request_allow(request);
        syslog(LOG_INFO, "fofline product media permission allowed code=meet_host");
        return TRUE;
    }
    /* Iframe getUserMedia is reported on the product WebView (fofline-spa). */
    if (empiier_native_bridge_fofline_uri(uri)) {
        webkit_permission_request_allow(request);
        syslog(LOG_INFO, "fofline product media permission allowed code=fofline_iframe");
        return TRUE;
    }
    webkit_permission_request_deny(request);
    return TRUE;
}

static void on_meet_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *js_result,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    JSCValue *value;
    char *text;
    char url[MEET_URL_MAX];
    double x = 0.0;
    double y = 0.0;
    double w = 0.0;
    double h = 0.0;
    const char *uri;
    int clip;

    (void)manager;
    if (bridge == NULL || bridge->web_view == NULL) {
        return;
    }
    if (!empiier_native_bridge_fofline_uri(webkit_web_view_get_uri(bridge->web_view))) {
        syslog(LOG_ERR, "fofline meet bridge denied origin");
        return;
    }
    value = webkit_javascript_result_get_js_value(js_result);
    text = value != NULL && jsc_value_is_string(value) ? jsc_value_to_string(value) : NULL;
    if (text == NULL) {
        syslog(LOG_ERR, "fofline meet stage denied code=join_url_denied");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"join_url_denied\"}");
        return;
    }
    meet_reply_id[0] = '\0';
    (void)empiier_meet_json_id(text, meet_reply_id, sizeof meet_reply_id);
    if (g_strcmp0(text, "close") == 0 || empiier_meet_is_op(text, "close")) {
        syslog(LOG_INFO, "fofline meet stage request kind=close");
        empiier_native_bridge_hide_meet(bridge);
        meet_resolve(bridge, "{\"ok\":true}");
        g_free(text);
        return;
    }
    if (g_str_has_prefix(text, "https://")) {
        syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_required");
        meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_required\"}");
        g_free(text);
        return;
    }
    if (empiier_meet_is_op(text, "bounds")) {
        if (!empiier_meet_json_bounds(text, &x, &y, &w, &h)) {
            syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_invalid");
            meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_invalid\"}");
            g_free(text);
            return;
        }
        uri = webkit_web_view_get_uri(bridge->meet_view);
        if (!bridge->meet_visible && !empiier_native_bridge_meet_host_ok(uri)) {
            syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_not_open");
            meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_not_open\"}");
            g_free(text);
            return;
        }
        clip = empiier_meet_apply_bounds(bridge, x, y, w, h);
        if (clip < 0) {
            meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_invalid\"}");
            g_free(text);
            return;
        }
        if (clip > 0 && empiier_native_bridge_meet_host_ok(uri)) {
            empiier_overlay_raise(bridge, bridge->meet_box, GTK_WIDGET(bridge->meet_view));
            bridge->meet_visible = TRUE;
        }
        meet_resolve(bridge, "{\"ok\":true}");
        g_free(text);
        return;
    }
    if (empiier_meet_is_op(text, "open")) {
        if (!empiier_meet_json_url(text, url, sizeof(url))
                || !empiier_meet_json_bounds(text, &x, &y, &w, &h)) {
            syslog(LOG_ERR, "fofline meet stage denied code=meet_stage_bounds_required");
            meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_required\"}");
            g_free(text);
            return;
        }
        syslog(LOG_INFO, "fofline meet stage request kind=open");
        clip = empiier_meet_apply_bounds(bridge, x, y, w, h);
        if (clip < 0) {
            meet_resolve(bridge, "{\"ok\":false,\"code\":\"meet_stage_bounds_invalid\"}");
            g_free(text);
            return;
        }
        if (clip == 0) {
            meet_resolve(bridge, "{\"ok\":true}");
            g_free(text);
            return;
        }
        empiier_native_bridge_show_meet(bridge, url);
        g_free(text);
        return;
    }
    syslog(LOG_ERR, "fofline meet stage denied code=join_url_denied");
    meet_resolve(bridge, "{\"ok\":false,\"code\":\"join_url_denied\"}");
    g_free(text);
}

static void on_meet_diag(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *js_result,
    gpointer user_data
) {
    JSCValue *value;
    char *text;
    size_t n;
    size_t i;

    (void)manager;
    (void)user_data;
    value = webkit_javascript_result_get_js_value(js_result);
    text = value != NULL && jsc_value_is_string(value) ? jsc_value_to_string(value) : NULL;
    if (text == NULL) {
        return;
    }
    n = strlen(text);
    if (n > 480) {
        text[480] = '\0';
        n = 480;
    }
    for (i = 0; i < n; i++) {
        if (text[i] < 0x20 && text[i] != '\t') {
            text[i] = ' ';
        }
    }
    if (strstr(text, "jwt=") != NULL || strstr(text, "JWT=") != NULL) {
        syslog(LOG_WARNING, "fofline meet diag redacted");
        g_free(text);
        return;
    }
    syslog(LOG_WARNING, "fofline meet diag %s", text);
    g_free(text);
}

void empiier_meet_register_diag(WebKitUserContentManager *manager) {
    static const char js[] =
        "(function(){"
        "function send(k,m){try{var s=String(m==null?'':m);"
        "s=s.replace(/jwt=[^&#\\s]*/gi,'jwt=redacted');"
        "if(s.length>480)s=s.slice(0,480);"
        "if(window.webkit&&webkit.messageHandlers&&"
        "webkit.messageHandlers.empiierosMeetDiag)"
        "webkit.messageHandlers.empiierosMeetDiag.postMessage(k+' '+s);"
        "}catch(e){}}"
        "window.addEventListener('error',function(ev){"
        "send('error',(ev&&ev.message)||'error');});"
        "window.addEventListener('unhandledrejection',function(ev){"
        "var r=ev&&ev.reason;send('reject',r&&(r.message||r));});"
        "var ce=console.error;"
        "console.error=function(){"
        "send('cerr',Array.prototype.join.call(arguments,' '));"
        "return ce.apply(console,arguments);};"
        "var OrigPC=window.RTCPeerConnection;"
        "if(!OrigPC){send('nowebrtc','no RTCPeerConnection');return;}"
        "function ensureTx(pc){"
        "if(!pc||pc.__eoTx)return;"
        "try{"
        "var t=pc.getTransceivers&&pc.getTransceivers();"
        "if(t&&t.length){pc.__eoTx=1;return;}"
        "pc.addTransceiver('audio',{direction:'sendrecv'});"
        "pc.addTransceiver('video',{direction:'sendrecv'});"
        "pc.__eoTx=1;"
        "send('tx','addTransceiver sendrecv audio,video');"
        "}catch(e){send('tx',e&&(e.message||e));}}"
        "function WrappedPC(c){"
        "var pc=c===undefined?new OrigPC():new OrigPC(c);"
        "ensureTx(pc);return pc;}"
        "WrappedPC.prototype=OrigPC.prototype;"
        "try{Object.keys(OrigPC).forEach(function(k){WrappedPC[k]=OrigPC[k];});}"
        "catch(e){}"
        "if(typeof OrigPC.generateCertificate==='function')"
        "WrappedPC.generateCertificate=OrigPC.generateCertificate.bind(OrigPC);"
        "window.RTCPeerConnection=WrappedPC;"
        "var Proto=OrigPC.prototype;"
        "function lowerCodecs(sdp){"
        "return String(sdp||'').replace(/^(a=rtpmap:\\d+\\s+)([^\\s/]+)/gm,"
        "function(m,a,c){return a+c.toLowerCase();});}"
        "function fixDesc(d){if(!d||typeof d.sdp!=='string')return d;"
        "var s=lowerCodecs(d.sdp);if(s===d.sdp)return d;"
        "try{return new RTCSessionDescription({type:d.type,sdp:s});}"
        "catch(e){d.sdp=s;return d;}}"
        "function wrap(name){var orig=Proto[name];"
        "if(typeof orig!=='function')return;"
        "Proto[name]=function(){"
        "var args=Array.prototype.slice.call(arguments);"
        "if(name==='createAnswer')ensureTx(this);"
        "if(name==='setLocalDescription'||name==='setRemoteDescription')"
        "args[0]=fixDesc(args[0]);"
        "try{var ret=orig.apply(this,args);"
        "if(ret&&typeof ret.then==='function'){"
        "return ret.then(function(v){"
        "if(name==='createAnswer'||name==='createOffer'){"
        "v=fixDesc(v);var s=(v&&v.sdp)||'';"
        "if(!this.__eoSdpLog){this.__eoSdpLog=1;"
        "send(name,(s.indexOf('a=rtpmap:111 opus')>=0?'opus':'OPUS')+' '+"
        "(s.indexOf('a=ssrc:')>=0?'ssrc':'nossrc')+' '+"
        "((s.match(/a=(sendrecv|recvonly|sendonly)/g)||[]).join(',')));}}"
        "return v;}.bind(this),function(err){"
        "send(name,err&&(err.message||err));throw err;});}"
        "return ret;}catch(err){send(name,err&&(err.message||err));throw err;}};}"
        "wrap('setRemoteDescription');wrap('setLocalDescription');"
        "wrap('createAnswer');wrap('createOffer');wrap('addIceCandidate');"
        "function wrapGet(name){"
        "var d=Object.getOwnPropertyDescriptor(Proto,name);"
        "if(!d||typeof d.get!=='function')return;"
        "var get=d.get;"
        "Object.defineProperty(Proto,name,{configurable:true,"
        "enumerable:!!d.enumerable,get:function(){return fixDesc(get.call(this));}});}"
        "wrapGet('localDescription');wrapGet('remoteDescription');"
        "wrapGet('currentLocalDescription');wrapGet('currentRemoteDescription');"
        "wrapGet('pendingLocalDescription');wrapGet('pendingRemoteDescription');"
        "send('pcok','RTCPeerConnection');"
        "})();";
    WebKitUserScript *script;

    if (manager == NULL) {
        return;
    }
    g_signal_connect(
        manager,
        "script-message-received::empiierosMeetDiag",
        G_CALLBACK(on_meet_diag),
        NULL
    );
    if (!webkit_user_content_manager_register_script_message_handler(
            manager,
            "empiierosMeetDiag"
        )) {
        g_error("failed to register empiierosMeetDiag script message handler");
    }
    script = webkit_user_script_new(
        js,
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
    webkit_user_content_manager_add_script(manager, script);
    webkit_user_script_unref(script);
}

void empiier_meet_register_product_bridge(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return;
    }
    g_signal_connect(
        bridge->manager,
        "script-message-received::empiierosMeet",
        G_CALLBACK(on_meet_message),
        bridge
    );
    if (!webkit_user_content_manager_register_script_message_handler(
            bridge->manager,
            "empiierosMeet"
        )) {
        g_error("failed to register empiierosMeet script message handler");
    }
}

gboolean empiier_native_bridge_meet_load_failed(
    WebKitWebView *web_view,
    WebKitLoadEvent load_event,
    const char *failing_uri,
    GError *error,
    gpointer user_data
) {
    (void)web_view;
    (void)load_event;
    (void)failing_uri;
    (void)user_data;
    syslog(
        LOG_ERR,
        "fofline meet load failed detail=%s",
        error != NULL ? error->message : "unknown"
    );
    return TRUE;
}

static gboolean on_meet_enter_fullscreen(
    WebKitWebView *web_view,
    gpointer user_data
) {
    (void)web_view;
    (void)user_data;
    syslog(LOG_INFO, "fofline meet stage fullscreen denied");
    return TRUE;
}

static const char *meet_term_reason(WebKitWebProcessTerminationReason reason) {
    switch (reason) {
    case WEBKIT_WEB_PROCESS_CRASHED:
        return "crashed";
    case WEBKIT_WEB_PROCESS_EXCEEDED_MEMORY_LIMIT:
        return "exceeded_memory";
    case WEBKIT_WEB_PROCESS_TERMINATED_BY_API:
        return "terminated_by_api";
    default:
        return "unknown";
    }
}

static void meet_notify_lost(EmpiierNativeBridge *bridge, const char *code) {
    char detail[96];

    if (bridge == NULL || code == NULL) {
        return;
    }
    if (strpbrk(code, "\"';\\") != NULL) {
        return;
    }
    syslog(LOG_ERR, "fofline meet stage crashed reason=%s", code);
    (void)snprintf(detail, sizeof(detail), "{\"code\":\"%s\"}", code);
    empiier_native_dispatch_custom_event(
        bridge,
        "empiieros-meet-stage-lost",
        detail
    );
}

static void on_meet_web_process_terminated(
    WebKitWebView *web_view,
    WebKitWebProcessTerminationReason reason,
    gpointer user_data
) {
    EmpiierNativeBridge *bridge = user_data;
    const char *why;
    gboolean was_visible;

    (void)web_view;
    if (bridge == NULL) {
        return;
    }
    why = meet_term_reason(reason);
    was_visible = bridge->meet_visible;
    syslog(
        LOG_ERR,
        "fofline meet stage process terminated reason=%s visible=%s",
        why,
        was_visible ? "1" : "0"
    );
    if (!was_visible) {
        return;
    }
    empiier_native_bridge_hide_meet(bridge);
    if (reason == WEBKIT_WEB_PROCESS_TERMINATED_BY_API) {
        return;
    }
    meet_notify_lost(bridge, "meet_process_crashed");
}

void empiier_native_bridge_attach_meet(
    EmpiierNativeBridge *bridge,
    GtkWidget *meet_box,
    WebKitWebView *meet_view
) {
    if (bridge == NULL) {
        return;
    }
    bridge->meet_box = meet_box;
    bridge->meet_view = meet_view;
    bridge->meet_visible = FALSE;
    bridge->meet_has_bounds = FALSE;
    if (meet_box != NULL) {
        gtk_widget_set_halign(meet_box, GTK_ALIGN_START);
        gtk_widget_set_valign(meet_box, GTK_ALIGN_START);
        gtk_widget_set_hexpand(meet_box, FALSE);
        gtk_widget_set_vexpand(meet_box, FALSE);
    }
    if (meet_view == NULL) {
        return;
    }
    g_signal_connect(
        meet_view,
        "enter-fullscreen",
        G_CALLBACK(on_meet_enter_fullscreen),
        bridge
    );
    g_signal_connect(
        meet_view,
        "web-process-terminated",
        G_CALLBACK(on_meet_web_process_terminated),
        bridge
    );
}
