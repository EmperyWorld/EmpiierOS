#!/usr/bin/env bash
# Semantic smoke for the currently wired secure Shell and native transport.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

[[ -d shell-ui/node_modules ]] || {
  echo "FAIL  shell-ui/node_modules absent; run npm ci --prefix shell-ui" >&2
  exit 1
}

./shell-ui/node_modules/.bin/tsx <<'TS'
const events = new EventTarget();
let posted: Record<string, unknown> | null = null;
const fakeWindow = Object.assign(events, {
  setTimeout,
  clearTimeout,
  webkit: undefined as
    | undefined
    | {
        messageHandlers: {
          empiieros: { postMessage: (message: unknown) => void };
        };
      },
});
Object.assign(globalThis, { window: fakeWindow });

const transport = await import("./shell-ui/src/secure/nativeTransport.ts");
if (transport.nativeBridgeAvailable()) {
  throw new Error("bridge must not be reported available without a handler");
}
await transport.runtimeRequest("runtime.health").then(
  () => {
    throw new Error("missing bridge must reject");
  },
  (error: unknown) => {
    if (!(error instanceof Error) || !error.message.includes("unavailable")) {
      throw error;
    }
  },
);

fakeWindow.webkit = {
  messageHandlers: {
    empiieros: {
      postMessage: (message: unknown): void => {
        posted = message as Record<string, unknown>;
      },
    },
  },
};
if (!transport.nativeBridgeAvailable()) {
  throw new Error("bridge handler was not detected");
}

const pending = transport.runtimeRequest<{ transport: string }>(
  "runtime.health",
  {},
  1_000,
);
if (posted?.op !== "runtime.health" || typeof posted.id !== "string") {
  throw new Error("request was not framed for the native bridge");
}
fakeWindow.dispatchEvent(
  new CustomEvent("empiieros-response", {
    detail: {
      protocol_version: 1,
      id: posted.id,
      ok: true,
      status: "ok",
      data: { transport: "unix-peercred-v1" },
      error: null,
    },
  }),
);
const response = await pending;
if (!response.ok || response.data?.transport !== "unix-peercred-v1") {
  throw new Error("native response correlation failed");
}

const locale = await import("./shell-ui/src/secure/locale/index.ts");
const en = locale.localeStrings("en");
const ru = locale.localeStrings("ru");
const sameKeys = (left: object, right: object): boolean => {
  const leftKeys = Object.keys(left).sort();
  const rightKeys = Object.keys(right).sort();
  if (JSON.stringify(leftKeys) !== JSON.stringify(rightKeys)) return false;
  return leftKeys.every((key) => {
    const leftValue = (left as Record<string, unknown>)[key];
    const rightValue = (right as Record<string, unknown>)[key];
    return typeof leftValue === "object" && leftValue !== null
      ? typeof rightValue === "object" &&
          rightValue !== null &&
          sameKeys(leftValue, rightValue)
      : typeof leftValue === typeof rightValue;
  });
};
if (!sameKeys(en, ru)) throw new Error("locale key sets diverged");
if (locale.localizedValue(ru, "local") !== "Только локальная сеть") {
  throw new Error("Russian machine-state localization failed");
}

console.log("PASS  native transport correlation + locale parity");
TS

python3 - <<'PY'
from pathlib import Path

main = Path("shell-ui/src/main.tsx").read_text(encoding="utf-8")
app = Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8")
poll = Path("shell-ui/src/secure/useSystemSnapshot.ts").read_text(encoding="utf-8")
bridge = Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
native_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
css = Path("shell-ui/src/secure/secure.css").read_text(encoding="utf-8")
vite = Path("shell-ui/vite.config.ts").read_text(encoding="utf-8")

assert 'from "./secure/SecureApp"' in main
assert "shell.window_mode" in app
assert "windowModeFailed" in app
assert "empiieros-window-mode" in app
assert "HelpTip" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert "hostWorkspaceHelp" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert "WorkspaceFrame" in app
assert "VITE_UI_LOCALE as string | undefined" in app
assert "navigator.language" in app
assert '"ru"' in app
assert "snapshot?.locale.data.locale" in app
assert "system.watch" in poll
assert "applyLocalFreshness" in poll
assert "empiieros-event" in poll
assert "void watch();" in poll and "schedule(delay" in poll
system = Path("shell-ui/src/secure/components/SystemPanel.tsx").read_text(encoding="utf-8")
diagnostics = Path("shell-ui/src/secure/components/DiagnosticsPanel.tsx").read_text(
    encoding="utf-8"
)
assert 'overall_status: "verified"' not in diagnostics  # value comes from snapshot
assert 'snapshot.overall_status === "verified" ? strings.ok : strings.attention' in diagnostics
assert 'snapshot.overall_status === "verified" ? "connected" : "degraded"' in diagnostics
assert "DetailsSpoiler" in diagnostics
assert 'state="ok"' not in diagnostics
tray = Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(encoding="utf-8")
toasts = Path("shell-ui/src/secure/components/NotificationToastStack.tsx").read_text(
    encoding="utf-8"
)
assert "toggleQuietMode()" in tray
assert "aria-pressed={quietMode}" in tray
assert "toastEligible(item)" in toasts
assert '"suppressed": suppressed,' in Path("runtime/src/kiosk_tray.rs").read_text(encoding="utf-8")
assert "snapshot.display" in system
assert "snapshot.locale" in system
assert "eventsEmpty" in diagnostics
assert "trusted_shell_uri" in bridge
assert "g_canonicalize_filename" in bridge
assert "dispatch_correlated_response" in bridge
assert "empiieros-event" in bridge
assert "runtime bridge response id=%s" in bridge
assert '#define SHELL_URI "empiier://shell/index.html"' in native_main
assert "fofline-spa" in native_main
assert "PACK_ROOT" in native_main
assert "webkit_web_context_register_uri_scheme" in native_main
assert "webkit_security_manager_register_uri_scheme_as_cors_enabled" in native_main
assert 'soup_message_headers_append(headers, "Access-Control-Allow-Origin", "*")' in native_main
assert "webkit_uri_scheme_request_finish_with_response" in native_main
assert "webkit_uri_scheme_response_set_status(response, 200U, NULL)" in native_main
assert "soup_message_headers_unref(headers)" not in native_main
assert "static void finish_scheme_ok" in native_main
assert "static void finish_scheme_bytes" in native_main
assert "static gboolean mime_is_stylesheet" in native_main
assert "static gboolean scheme_request_is_cors" in native_main
assert "static gboolean path_is_pack_module_css" in native_main
assert "fofline boot" in native_main
assert "Vite __vitePreload inserts" in native_main
assert "mode=simple cors=%d pack=%d" in native_main
assert 'register_uri_scheme_as_local(security_manager, "fofline-spa")' not in native_main
assert 'register_uri_scheme_as_local(security_manager, "empiier")' in native_main
assert "g_timeout_add(15000U, run_fofline_boot_probe" in native_main
assert "g_timeout_add(45000U, run_fofline_boot_probe" in native_main
assert "XMLHttpRequest.prototype.open" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "decodeURIComponent(escape(bin))" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "refreshToken:bootstrap.token" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "https://app.eliteew.ru'+u" not in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "u.slice(app.length)" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_fofline_api_proxy" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "fofline api proxy" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
proxy = Path("runtime/shell-gui/fofline_api_proxy.c").read_text(encoding="utf-8")
assert "path_is_content_download" in proxy
assert "content=octet-stream" in proxy
assert "sniff_audio_type" in proxy
assert "media_range" in proxy
assert "api_proxy_partial_content" in proxy
assert "Access-Control-Allow-Origin" in proxy
assert "Access-Control-Allow-Credentials" in proxy
assert "__empiierosAudioBlob" in native_main
assert "__empiierosNeedLoad" in native_main
assert (
    'soup_message_headers_replace(headers, "X-SSO-Replace-Confirm", "1")'
    in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(encoding="utf-8")
)
assert (
    "eliteew.auth.sessionReplaceConfirmedUntil.v1"
    in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
)
assert "function hostSsoRefresh" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "eliteew.mobileEmbeddedClient.v1" not in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "scheme css finish" in native_main
assert "__empiierBootErr" in native_main
assert "__empiierBootSrc" in native_main
assert "fofline edge start" in native_main
assert "copy_json_reason" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
assert "g_str_has_prefix(path, PACK_ROOT)" in native_main
assert "webkit_uri_scheme_request_get_http_headers" in native_main
assert 'g_strcmp0(mode, "cors") == 0' in native_main
assert 'g_strcmp0(mime_type, "text/css") == 0' in native_main
assert "b.token||b.accessToken" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert native_main.index("webkit_uri_scheme_response_set_http_headers") < native_main.index(
    "webkit_uri_scheme_response_set_content_type"
)
fofline_fn = native_main.split("static void serve_fofline_pack", 1)[1].split(
    "static void enable_webkit_sandbox", 1
)[0]
assert "finish_scheme_bytes" in fofline_fn
assert "Access-Control-Allow-Origin" not in fofline_fn
assert "webkit_web_context_set_sandbox_enabled(context, FALSE)" in native_main
assert "host CA/NSS product HTTPS" in native_main
assert "webkit_web_context_set_sandbox_enabled(context, TRUE)" not in native_main
assert "enable_webkit_sandbox(context)" in native_main
assert 'getenv("EMPIIEROS_SHELL_MAINTENANCE")' in native_main
assert "WebKit DOM diagnostics disabled" in native_main
assert "empiier_native_bridge_publish_window_state" in native_main
assert "set_allow_file_access_from_file_urls(settings, FALSE)" in native_main
assert "on_scroll_event" not in native_main
assert "GDK_KEY_F11" in native_main
assert 'g_signal_connect(view_widget, "scroll-event"' not in native_main
assert "GDK_SCROLL_MASK" not in native_main
assert "Do not connect scroll-event" in native_main
assert "empiier_native_bridge_apply_window_mode" in native_main
assert "empiier_native_bridge_apply_window_mode" in bridge
assert "empiier_shell_window_mode_service_allowed" in bridge
assert "shell-window-mode.allow" in bridge
assert "window_mode_service_disabled" in bridge
assert "windowModeServiceHint" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "windowedMode" not in Path("shell-ui/src/desktop/FoflineDesktop.tsx").read_text(
    encoding="utf-8"
)
assert "empiier_native_bridge_publish_window_state" in bridge
assert "shell window state observed mode=%s" in bridge
assert "shell scroll observed" not in native_main
assert "window.scrollBy({" not in native_main
assert "on_window_scroll_event" not in native_main
assert "installViewportLockedWheelScroll" in Path(
    "shell-ui/src/secure/SecureApp.tsx"
).read_text(encoding="utf-8")
assert "elementsFromPoint" in Path(
    "shell-ui/src/secure/wheelScrollTarget.ts"
).read_text(encoding="utf-8")
assert "webkit_settings_set_enable_media_stream(settings, TRUE)" in native_main
assert "g_lstat(candidate, &metadata)" in native_main
assert "RENDER_PROBE_MAX_ATTEMPTS" in native_main
assert "g_timeout_add(RENDER_PROBE_INTERVAL_MS, run_render_probe, probe)" in native_main
assert '"shell render ready attempts=%u"' in native_main
page_css = "\n".join(
    block
    for block in css.split("}")
    if any(
        token in block.split("{", 1)[0]
        for token in ("html", "body", "#root", ".secure-shell")
    )
)
assert "overflow-y: scroll" not in css
assert "100dvh" in css
assert "overflow: hidden" in page_css
assert "secure-app-window" in css
assert "8.4rem" not in css
assert "8.4rem" not in Path("shell-ui/src/desktop/fofline.css").read_text(encoding="utf-8")
assert "TaskbarTray" in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(
    encoding="utf-8"
)
assert "system.power" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert "taskbarPowerMenu" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "tray-volume.png" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert "setSeatVolume" in Path("shell-ui/src/secure/components/taskbarTrayControl.ts").read_text(
    encoding="utf-8"
)
assert "empiieros-local-stylesheet" in vite
assert 'rel="stylesheet"' in vite and "crossorigin" in vite
assert "empiieros-local-browser" in vite
assert "if (!ctx.server)" in vite
assert "LOCAL_DEV_CSP" in vite
assert "script-src empiier:" in Path("shell-ui/index.html").read_text(encoding="utf-8")
assert "LOCAL_DEV_CSP" not in Path("shell-ui/index.html").read_text(encoding="utf-8")
host = Path("shell-ui/src/secure/components/ProductHost.tsx").read_text(encoding="utf-8")
desktop = Path("shell-ui/src/desktop/FoflineDesktop.tsx").read_text(encoding="utf-8")
surface = Path("shell-ui/src/desktop/foflineConnectedSurface.ts").read_text(
    encoding="utf-8"
)
catalog = Path("shell-ui/src/desktop/foflineCatalog.ts").read_text(encoding="utf-8")
assert 'from "../../desktop/FoflineDesktop"' in host
assert "FoflineDesktop" in host
assert 'FOFLINE_CONNECTED_PROFILE_ID = "fofline-connected"' in surface
assert "GAFChatApp" in surface
assert "VideoCallApp" in surface
assert "DesktopAiChatApp" not in surface
assert "MusicForge" not in surface
assert "GAFChatApp" in catalog
assert "VideoCallApp" in catalog
assert 'localKind: "pack"' in catalog
assert "NotificationsApp" in catalog
assert "VoiceStudio" in catalog
assert 'localKind: "pack"' in catalog
assert 'localKind: "events"' in catalog
assert 'localKind: "audio"' in catalog
assert "product.open" in desktop
assert "capability_not_available" not in desktop  # probe observes Runtime; does not mint success
assert "edition.profile=fofline" not in desktop
assert "edition.profile=fofline" not in host
assert "foflinePackMissing" in desktop
assert "FoflineActivator" in desktop
activator = Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert 'runtimeRequest<LauncherStatus>("launcher.status"' in activator
assert "launcher.capsule_import" in activator
assert "launcher.air_activate" in activator
assert "launcher.pin_issue" not in activator
assert "launcher.pin_unlock" in activator
assert "product.open" in activator
assert "shell.product_surface" in activator
assert "autoOpened" not in activator
assert "launcher.oidc_start" in activator
assert "launcher.oidc_logout" in activator
assert "foflineActivatorOidcLeave" in activator
assert "shell.auth_surface" in activator
assert "oidc_session" in activator
assert "fofline-spa://localhost/index.html?eliteewFoflineClient=1" in activator
chrome = Path("shell-ui/src/desktop/foflineLauncherChrome.tsx").read_text(
    encoding="utf-8"
)
assert "foflineActivatorConsumeLocked" in chrome
assert "FoflineActivatorLab" in activator
assert "FoflineLauncherJourney" in activator
assert "tauri" in activator
assert "from \"./foflineLauncherStage\"" in activator
assert "mayAutoBringUpPortal" in activator
assert "autoOpened" not in Path("shell-ui/src/desktop/foflineLauncherStage.ts").read_text(
    encoding="utf-8"
)
assert "FoflineLauncherJourney" in chrome
assert (
    'export const LAUNCHER_JOURNEY = [\n  "pin",\n  "consume",\n  "portal",\n'
    '  "spa",\n  "login",\n  "ready",\n] as const;'
    in Path("shell-ui/src/desktop/foflineLauncherStage.ts").read_text(encoding="utf-8")
)
assert Path("session/SEAT-LAUNCHER.md").is_file()
assert Path("session/SEAT-FACE.md").is_file()
assert "empiier://shell/index.html" in Path("session/SEAT-FACE.md").read_text(
    encoding="utf-8"
)
assert "foflineHostTiles" in catalog
assert 'localKind !== "pack"' in catalog
assert "foflineHostTiles" in desktop
assert "Cloud-full Desktop is not mounted" not in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "foflineActivatorLabTools" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
auth_surface = Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "EmperyDesktopMeetStage" in auth_surface
assert "empiierosMeet" in auth_surface
assert "closeMeetStage" in auth_surface
assert "setMeetStageBounds" in auth_surface
assert "meet_stage_bounds_required" in auth_surface
assert "join_url_denied" in auth_surface
assert "charAt(23)" in auth_surface
assert "#define MEET_HOST_LEN 15" in Path("runtime/shell-gui/meet_surface.c").read_text(
    encoding="utf-8"
)
meet_surface = Path("runtime/shell-gui/meet_surface.c").read_text(encoding="utf-8")
meet_bounds = Path("runtime/shell-gui/meet_bounds.c").read_text(encoding="utf-8")
assert "meet_hostmap_unavailable" in meet_surface
assert "fofline meet stage denied code=" in meet_surface
assert "fofline meet stage request kind=" in meet_surface
assert "meet_stage_bounds_required" in meet_surface
assert "meet_stage_bounds_invalid" in meet_bounds
assert "enter-fullscreen" in meet_surface
assert "web-process-terminated" in meet_surface
assert "fofline meet stage crashed reason=" in meet_surface
assert "empiieros-meet-stage-lost" in meet_surface
assert "meet_process_crashed" in meet_surface
assert "fofline product media permission allowed" in meet_surface
assert "gtk_widget_set_halign(meet_box, GTK_ALIGN_FILL)" not in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "gtk_widget_set_halign(meet_box, GTK_ALIGN_START)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "fofline product meet navigation kind=iframe" in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8")
assert "empiier_overlay_raise" in auth_surface
assert "empiier_overlay_park" in auth_surface
assert "kind=card" in auth_surface
assert Path("runtime/shell-gui/auth_bounds.c").is_file()
assert "empiieros-auth-dimmer" in Path("runtime/shell-gui/auth_bounds.c").read_text(
    encoding="utf-8"
)
assert "gtk_window_new" not in Path("runtime/shell-gui/auth_bounds.c").read_text(
    encoding="utf-8"
)
assert "https://auth.eliteew.ru:443/" in auth_surface
assert 'strstr(uri, ":443") == NULL' not in auth_surface
assert "request_uri=urn%3Aietf%3Aparams%3Aoauth%3Arequest_uri%3A" in auth_surface
assert "oidc auth form blocked reason=empty_hidden_or_incomplete" in auth_surface
assert "Мы сожалеем" in auth_surface
assert "oidc auth rebuild start" in auth_surface
assert "eliteew_ticket" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "SeatShield" not in Path("shell-ui/src/desktop/foflineCatalog.ts").read_text(encoding="utf-8")
assert '"shield"' in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(encoding="utf-8")
assert "Отмена" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "webkit_web_context_set_sandbox_enabled(auth_context, FALSE)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
chrome = Path("runtime/shell-gui/product_chrome.c").read_text(encoding="utf-8")
assert "empiieros-seat-panel" in chrome
assert "always-on seat chrome" in chrome
assert "gtk_popover_set_modal" in chrome
assert "chrome_input_shape" in chrome
assert "movable no-peek" in chrome
assert "empiieros-hud-grip" in chrome
assert "GTK_POS_LEFT" in chrome
assert "chrome_peek_height" not in chrome
assert "chrome_apply_clip" not in chrome
assert "chrome_schedule_peek" not in chrome
assert "gtk_scrolled_window_new" not in chrome
assert "CHROME_PEEK_VISIBLE_DEN" not in chrome
assert "chrome_peek_margin" not in chrome
assert "margin-top: -48px" not in chrome
assert "schedule_collapse" not in chrome
assert "click-eliteew.wav" in Path("runtime/shell-gui/product_chrome.c").read_text(
    encoding="utf-8"
)
assert "chrome_hide_flyouts" in chrome
assert "HUD_FLY_VOLUME" in chrome
assert "Открыть EliteEW" in chrome
_vol = chrome[
    chrome.find("static void on_volume_clicked") : chrome.find(
        "static void on_shield_icon_clicked"
    )
]
assert "on_panel_clicked" not in _vol
assert Path("runtime/shell-gui/product-chrome/launcher.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-shield.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-ota.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-volume.png").is_file()
assert Path("runtime/shell-gui/product-chrome/panel-mic.png").is_file()
assert Path("runtime/shell-gui/product-chrome/start_os.mp4").is_file()
assert "SPLASH_VOLUME 0.25" in Path("runtime/shell-gui/boot_splash.c").read_text(
    encoding="utf-8"
)
assert "boot splash revealed hud" in Path("runtime/shell-gui/boot_splash.c").read_text(
    encoding="utf-8"
)
assert "empiier_widget_show_deferred" in Path("runtime/shell-gui/product_chrome.c").read_text(
    encoding="utf-8"
)
assert "gtk_widget_set_no_show_all(widget, FALSE)" in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8")
seat_browser = Path("runtime/shell-gui/seat_browser_surface.c").read_text(encoding="utf-8")
assert "empiier_seat_browser_chrome_build" in seat_browser
assert "empiier_native_bridge_seat_browser_command" in seat_browser
assert "seat.browser.reload" in Path(
    "shell-ui/src/secure/components/SeatBrowserCard.tsx"
).read_text(encoding="utf-8")
assert '"host_action": "reload"' in Path(
    "runtime/seat_browser_core/src/service.rs"
).read_text(encoding="utf-8")
core = Path("runtime/src/core.rs").read_text(encoding="utf-8")
assert '"launcher.status"' in core
assert '"launcher.capsule_import"' in core
assert '"launcher.air_activate"' in core
assert '"launcher.pin_issue"' in core
assert '"launcher.pin_unlock"' in core
assert '"product.open"' in core
assert '"launcher.oidc_start"' in core
assert '"launcher.oidc_complete"' in core
assert '"launcher.oidc_logout"' in core
assert "launcher_product::open" in core
probe = Path("runtime/src/system_probe.rs").read_text(encoding="utf-8")
gate = Path("runtime/src/capability_gate.rs").read_text(encoding="utf-8")
assert "capability_gate::capsule_import" in probe
assert "capability_gate::air_activate" in probe
assert "capability_gate::pin_issue" in probe
assert "capability_gate::pin_unlock" in probe
assert '"launcher.capsule_import"' in gate
assert '"launcher.air_activate"' in gate
assert '"launcher.pin_issue"' in gate
assert '"launcher.pin_unlock"' in gate
assert "license." in core
assert "product." in core
PY

./shell-ui/node_modules/.bin/tsx "$ROOT/shell-ui/src/desktop/foflineLauncherStage.check.ts"
./shell-ui/node_modules/.bin/tsx "$ROOT/shell-ui/src/session/seatBrowser.check.ts"
./shell-ui/node_modules/.bin/tsx "$ROOT/shell-ui/src/secure/notificationIpc.check.ts"

printf 'PASS  secure Shell wiring + recurring polling + confined URI boundary\n'
