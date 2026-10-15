#define _POSIX_C_SOURCE 200809L

#include "auth_bounds.h"
#include "native_bridge.h"

#include <gio/gio.h>
#include <jsc/jsc.h>
#include <string.h>
#include <syslog.h>

static gboolean jwt_safe(const char *value) {
    size_t dots = 0U;
    size_t length = 0U;
    if (value == NULL) {
        return FALSE;
    }
    for (const char *cursor = value; *cursor != '\0'; cursor += 1) {
        char item = *cursor;
        length += 1U;
        if (item == '.') {
            dots += 1U;
            continue;
        }
        if (!g_ascii_isalnum(item) && item != '-' && item != '_') {
            return FALSE;
        }
    }
    return dots == 2U && length >= 16U && length <= 8192U;
}

static char *extract_json_string(const char *json, const char *key) {
    if (json == NULL || key == NULL) {
        return NULL;
    }
    char *needle = g_strdup_printf("\"%s\":\"", key);
    const char *found = strstr(json, needle);
    g_free(needle);
    if (found == NULL) {
        return NULL;
    }
    found += strlen(key) + 4U;
    GString *out = g_string_new(NULL);
    for (const char *cursor = found; *cursor != '\0'; cursor += 1) {
        if (*cursor == '\\' && cursor[1] != '\0') {
            cursor += 1;
            g_string_append_c(out, *cursor);
            continue;
        }
        if (*cursor == '"') {
            break;
        }
        g_string_append_c(out, *cursor);
    }
    return g_string_free(out, FALSE);
}

static gboolean auth_start_prefix(const char *uri) {
    return uri != NULL
        && (g_str_has_prefix(uri, EMPIIEROS_OIDC_AUTH)
            || g_str_has_prefix(
                uri,
                "https://auth.eliteew.ru:443/realms/empery/protocol/openid-connect/auth"
            ));
}

gboolean empiier_native_bridge_auth_url(const char *uri) {
    if (!auth_start_prefix(uri)) {
        return FALSE;
    }
    if (strstr(uri, "..") != NULL
        || strchr(uri, '\\') != NULL
        || strstr(uri, "code_challenge") != NULL
        || strstr(uri, "login_hint=") != NULL
        || strstr(uri, "ticket=") != NULL
        || strstr(uri, "eliteewLicenseId=") != NULL) {
        return FALSE;
    }
    if (strstr(uri, "client_id=empery-fofline") == NULL) {
        return FALSE;
    }
    return strstr(uri, "request_uri=urn:ietf:params:oauth:request_uri:") != NULL
        || strstr(uri, "request_uri=urn%3Aietf%3Aparams%3Aoauth%3Arequest_uri%3A") != NULL;
}

gboolean empiier_native_bridge_auth_callback(const char *uri) {
    if (uri == NULL || !g_str_has_prefix(uri, EMPIIEROS_OIDC_CALLBACK)) {
        return FALSE;
    }
    const char *query = strchr(uri, '?');
    return query != NULL
        && strstr(query, "code=") != NULL
        && strstr(query, "state=") != NULL
        && strstr(query, "error=") == NULL;
}

static gboolean auth_app_root_cancel(const char *uri) {
    if (uri == NULL || g_str_has_prefix(uri, EMPIIEROS_OIDC_CALLBACK)) {
        return FALSE;
    }
    if (strstr(uri, "code=") != NULL) {
        return FALSE;
    }
    return g_strcmp0(uri, "https://app.eliteew.ru") == 0
        || g_strcmp0(uri, "https://app.eliteew.ru/") == 0
        || g_str_has_prefix(uri, "https://app.eliteew.ru/?");
}

static gboolean auth_host_allowed(const char *uri) {
    if (g_strcmp0(uri, "about:blank") == 0) {
        return TRUE;
    }
    if (uri == NULL || strstr(uri, "@") != NULL) {
        return FALSE;
    }
    return g_str_has_prefix(uri, "https://auth.eliteew.ru/")
        || g_str_has_prefix(uri, "https://auth.eliteew.ru:443/")
        || g_strcmp0(uri, "https://auth.eliteew.ru") == 0
        || g_strcmp0(uri, "https://auth.eliteew.ru:443") == 0;
}

static void empiier_oidc_try_fresh_par(EmpiierNativeBridge *bridge, const char *reason);

WebKitUserScript *empiier_oidc_manual_login_script(void) {
    return webkit_user_script_new(
        "(function(){if(window.__empiierosManualLogin)return;window.__empiierosManualLogin=true;"
        "try{Object.defineProperty(window,'PublicKeyCredential',{configurable:true,enumerable:false,get:function(){return undefined;},set:function(){}});}catch(e){}"
        "function isForm(form){return !!(form&&form.tagName&&String(form.tagName).toLowerCase()==='form');}"
        "function login(form){return isForm(form)&&(form.id==='kc-form-login'||(form.querySelector(\"#username, input[name='username']\")&&form.querySelector(\"#password, input[name='password']\")));}"
        "function webauth(form){if(!isForm(form))return false;if(String(form.id||'').toLowerCase().indexOf('webauth')!==-1)return true;return !!(form.querySelector(\"input[name='clientDataJSON'], input[name='authenticatorData'], input[name='signature']\")&&!form.querySelector(\"#password, input[name='password']\"));}"
        "function otp(form){return !!(isForm(form)&&form.querySelector(\"input[name='otp']\")&&!form.querySelector(\"#password, input[name='password']\"));}"
        "function filled(form){var u=form.querySelector(\"#username, input[name='username']\");var p=form.querySelector(\"#password, input[name='password']\");return !!(u&&p&&String(u.value||'').trim()&&String(p.value||''));}"
        "function block(form){if(!isForm(form))return false;if(webauth(form))return true;if(login(form)&&!filled(form))return true;if(otp(form)){var o=form.querySelector(\"input[name='otp']\");return !(o&&String(o.value||'').trim());}if(login(form)||String(form.getAttribute('action')||'').indexOf('login-actions/authenticate')!==-1)return !filled(form)&&!otp(form);return false;}"
        "var orig=HTMLFormElement.prototype.submit;"
        "HTMLFormElement.prototype.submit=function(){if(block(this))return;return orig.apply(this,arguments);};"
        "if(typeof HTMLFormElement.prototype.requestSubmit==='function'){var origReq=HTMLFormElement.prototype.requestSubmit;HTMLFormElement.prototype.requestSubmit=function(){if(block(this))return;return origReq.apply(this,arguments);};}"
        "document.addEventListener('submit',function(ev){if(block(ev.target)){ev.preventDefault();ev.stopPropagation();}},true);})();",
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
}

gboolean empiier_native_bridge_auth_decide_policy(
    WebKitWebView *web_view,
    WebKitPolicyDecision *decision,
    WebKitPolicyDecisionType type,
    gpointer user_data
) {
    (void)web_view;
    EmpiierNativeBridge *bridge = user_data;
    if (type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        webkit_policy_decision_ignore(decision);
        return TRUE;
    }
    if (type != WEBKIT_POLICY_DECISION_TYPE_NAVIGATION_ACTION) {
        return FALSE;
    }
    WebKitNavigationAction *action = webkit_navigation_policy_decision_get_navigation_action(
        WEBKIT_NAVIGATION_POLICY_DECISION(decision)
    );
    WebKitURIRequest *request = webkit_navigation_action_get_request(action);
    const char *uri = webkit_uri_request_get_uri(request);
    if (uri != NULL
        && g_str_has_prefix(uri, EMPIIEROS_OIDC_CALLBACK)
        && strstr(uri, "error=") != NULL) {
        syslog(LOG_WARNING, "oidc auth callback error uri=%s", uri);
        empiier_native_bridge_hide_auth(bridge);
        empiier_native_dispatch_custom_event(bridge, "empiieros-oidc", "{\"ok\":false}");
        webkit_policy_decision_ignore(decision);
        return TRUE;
    }
    if (empiier_native_bridge_auth_callback(uri)) {
        syslog(LOG_INFO, "oidc auth callback captured");
        empiier_oidc_complete_from_callback(bridge, uri);
        webkit_policy_decision_ignore(decision);
        return TRUE;
    }
    if (auth_app_root_cancel(uri)) {
        syslog(LOG_INFO, "oidc auth back-to-app ignored origin=app.eliteew.ru");
        webkit_policy_decision_ignore(decision);
        empiier_oidc_try_fresh_par(bridge, "back_to_app");
        return TRUE;
    }
    if (auth_host_allowed(uri) || empiier_native_bridge_auth_url(uri)) {
        return FALSE;
    }
    syslog(LOG_WARNING, "oidc auth navigation denied uri=%s", uri != NULL ? uri : "");
    webkit_policy_decision_ignore(decision);
    return TRUE;
}

gboolean empiier_native_bridge_show_auth(
    EmpiierNativeBridge *bridge,
    const char *entrypoint
) {
    if (bridge == NULL || bridge->auth_view == NULL || bridge->auth_box == NULL) {
        return FALSE;
    }
    if (!empiier_native_bridge_auth_url(entrypoint)) {
        return FALSE;
    }
    empiier_native_bridge_hide_seat_browser(bridge);
    empiier_native_bridge_hide_meet(bridge);
    webkit_web_view_load_uri(bridge->auth_view, entrypoint);
    empiier_auth_card_apply(bridge);
    empiier_overlay_raise(bridge, bridge->auth_box, GTK_WIDGET(bridge->auth_view));
    bridge->auth_visible = TRUE;
    syslog(LOG_INFO, "auth surface show origin=auth.eliteew.ru kind=card");
    return TRUE;
}

void empiier_native_bridge_hide_auth(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->auth_box == NULL || bridge->auth_view == NULL) {
        return;
    }
    webkit_web_view_load_uri(bridge->auth_view, "about:blank");
    empiier_overlay_park(bridge, bridge->auth_box, GTK_WIDGET(bridge->auth_view));
    bridge->auth_visible = FALSE;
    syslog(LOG_INFO, "auth surface hide");
}

void empiier_oidc_clear_product(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return;
    }
    if (bridge->oidc_script != NULL) {
        webkit_user_content_manager_remove_all_scripts(bridge->manager);
        webkit_user_script_unref(bridge->oidc_script);
        bridge->oidc_script = NULL;
    }
}

/* Official pack axios still uses relative /api on fofline-spa. Keep
 * those URLs same-origin so the scheme proxy hops to the VIP. Absolute
 * app.eliteew.ru /api is folded back to the path. That bake has no
 * EmperyDesktopNativeOidc: init skips access-only storage, applyTokens
 * needs a refreshToken field, then refresh goes to backend SSO which
 * has no native session. Answer that fetch from the current access JWT
 * Runtime refresh is owned by product `empiierosOidc` → inject
 * `{force:true}` (living product_session_refresh). Do not set the mobile-embedded
 * client flag: living profile prep would pick empery-mobile over
 * empery-fofline. Product assign to Keycloak is swallowed;
 * decide_policy cannot finish in-SPA PKCE. Native login is the
 * operator confirm: arm the living SSO replace window so Socket.IO
 * sends sso_replace_confirm. The scheme proxy plants the HTTP header. */
static const char OIDC_INJECT_HEAD[] =
    "(function(){try{"
    "var href=String(window.location&&window.location.href||'');"
    "if(href.indexOf('fofline-spa:')!==0&&String(window.location.protocol||'')!=='fofline-spa:')return;"
    "try{sessionStorage.setItem('eliteew.foflineClient.v1','1');}catch(e){}"
    "try{sessionStorage.setItem('eliteew.auth.sessionReplaceConfirmedUntil.v1',String(Date.now()+900000));}catch(e){}"
    "var origAtob=window.atob.bind(window);"
    "window.atob=function(s){var bin=origAtob(s);"
    "if(bin.charAt(0)==='{'||bin.charAt(0)==='['){"
    "try{return decodeURIComponent(escape(bin));}catch(e){return bin;}}"
    "return bin;};"
    "function absApi(u){if(typeof u!=='string')return u;"
    "var app='https://app.eliteew.ru';"
    "if(u.indexOf(app+'/api/')===0)return u.slice(app.length);"
    "var http='http://app.eliteew.ru';"
    "if(u.indexOf(http+'/api/')===0)return u.slice(http.length);"
    "return u;}"
    "function accessOf(b){return b&&(b.token||b.accessToken);}"
    "function hostOidc(op){"
    "if(window.__empiierosOidcWait)return window.__empiierosOidcWait;"
    "window.__empiierosOidcWait=new Promise(function(resolve){"
    "window.__empiierosOidcDone=function(raw){"
    "window.__empiierosOidcWait=null;window.__empiierosOidcDone=null;resolve(raw);};"
    "try{window.webkit.messageHandlers.empiierosOidc.postMessage({op:String(op||'')});}"
    "catch(e){window.__empiierosOidcWait=null;window.__empiierosOidcDone=null;"
    "resolve(JSON.stringify({ok:false,error:'Authentication could not be completed'}));}"
    "});return window.__empiierosOidcWait;}"
    "function hostSsoRefresh(u){"
    "if(typeof u!=='string'||u.indexOf('/api/v1/auth/sso/refresh')<0)return null;"
    "return hostOidc('refresh').then(function(raw){"
    "var o={};try{o=JSON.parse(raw);}catch(e){o={};}"
    "var tok=accessOf(o);"
    "if(!tok)return new Response(JSON.stringify({ok:false}),{status:401,"
    "headers:{'content-type':'application/json'}});"
    "return new Response(JSON.stringify({access_token:tok,refresh_token:tok,id_token:o.idToken}),"
    "{status:200,headers:{'content-type':'application/json'}});});}"
    "var rawFetch=window.fetch;"
    "window.fetch=function(input,init){"
    "if(typeof input==='string'){var nu=absApi(input);var fake=hostSsoRefresh(nu);"
    "if(fake)return Promise.resolve(fake);if(nu!==input)return rawFetch.call(this,nu,init);}"
    "else if(input&&typeof input.url==='string'){var nu2=absApi(input.url);var fake2=hostSsoRefresh(nu2);"
    "if(fake2)return Promise.resolve(fake2);if(nu2!==input.url)return rawFetch.call(this,nu2,init);}"
    "return rawFetch.apply(this,arguments);};"
    "var rawOpen=XMLHttpRequest.prototype.open;"
    "XMLHttpRequest.prototype.open=function(method,url){"
    "if(typeof url==='string')arguments[1]=absApi(url);"
    "return rawOpen.apply(this,arguments);};"
    "try{var rawAssign=window.location.assign.bind(window.location);"
    "window.location.assign=function(u){var s=String(u||'');"
    "if(s.indexOf('auth.eliteew.ru')>=0||s.indexOf('fofline-oidc-bridge')>=0)return;"
    "return rawAssign(u);};}catch(e){}";

static const char OIDC_INJECT_TAIL[] =
    "try{sessionStorage.setItem('eliteew.keycloak.foflineNative.oidcTokens.v1',"
    "JSON.stringify({token:bootstrap.token,refreshToken:bootstrap.token,idToken:bootstrap.idToken,savedAt:Date.now(),persistence:'session'}));}catch(e){}"
    "window.__ELITEEW_PRODUCT_SESSION_BOOTSTRAP__=bootstrap;"
    "window.EmperyDesktopNativeOidc={"
    "getBootstrapJson:function(){return JSON.stringify(window.__ELITEEW_PRODUCT_SESSION_BOOTSTRAP__||{ok:false});},"
    "refreshSessionJson:function(){return hostOidc('refresh');},"
    "logoutSessionJson:function(){return hostOidc('logout');}"
    "};"
    "window.__empiierosMeetWait=window.__empiierosMeetWait||{};"
    "window.__empiierosMeetSeq=window.__empiierosMeetSeq||0;"
    "window.__empiierosMeetDone=function(r){"
    "var rid=r&&r.id!=null?String(r.id):'';"
    "var fn=rid?window.__empiierosMeetWait[rid]:null;"
    "if(!fn)return;"
    "delete window.__empiierosMeetWait[rid];"
    "fn(r||{ok:false,code:'meet_stage_failed'});};"
    "function meetPost(o){"
    "return new Promise(function(resolve){"
    "var id=String(++window.__empiierosMeetSeq);"
    "o.id=id;"
    "var t=setTimeout(function(){"
    "if(!window.__empiierosMeetWait[id])return;"
    "delete window.__empiierosMeetWait[id];"
    "resolve({ok:false,code:'meet_stage_failed'});},2500);"
    "window.__empiierosMeetWait[id]=function(r){clearTimeout(t);resolve(r);};"
    "try{window.webkit.messageHandlers.empiierosMeet.postMessage(JSON.stringify(o));}"
    "catch(e){clearTimeout(t);delete window.__empiierosMeetWait[id];"
    "resolve({ok:false,code:'meet_stage_unavailable'});}});}"
    "function meetBoundsOk(b){"
    "return b&&isFinite(b.x)&&isFinite(b.y)&&isFinite(b.width)&&isFinite(b.height);}"
    "window.EmperyDesktopMeetStage={"
    "openMeetStage:function(u,b){"
    "var s=String(u||'');"
    "if(s.indexOf('https://meet.eliteew.ru')!==0){"
    "return Promise.resolve({ok:false,code:'join_url_denied'});}"
    "var n=s.charAt(23);"
    "if(s.length>23&&n!=='/'&&n!=='?'&&n!=='#'){"
    "return Promise.resolve({ok:false,code:'join_url_denied'});}"
    "if(!meetBoundsOk(b)){"
    "return Promise.resolve({ok:false,code:'meet_stage_bounds_required'});}"
    "return meetPost({op:'open',url:s,x:b.x,y:b.y,width:b.width,height:b.height});},"
    "setMeetStageBounds:function(b){"
    "if(!meetBoundsOk(b)){"
    "return Promise.resolve({ok:false,code:'meet_stage_bounds_required'});}"
    "return meetPost({op:'bounds',x:b.x,y:b.y,width:b.width,height:b.height});},"
    "closeMeetStage:function(){"
    "return meetPost({op:'close'});}"
    "};"
    "window.__empiierosNetworkWait=null;"
    "function hostNetworkLink(){"
    "if(window.__empiierosNetworkWait)return window.__empiierosNetworkWait;"
    "window.__empiierosNetworkWait=new Promise(function(resolve){"
    "var t=setTimeout(function(){"
    "if(!window.__empiierosNetworkWait)return;"
    "window.__empiierosNetworkWait=null;"
    "resolve({ok:false,kind:null,active:false});},6000);"
    "window.__empiierosNetworkDone=function(r){"
    "clearTimeout(t);window.__empiierosNetworkWait=null;"
    "resolve(r||{ok:false,kind:null,active:false});};"
    "try{window.webkit.messageHandlers.empiierosNetwork.postMessage('snapshot');}"
    "catch(e){clearTimeout(t);window.__empiierosNetworkWait=null;"
    "resolve({ok:false,kind:null,active:false});}});"
    "return window.__empiierosNetworkWait;}"
    "window.EmperyDesktopNetworkLink={"
    "getLinkSnapshot:function(){return hostNetworkLink();}"
    "};"
    "window.__empiierosSeatBrowserWait=window.__empiierosSeatBrowserWait||{};"
    "window.__empiierosSeatBrowserSeq=window.__empiierosSeatBrowserSeq||0;"
    "window.__empiierosSeatBrowserDone=function(r){"
    "var rid=r&&r.id!=null?String(r.id):'';"
    "var fn=rid?window.__empiierosSeatBrowserWait[rid]:null;"
    "if(!fn)return;"
    "delete window.__empiierosSeatBrowserWait[rid];"
    "fn(r||{ok:false,code:'seat_browser_unavailable'});};"
    "function seatBrowserPost(o){"
    "return new Promise(function(resolve){"
    "var id=String(++window.__empiierosSeatBrowserSeq);"
    "o.id=id;"
    "var t=setTimeout(function(){"
    "if(!window.__empiierosSeatBrowserWait[id])return;"
    "delete window.__empiierosSeatBrowserWait[id];"
    "resolve({ok:false,code:'seat_browser_unavailable'});},2500);"
    "window.__empiierosSeatBrowserWait[id]=function(r){clearTimeout(t);resolve(r);};"
    "try{window.webkit.messageHandlers.empiierosSeatBrowser.postMessage(JSON.stringify(o));}"
    "catch(e){clearTimeout(t);delete window.__empiierosSeatBrowserWait[id];"
    "resolve({ok:false,code:'seat_browser_unavailable'});}});}"
    "window.EmperyDesktopSeatBrowser={"
    "openSeatBrowser:function(args){"
    "var a=args&&typeof args==='object'?args:{};"
    "if(a.accessToken||a.refreshToken||a.token){"
    "return Promise.resolve({ok:false,code:'token_forbidden'});}"
    "var u=a.url==null?'':String(a.url);"
    "if(u&&u.indexOf('https://')!==0){"
    "return Promise.resolve({ok:false,code:'scheme'});}"
    "var o={op:'open'};"
    "if(u)o.url=u;"
    "return seatBrowserPost(o);},"
    "closeSeatBrowser:function(){"
    "return seatBrowserPost({op:'close'});}"
    "};"
    "}catch(e){}})();";

static char *oidc_runtime_inject(EmpiierNativeBridge *bridge, gboolean force, GError **error) {
    return empiier_native_runtime_exchange(
        bridge,
        force
            ? "{\"id\":\"oidc-inject\",\"op\":\"launcher.oidc_inject\",\"params\":{\"force\":true}}"
            : "{\"id\":\"oidc-inject\",\"op\":\"launcher.oidc_inject\",\"params\":{}}",
        error
    );
}

static void oidc_product_resolve(EmpiierNativeBridge *bridge, const char *json) {
    if (bridge == NULL || bridge->web_view == NULL || json == NULL) {
        return;
    }
    char *escaped = empiier_native_json_escape(json);
    char *script = g_strdup_printf(
        "if(window.__empiierosOidcDone)window.__empiierosOidcDone(%s);",
        escaped
    );
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-oidc-product",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
    g_free(escaped);
}

static void oidc_product_resolve_deny(EmpiierNativeBridge *bridge) {
    oidc_product_resolve(
        bridge,
        "{\"ok\":false,\"error\":\"Authentication could not be completed\"}"
    );
}

static gboolean oidc_hide_product_later(gpointer user_data) {
    empiier_native_bridge_hide_product(user_data);
    return G_SOURCE_REMOVE;
}

static gboolean oidc_apply_inject_response(
    const char *response,
    char **token_out,
    char **id_out
) {
    if (response == NULL
        || strstr(response, "\"access_token\"") != NULL
        || strstr(response, "\"refresh_token\"") != NULL) {
        syslog(LOG_ERR, "oidc inject denied refresh");
        return FALSE;
    }
    char *token = extract_json_string(response, "token");
    char *id_token = extract_json_string(response, "idToken");
    if (!jwt_safe(token)) {
        g_free(token);
        g_free(id_token);
        syslog(LOG_ERR, "oidc inject missing session");
        return FALSE;
    }
    if (id_token != NULL && !jwt_safe(id_token)) {
        g_free(id_token);
        id_token = NULL;
    }
    if (token_out != NULL) {
        *token_out = token;
    } else {
        g_free(token);
    }
    if (id_out != NULL) {
        *id_out = id_token;
    } else {
        g_free(id_token);
    }
    return TRUE;
}

gboolean empiier_oidc_inject_product(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return FALSE;
    }
    GError *error = NULL;
    char *response = oidc_runtime_inject(bridge, FALSE, &error);
    if (response == NULL) {
        syslog(LOG_ERR, "oidc inject failed: %s", error != NULL ? error->message : "unknown");
        g_clear_error(&error);
        return FALSE;
    }
    char *token = NULL;
    char *id_token = NULL;
    if (!oidc_apply_inject_response(response, &token, &id_token)) {
        g_free(response);
        return FALSE;
    }
    g_free(response);
    empiier_oidc_clear_product(bridge);
    char *script = NULL;
    if (id_token != NULL) {
        script = g_strdup_printf(
            "%svar bootstrap={ok:true,token:\"%s\",idToken:\"%s\"};%s",
            OIDC_INJECT_HEAD,
            token,
            id_token,
            OIDC_INJECT_TAIL
        );
    } else {
        script = g_strdup_printf(
            "%svar bootstrap={ok:true,token:\"%s\"};%s",
            OIDC_INJECT_HEAD,
            token != NULL ? token : "",
            OIDC_INJECT_TAIL
        );
    }
    bridge->oidc_script = webkit_user_script_new(
        script,
        WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
        WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
        NULL,
        NULL
    );
    webkit_user_content_manager_add_script(bridge->manager, bridge->oidc_script);
    g_free(script);
    g_free(token);
    g_free(id_token);
    syslog(LOG_INFO, "oidc session injected origin=fofline-spa");
    syslog(LOG_INFO, "fofline meet stage hook planted handler=empiierosMeet");
    syslog(LOG_INFO, "fofline seat browser hook planted handler=empiierosSeatBrowser");
    return TRUE;
}

typedef struct {
    EmpiierNativeBridge *bridge;
    char *op;
} OidcProductJob;

static void oidc_product_job_free(OidcProductJob *job) {
    if (job == NULL) {
        return;
    }
    g_free(job->op);
    g_free(job);
}

static void oidc_product_apply_tokens(
    EmpiierNativeBridge *bridge,
    const char *token,
    const char *id_token
) {
    char *script = NULL;
    if (id_token != NULL && id_token[0] != '\0') {
        script = g_strdup_printf(
            "try{var bootstrap={ok:true,token:\"%s\",idToken:\"%s\"};"
            "window.__ELITEEW_PRODUCT_SESSION_BOOTSTRAP__=bootstrap;"
            "if(window.__empiierosOidcDone)window.__empiierosOidcDone(JSON.stringify(bootstrap));"
            "}catch(e){if(window.__empiierosOidcDone)window.__empiierosOidcDone("
            "JSON.stringify({ok:false,error:'Authentication could not be completed'}));}",
            token,
            id_token
        );
    } else {
        script = g_strdup_printf(
            "try{var bootstrap={ok:true,token:\"%s\"};"
            "window.__ELITEEW_PRODUCT_SESSION_BOOTSTRAP__=bootstrap;"
            "if(window.__empiierosOidcDone)window.__empiierosOidcDone(JSON.stringify(bootstrap));"
            "}catch(e){if(window.__empiierosOidcDone)window.__empiierosOidcDone("
            "JSON.stringify({ok:false,error:'Authentication could not be completed'}));}",
            token
        );
    }
    webkit_web_view_evaluate_javascript(
        bridge->web_view,
        script,
        -1,
        NULL,
        "empiieros-oidc-product",
        NULL,
        NULL,
        NULL
    );
    g_free(script);
}

static void oidc_product_thread(
    GTask *task,
    gpointer source_object,
    gpointer task_data,
    GCancellable *cancellable
) {
    (void)source_object;
    (void)cancellable;
    OidcProductJob *job = task_data;
    GError *error = NULL;
    char *response = NULL;
    if (g_strcmp0(job->op, "logout") == 0) {
        response = empiier_native_runtime_exchange(
            job->bridge,
            "{\"id\":\"oidc-logout\",\"op\":\"launcher.oidc_logout\",\"params\":{}}",
            &error
        );
    } else {
        response = oidc_runtime_inject(job->bridge, TRUE, &error);
    }
    if (response == NULL) {
        g_task_return_error(task, error);
        return;
    }
    if (strstr(response, "\"access_token\"") != NULL
        || strstr(response, "\"refresh_token\"") != NULL) {
        g_free(response);
        g_task_return_new_error(
            task,
            G_IO_ERROR,
            G_IO_ERROR_FAILED,
            "oidc token leaked"
        );
        return;
    }
    g_task_return_pointer(task, response, g_free);
}

static void oidc_product_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)source_object;
    EmpiierNativeBridge *bridge = user_data;
    GError *error = NULL;
    char *response = g_task_propagate_pointer(G_TASK(result), &error);
    bridge->oidc_product_busy = FALSE;
    if (response == NULL) {
        syslog(
            LOG_ERR,
            "oidc product bridge failed: %s",
            error != NULL ? error->message : "unknown"
        );
        g_clear_error(&error);
        oidc_product_resolve_deny(bridge);
        return;
    }
    if (strstr(response, "\"oidc_session\":\"missing\"") != NULL) {
        syslog(LOG_INFO, "oidc product logout applied");
        oidc_product_resolve(bridge, "{\"ok\":false}");
        g_free(response);
        g_timeout_add(120, oidc_hide_product_later, bridge);
        return;
    }
    char *token = NULL;
    char *id_token = NULL;
    if (!oidc_apply_inject_response(response, &token, &id_token)) {
        g_free(response);
        oidc_product_resolve_deny(bridge);
        return;
    }
    g_free(response);
    oidc_product_apply_tokens(bridge, token, id_token);
    syslog(LOG_INFO, "oidc product refresh applied");
    g_free(token);
    g_free(id_token);
}

static char *oidc_product_op(JSCValue *value) {
    if (value == NULL) {
        return NULL;
    }
    if (jsc_value_is_string(value)) {
        return jsc_value_to_string(value);
    }
    if (!jsc_value_is_object(value)) {
        return NULL;
    }
    JSCValue *op = jsc_value_object_get_property(value, "op");
    if (op == NULL || !jsc_value_is_string(op)) {
        return NULL;
    }
    return jsc_value_to_string(op);
}

void empiier_oidc_on_product_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *js_result,
    gpointer user_data
) {
    (void)manager;
    EmpiierNativeBridge *bridge = user_data;
    const char *uri = webkit_web_view_get_uri(bridge->web_view);
    if (!empiier_native_bridge_fofline_uri(uri)) {
        syslog(LOG_ERR, "oidc product bridge denied origin");
        return;
    }
    JSCValue *value = webkit_javascript_result_get_js_value(js_result);
    char *op = oidc_product_op(value);
    if (g_strcmp0(op, "refresh") != 0 && g_strcmp0(op, "logout") != 0) {
        syslog(LOG_ERR, "oidc product bridge denied op");
        g_free(op);
        oidc_product_resolve_deny(bridge);
        return;
    }
    if (bridge->oidc_product_busy) {
        g_free(op);
        return;
    }
    bridge->oidc_product_busy = TRUE;
    OidcProductJob *job = g_new0(OidcProductJob, 1);
    job->bridge = bridge;
    job->op = op;
    GTask *task = g_task_new(NULL, NULL, oidc_product_finished, bridge);
    g_task_set_task_data(task, job, (GDestroyNotify)oidc_product_job_free);
    g_task_run_in_thread(task, oidc_product_thread);
    g_object_unref(task);
}

typedef struct {
    EmpiierNativeBridge *bridge;
} NetworkLinkJob;

static gboolean network_link_busy = FALSE;

static void network_link_job_free(NetworkLinkJob *job) {
    g_free(job);
}

static void network_link_resolve(
    EmpiierNativeBridge *bridge,
    gboolean ok,
    const char *kind,
    gboolean active
) {
    const char *kind_js = "null";
    if (!ok || (g_strcmp0(kind, "ethernet") != 0 && g_strcmp0(kind, "wifi") != 0)) {
        ok = FALSE;
        active = FALSE;
    } else if (g_strcmp0(kind, "wifi") == 0) {
        kind_js = "\"wifi\"";
    } else {
        kind_js = "\"ethernet\"";
    }
    char *script = g_strdup_printf(
        "if(window.__empiierosNetworkDone)window.__empiierosNetworkDone({ok:%s,kind:%s,active:%s});",
        ok ? "true" : "false",
        kind_js,
        (ok && active) ? "true" : "false"
    );
    if (bridge != NULL && bridge->web_view != NULL) {
        webkit_web_view_evaluate_javascript(
            bridge->web_view,
            script,
            -1,
            NULL,
            "empiieros-network-link",
            NULL,
            NULL,
            NULL
        );
    }
    g_free(script);
    if (ok) {
        syslog(LOG_INFO, "fofline network link kind=%s active=%d", kind, active ? 1 : 0);
    } else {
        syslog(LOG_INFO, "fofline network link unavailable");
    }
}

static void network_link_thread(
    GTask *task,
    gpointer source_object,
    gpointer task_data,
    GCancellable *cancellable
) {
    (void)source_object;
    (void)cancellable;
    NetworkLinkJob *job = task_data;
    GError *error = NULL;
    char *response = empiier_native_runtime_exchange(
        job->bridge,
        "{\"id\":\"net-link\",\"op\":\"network.link_snapshot\",\"params\":{}}",
        &error
    );
    if (response == NULL) {
        g_task_return_error(task, error);
        return;
    }
    g_task_return_pointer(task, response, g_free);
}

static void network_link_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)source_object;
    EmpiierNativeBridge *bridge = user_data;
    GError *error = NULL;
    char *response = g_task_propagate_pointer(G_TASK(result), &error);
    network_link_busy = FALSE;
    if (response == NULL) {
        syslog(
            LOG_ERR,
            "fofline network link failed: %s",
            error != NULL ? error->message : "unknown"
        );
        g_clear_error(&error);
        network_link_resolve(bridge, FALSE, NULL, FALSE);
        return;
    }
    char *kind = extract_json_string(response, "kind");
    gboolean active = strstr(response, "\"active\":true") != NULL;
    gboolean ok = kind != NULL && strstr(response, "\"ok\":true") != NULL;
    network_link_resolve(bridge, ok, kind, active);
    g_free(kind);
    g_free(response);
}

static void on_network_link_message(
    WebKitUserContentManager *manager,
    WebKitJavascriptResult *js_result,
    gpointer user_data
) {
    (void)manager;
    EmpiierNativeBridge *bridge = user_data;
    if (bridge == NULL || bridge->web_view == NULL) {
        return;
    }
    const char *uri = webkit_web_view_get_uri(bridge->web_view);
    if (!empiier_native_bridge_fofline_uri(uri)) {
        syslog(LOG_ERR, "fofline network link denied origin");
        return;
    }
    JSCValue *value = webkit_javascript_result_get_js_value(js_result);
    char *op = NULL;
    if (value != NULL && jsc_value_is_string(value)) {
        op = jsc_value_to_string(value);
    }
    if (g_strcmp0(op, "snapshot") != 0) {
        syslog(LOG_ERR, "fofline network link denied op");
        g_free(op);
        network_link_resolve(bridge, FALSE, NULL, FALSE);
        return;
    }
    g_free(op);
    if (network_link_busy) {
        return;
    }
    network_link_busy = TRUE;
    NetworkLinkJob *job = g_new0(NetworkLinkJob, 1);
    job->bridge = bridge;
    GTask *task = g_task_new(NULL, NULL, network_link_finished, bridge);
    g_task_set_task_data(task, job, (GDestroyNotify)network_link_job_free);
    g_task_run_in_thread(task, network_link_thread);
    g_object_unref(task);
}

void empiier_network_register_product_bridge(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return;
    }
    g_signal_connect(
        bridge->manager,
        "script-message-received::empiierosNetwork",
        G_CALLBACK(on_network_link_message),
        bridge
    );
    if (!webkit_user_content_manager_register_script_message_handler(
            bridge->manager,
            "empiierosNetwork"
        )) {
        g_error("failed to register empiierosNetwork script message handler");
    }
}

void empiier_oidc_register_product_bridge(EmpiierNativeBridge *bridge) {
    if (bridge == NULL || bridge->manager == NULL) {
        return;
    }
    g_signal_connect(
        bridge->manager,
        "script-message-received::empiierosOidc",
        G_CALLBACK(empiier_oidc_on_product_message),
        bridge
    );
    if (!webkit_user_content_manager_register_script_message_handler(
            bridge->manager,
            "empiierosOidc"
        )) {
        g_error("failed to register empiierosOidc script message handler");
    }
}

typedef struct {
    EmpiierNativeBridge *bridge;
    char *callback_url;
} OidcCompleteJob;

static void oidc_complete_job_free(OidcCompleteJob *job) {
    if (job == NULL) {
        return;
    }
    g_free(job->callback_url);
    g_free(job);
}

static void oidc_complete_thread(
    GTask *task,
    gpointer source_object,
    gpointer task_data,
    GCancellable *cancellable
) {
    (void)source_object;
    (void)cancellable;
    OidcCompleteJob *job = task_data;
    char *url_json = empiier_native_json_escape(job->callback_url);
    char *request = g_strdup_printf(
        "{\"id\":\"oidc-complete\",\"op\":\"launcher.oidc_complete\",\"params\":{\"callback_url\":%s}}",
        url_json
    );
    g_free(url_json);
    GError *error = NULL;
    char *response = empiier_native_runtime_exchange(job->bridge, request, &error);
    g_free(request);
    if (response == NULL) {
        g_task_return_error(task, error);
        return;
    }
    if (strstr(response, "\"access_token\"") != NULL
        || strstr(response, "\"refresh_token\"") != NULL) {
        g_free(response);
        g_task_return_new_error(task, G_IO_ERROR, G_IO_ERROR_FAILED, "oidc token leaked");
        return;
    }
    g_task_return_pointer(task, response, g_free);
}

static void oidc_complete_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)source_object;
    EmpiierNativeBridge *bridge = user_data;
    GError *error = NULL;
    char *response = g_task_propagate_pointer(G_TASK(result), &error);
    empiier_native_bridge_hide_auth(bridge);
    if (response != NULL && strstr(response, "\"oidc_session\":\"present\"") != NULL) {
        syslog(LOG_INFO, "oidc complete applied");
        empiier_native_dispatch_custom_event(bridge, "empiieros-oidc", "{\"ok\":true}");
    } else {
        char *fail_code = extract_json_string(response, "message");
        syslog(
            LOG_ERR,
            "oidc complete failed: %s",
            fail_code != NULL && fail_code[0] != '\0'
                ? fail_code
                : (error != NULL ? error->message : "denied")
        );
        g_free(fail_code);
        empiier_native_dispatch_custom_event(bridge, "empiieros-oidc", "{\"ok\":false}");
    }
    g_free(response);
    g_clear_error(&error);
}

void empiier_oidc_complete_from_callback(
    EmpiierNativeBridge *bridge,
    const char *callback_url
) {
    if (bridge == NULL || !empiier_native_bridge_auth_callback(callback_url)) {
        return;
    }
    OidcCompleteJob *job = g_new0(OidcCompleteJob, 1);
    job->bridge = bridge;
    job->callback_url = g_strdup(callback_url);
    GTask *task = g_task_new(NULL, NULL, oidc_complete_finished, bridge);
    g_task_set_task_data(task, job, (GDestroyNotify)oidc_complete_job_free);
    g_task_run_in_thread(task, oidc_complete_thread);
    g_object_unref(task);
}

static gboolean auth_login_fields_allow(GPtrArray *names, GPtrArray *values) {
    const char *user = NULL;
    const char *pass = NULL;
    const char *otp = NULL;
    guint count = names != NULL && values != NULL
        ? MIN(names->len, values->len)
        : 0U;
    for (guint index = 0; index < count; index += 1U) {
        const char *key = g_ptr_array_index(names, index);
        const char *value = g_ptr_array_index(values, index);
        if (key == NULL || value == NULL) {
            continue;
        }
        if (g_ascii_strcasecmp(key, "username") == 0 || g_ascii_strcasecmp(key, "email") == 0) {
            user = value;
        } else if (g_ascii_strcasecmp(key, "password") == 0) {
            pass = value;
        } else if (g_ascii_strcasecmp(key, "otp") == 0) {
            otp = value;
        }
    }
    if (otp != NULL && otp[0] != '\0' && user == NULL && pass == NULL) {
        return TRUE;
    }
    return user != NULL && user[0] != '\0' && pass != NULL && pass[0] != '\0';
}

void empiier_oidc_on_auth_submit_form(
    WebKitWebView *web_view,
    WebKitFormSubmissionRequest *request,
    gpointer user_data
) {
    (void)user_data;
    const char *uri = webkit_web_view_get_uri(web_view);
    if (!auth_host_allowed(uri) && !auth_start_prefix(uri)) {
        webkit_form_submission_request_submit(request);
        return;
    }
    GPtrArray *names = NULL;
    GPtrArray *values = NULL;
    gboolean listed = webkit_form_submission_request_list_text_fields(request, &names, &values);
    guint fields = listed && names != NULL ? names->len : 0U;
    gboolean allowed = listed && auth_login_fields_allow(names, values);
    syslog(
        LOG_INFO,
        "oidc auth form submit allowed=%u fields=%u",
        allowed ? 1U : 0U,
        fields
    );
    if (names != NULL) {
        g_ptr_array_unref(names);
    }
    if (values != NULL) {
        g_ptr_array_unref(values);
    }
    if (allowed) {
        webkit_form_submission_request_submit(request);
        return;
    }
    syslog(LOG_WARNING, "oidc auth form blocked reason=empty_hidden_or_incomplete");
    /* Dispose auto-submits when the last ref drops. Living FoFline leaks one. */
    g_object_ref(request);
}

static void oidc_fresh_par_thread(
    GTask *task,
    gpointer source_object,
    gpointer task_data,
    GCancellable *cancellable
) {
    (void)source_object;
    (void)cancellable;
    EmpiierNativeBridge *bridge = task_data;
    GError *error = NULL;
    char *response = empiier_native_runtime_exchange(
        bridge,
        "{\"id\":\"oidc-fresh\",\"op\":\"launcher.oidc_start\",\"params\":{}}",
        &error
    );
    if (response == NULL) {
        g_task_return_error(task, error);
        return;
    }
    g_task_return_pointer(task, response, g_free);
}

static void oidc_fresh_par_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)source_object;
    EmpiierNativeBridge *bridge = user_data;
    GError *error = NULL;
    char *response = g_task_propagate_pointer(G_TASK(result), &error);
    bridge->auth_fresh_busy = FALSE;
    char *url = extract_json_string(response, "authorization_url");
    if (url != NULL && empiier_native_bridge_show_auth(bridge, url)) {
        syslog(LOG_INFO, "oidc auth rebuild applied");
    } else {
        syslog(
            LOG_ERR,
            "oidc auth rebuild failed: %s",
            error != NULL ? error->message : "denied"
        );
    }
    g_free(url);
    g_free(response);
    g_clear_error(&error);
}

static void empiier_oidc_try_fresh_par(EmpiierNativeBridge *bridge, const char *reason) {
    if (bridge == NULL) {
        return;
    }
    if (bridge->auth_fresh_busy) {
        return;
    }
    if (bridge->auth_fresh_budget == 0) {
        syslog(
            LOG_WARNING,
            "oidc auth rebuild denied reason=%s budget=0",
            reason != NULL ? reason : "unknown"
        );
        return;
    }
    bridge->auth_fresh_budget = 0;
    bridge->auth_fresh_busy = TRUE;
    syslog(LOG_INFO, "oidc auth rebuild start reason=%s", reason != NULL ? reason : "unknown");
    GTask *task = g_task_new(NULL, NULL, oidc_fresh_par_finished, bridge);
    g_task_set_task_data(task, bridge, NULL);
    g_task_run_in_thread(task, oidc_fresh_par_thread);
    g_object_unref(task);
}

static void oidc_inspect_finished(
    GObject *source_object,
    GAsyncResult *result,
    gpointer user_data
) {
    (void)user_data;
    GError *error = NULL;
    JSCValue *value = webkit_web_view_evaluate_javascript_finish(
        WEBKIT_WEB_VIEW(source_object),
        result,
        &error
    );
    char *kind = value != NULL && jsc_value_is_string(value)
        ? jsc_value_to_string(value)
        : NULL;
    syslog(LOG_INFO, "oidc auth page kind=%s", kind != NULL ? kind : "unknown");
    /* First GET error.ftl is Keycloak preauth/precheck, not an empty
     * WebKit submit. A second PAR here burns the ticket and still
     * paints «Мы сожалеем». Rebuild only on back-to-app. */
    g_free(kind);
    g_clear_object(&value);
    g_clear_error(&error);
}

void empiier_oidc_on_auth_page_finished(
    WebKitWebView *web_view,
    EmpiierNativeBridge *bridge
) {
    const char *uri = webkit_web_view_get_uri(web_view);
    const char *title = webkit_web_view_get_title(web_view);
    if (uri == NULL || g_strcmp0(uri, "about:blank") == 0) {
        return;
    }
    syslog(LOG_INFO, "oidc auth page title=%s", title != NULL && title[0] != '\0' ? title : "");
    webkit_web_view_evaluate_javascript(
        web_view,
        "(function(){try{"
        "var host=String(location.hostname||'').toLowerCase();"
        "if(host!=='auth.eliteew.ru'&&host.indexOf('.auth.eliteew.ru')===-1)return 'skip';"
        "var path=String(location.pathname||'');"
        "if(document.getElementById('kc-form-login'))return 'login';"
        "if(document.querySelector(\"input[name='password']\"))return 'login';"
        "var t=(document.body&&document.body.innerText)||'';"
        "var err=path.indexOf('/login-actions/error')!==-1"
        "||t.indexOf('Назад в приложение')!==-1"
        "||t.indexOf('Back to Application')!==-1"
        "||t.indexOf('Мы сожалеем')!==-1"
        "||t.indexOf('We are sorry')!==-1;"
        "if(!err)return 'other';"
        "if(t.indexOf('Неправильное')!==-1||t.indexOf('Invalid user')!==-1)return 'error:credentials';"
        "if(t.indexOf('Access denied')!==-1||t.indexOf('доступ')!==-1)return 'error:denied';"
        "return 'error:sorry';"
        "}catch(e){return 'skip';}})()",
        -1,
        NULL,
        "empiieros-oidc-inspect",
        NULL,
        oidc_inspect_finished,
        bridge
    );
}
