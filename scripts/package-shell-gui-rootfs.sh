#!/usr/bin/env bash
# Build an immutable EmpiierOS overlay consumed by the Trusted Substrate factory.
# This script never repacks or writes a rootfs archive and never writes USB media.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUBSTRATE_ROOT="${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}"
BUILD_IMAGE=0
OPERATOR_KEY=""

usage() {
  cat <<'EOF'
Usage: scripts/package-shell-gui-rootfs.sh [options]

Options:
  --operator-key PATH  Provision one or more OpenSSH public keys.
  --build-image        Resume Trusted Substrate image assemble after staging
                       (payload overlay only; does not rebuild the floor).
  -h, --help           Show this help.

Output:
  $ELITEEW_TS_WORK_ROOT/empiieros-overlays/<manifest-sha256>/
  $ELITEEW_TS_WORK_ROOT/empiieros-rootfs-overlay -> immutable overlay
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --operator-key)
      [[ $# -ge 2 ]] || { echo "missing --operator-key path" >&2; exit 2; }
      OPERATOR_KEY="$2"
      shift 2
      ;;
    --build-image)
      BUILD_IMAGE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

[[ -d "$SUBSTRATE_ROOT" ]] || {
  echo "Trusted Substrate root not found: $SUBSTRATE_ROOT" >&2
  exit 1
}

# shellcheck source=/dev/null
source "$SUBSTRATE_ROOT/scripts/work-root.sh"

HOST_BIN="${ELITEEW_TS_BUILDROOT_OUT}/host/bin"
CC="${HOST_BIN}/x86_64-buildroot-linux-gnu-gcc"
STRIP="${HOST_BIN}/x86_64-buildroot-linux-gnu-strip"
OBJDUMP="${HOST_BIN}/x86_64-buildroot-linux-gnu-objdump"
PKG_CONFIG="${HOST_BIN}/pkg-config"

for tool in npm cargo python3 git; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "required build tool missing: $tool" >&2
    exit 1
  }
done
if [[ ! -x "$CC" || ! -x "$STRIP" || ! -x "$OBJDUMP" || ! -x "$PKG_CONFIG" ]] \
  || ! "$PKG_CONFIG" --exists webkit2gtk-4.1 gtk+-3.0 2>/dev/null; then
  printf '[package] Buildroot target SDK is absent; preparing it first\n'
  bash "$SUBSTRATE_ROOT/scripts/buildroot-build.sh" --prepare-empiieros-sdk
fi
for tool in "$CC" "$STRIP" "$OBJDUMP" "$PKG_CONFIG"; do
  [[ -x "$tool" ]] || {
    echo "Buildroot target tool missing: $tool" >&2
    exit 1
  }
done
"$PKG_CONFIG" --exists webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0 || {
  echo "Buildroot target WebKitGTK/GStreamer development metadata is missing" >&2
  exit 1
}

if [[ -n "$OPERATOR_KEY" ]]; then
  [[ -f "$OPERATOR_KEY" ]] || {
    echo "operator public-key file not found: $OPERATOR_KEY" >&2
    exit 1
  }
  python3 - "$OPERATOR_KEY" <<'PY'
from pathlib import Path
import base64
import re
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
if "PRIVATE KEY" in text:
    raise SystemExit("private key material is forbidden")
pattern = re.compile(
    r"^(ssh-ed25519|sk-ssh-ed25519@openssh\.com|"
    r"ecdsa-sha2-nistp(?:256|384|521)|"
    r"sk-ecdsa-sha2-nistp256@openssh\.com|ssh-rsa) "
    r"([A-Za-z0-9+/=]+)(?: .*)?$"
)
keys = [line.strip() for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]
if not keys:
    raise SystemExit("operator key file contains no public keys")
for line in keys:
    match = pattern.fullmatch(line)
    if match is None:
        raise SystemExit(f"unsupported authorized_keys line: {line[:48]}")
    base64.b64decode(match.group(2), validate=True)
PY
fi

cd "$ROOT"
printf '[package] deterministic Shell install\n'
npm --prefix shell-ui ci --ignore-scripts
npm --prefix shell-ui run build
[[ -s shell-ui/dist/index.html ]] || {
  echo "Shell build did not produce dist/index.html" >&2
  exit 1
}
python3 - <<'PY'
from pathlib import Path
import re

html = Path("shell-ui/dist/index.html").read_text(encoding="utf-8")
if "script-src empiier:" not in html:
    raise SystemExit("production dist/index.html lost kiosk CSP")
if "unsafe-eval" in html or "unsafe-inline" in html:
    raise SystemExit("production dist/index.html must not carry local Vite CSP")


def validate_stylesheet_transport(document: str) -> None:
    stylesheets = re.findall(r"<link\b[^>]*\brel=\"stylesheet\"[^>]*>", document)
    if len(stylesheets) != 1:
        raise ValueError(f"expected one generated stylesheet, found {len(stylesheets)}")
    if "crossorigin" in stylesheets[0]:
        raise ValueError("local custom-scheme stylesheet must not request CORS mode")


try:
    validate_stylesheet_transport(html)
except ValueError as error:
    raise SystemExit(str(error)) from error

adversarial = html.replace('rel="stylesheet"', 'rel="stylesheet" crossorigin', 1)
try:
    validate_stylesheet_transport(adversarial)
except ValueError:
    pass
else:
    raise SystemExit("stylesheet transport validator accepted adversarial crossorigin")
PY
if ! rg -q 'ota_reload_shell' shell-ui/dist/assets/index-*.js; then
  echo "Shell production bundle missing ota_reload_shell" >&2
  exit 1
fi

printf '[package] production Runtime cross-build\n'
# Cargo may honor an external CARGO_TARGET_DIR (Cursor sandbox). Bake from
# that path or the overlay will ship a stale workspace binary.
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/runtime/target}"
export CARGO_TARGET_DIR
# Factory-only: Buildroot host cargo/rustc (not Fedora host rustc). Cargo.lock
# must stay within this rustc's edition/rust-version ceiling.
export PATH="${HOST_BIN}:${PATH}"
case "$(command -v rustc)" in
  "${HOST_BIN}/rustc") ;;
  *)
    echo "factory rustc required under ${HOST_BIN}/rustc; got: $(command -v rustc)" >&2
    exit 1
    ;;
esac
printf '[package] cargo=%s rustc=%s\n' "$(command -v cargo)" "$(command -v rustc)"
cargo --version
rustc --version
# H2: glibc PIE + Full RELRO + NX. Do not switch this bake to musl.
EMPIIEROS_HARDEN_RUSTFLAGS="-C relocation-model=pic -C link-arg=-Wl,-z,relro,-z,now -C link-arg=-Wl,-z,noexecstack"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }${EMPIIEROS_HARDEN_RUSTFLAGS}"
printf '[package] target RUSTFLAGS harden gnu: %s\n' "$EMPIIEROS_HARDEN_RUSTFLAGS"
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin empiieros-runtime \
    --bin empiieros-air-enroll \
    --bin empiieros-portal-up \
    --bin empiieros-spa-fetch \
    --bin empiieros-ota-fetch
printf '[package] Seat Shield floor helper cross-build\n'
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/seat_shield_core/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin seat-shield-fw \
    --bin seat-shield-watch
printf '[package] Seat Shield inet_diag probe cross-build\n'
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/seat_shield_probe/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin seat-shield-probe
RUNTIME_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-runtime"
AIR_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-air-enroll"
PORTAL_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-portal-up"
SPA_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-spa-fetch"
OTA_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-ota-fetch"
SEAT_FW_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-fw"
SEAT_WATCH_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-watch"
SEAT_PROBE_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-probe"
[[ -x "$RUNTIME_BIN" ]] || {
  echo "Runtime cross-build output missing: $RUNTIME_BIN" >&2
  exit 1
}
[[ -x "$AIR_BIN" ]] || {
  echo "Air helper cross-build output missing: $AIR_BIN" >&2
  exit 1
}
[[ -x "$PORTAL_BIN" ]] || {
  echo "Portal helper cross-build output missing: $PORTAL_BIN" >&2
  exit 1
}
[[ -x "$SPA_BIN" ]] || {
  echo "SPA helper cross-build output missing: $SPA_BIN" >&2
  exit 1
}
[[ -x "$OTA_BIN" ]] || {
  echo "OTA helper cross-build output missing: $OTA_BIN" >&2
  exit 1
}
[[ -x "$SEAT_FW_BIN" ]] || {
  echo "Seat Shield helper cross-build output missing: $SEAT_FW_BIN" >&2
  exit 1
}
[[ -x "$SEAT_WATCH_BIN" ]] || {
  echo "Seat Shield watch helper cross-build output missing: $SEAT_WATCH_BIN" >&2
  exit 1
}
[[ -x "$SEAT_PROBE_BIN" ]] || {
  echo "Seat Shield probe helper cross-build output missing: $SEAT_PROBE_BIN" >&2
  exit 1
}
"$STRIP" "$RUNTIME_BIN"
"$STRIP" "$AIR_BIN"
"$STRIP" "$PORTAL_BIN"
"$STRIP" "$SPA_BIN"
"$STRIP" "$OTA_BIN"
"$STRIP" "$SEAT_FW_BIN"
"$STRIP" "$SEAT_WATCH_BIN"
"$STRIP" "$SEAT_PROBE_BIN"
printf '[package] OS-plane WireGuard userspace (no wg-quick)\n'
WG_TOOLS="${ELITEEW_TS_BUILDROOT_OUT}/build/wireguard-tools-1.0.20210914"
[[ -d "$WG_TOOLS/src" ]] || {
  echo "wireguard-tools source missing: $WG_TOOLS/src" >&2
  exit 1
}
make -C "$WG_TOOLS/src" -j"$(nproc)" \
  CC="$CC" \
  WITH_WGQUICK=no \
  WITH_SYSTEMDUNITS=no
WG_BIN="$(mktemp -p "$ELITEEW_TS_WORK_ROOT" empiieros-wg.XXXXXX)"
install -m 0755 "$WG_TOOLS/src/wg" "$WG_BIN"
"$STRIP" "$WG_BIN"
if rg -a -q 'wg-quick' "$WG_BIN"; then
  echo "empiieros-wg unexpectedly embeds wg-quick" >&2
  exit 1
fi
python3 - "$RUNTIME_BIN" <<'PY'
from pathlib import Path
import sys

runtime = Path(sys.argv[1]).read_bytes()
if b"EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER" in runtime:
    raise SystemExit("production Runtime contains the development peer bypass")
if b"launcher.pin_issue" not in runtime or b"launcher.pin_unlock" not in runtime:
    raise SystemExit("production Runtime is missing first-boot PIN ops")
if b"installer_issued_only" not in runtime:
    raise SystemExit("production Runtime still mints first-boot PIN over IPC")
if b"pin_not_unlocked" not in runtime:
    raise SystemExit("production Runtime does not gate consume on first-boot unlock")
if b"empiieros-first-boot-pin-v1|" not in runtime:
    raise SystemExit("production Runtime is missing first-boot PIN hash domain")
if b"launcher.edge_health" not in runtime or b"remote_spa_forbidden" not in runtime:
    raise SystemExit("production Runtime is missing typed product.open / edge_health")
if b"launcher.oidc_start" not in runtime or b"oidc_session_required" not in runtime:
    raise SystemExit("production Runtime is missing native FoFline OIDC")
if b"launcher.oidc_logout" not in runtime:
    raise SystemExit("production Runtime is missing native FoFline OIDC logout")
if b"ext/par/request" not in runtime:
    raise SystemExit("production Runtime is missing Keycloak PAR")
if b"eliteew_ticket" not in runtime:
    raise SystemExit("production Runtime is missing Keycloak binding ticket PAR field")
# Function names vanish after target strip. Lock payload / restart literals.
if b"ota_artifact_pk" not in runtime:
    raise SystemExit("production Runtime is missing Air OTA PK field")
if b"--on-active=3s" not in runtime:
    raise SystemExit("production Runtime is missing deferred cutover restart")
if b"date_header_missing" in runtime:
    raise SystemExit("production Runtime still has date_header_missing")
print("[package] Runtime carries first-boot PIN ops")
PY

python3 - <<'PY'
from pathlib import Path
import os

bridge = Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
auth = Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
chrome = Path("runtime/shell-gui/product_chrome.c").read_text(encoding="utf-8")
seat_browser = Path("runtime/shell-gui/seat_browser_surface.c").read_text(encoding="utf-8")
meet = Path("runtime/shell-gui/meet_surface.c").read_text(encoding="utf-8")
if "gtk_widget_set_no_show_all(widget, FALSE)" not in bridge:
    raise SystemExit("GTK3 overlay show still no-ops when no_show_all is set")
if "empiier_overlay_raise" not in auth or "empiier_overlay_park" not in auth:
    raise SystemExit("auth surface must raise/park the card overlay")
if "empiier_widget_show_deferred" not in chrome:
    raise SystemExit("product chrome must reveal overlays through show_deferred")
if "empiier_widget_show_deferred" not in bridge:
    raise SystemExit("overlay show_deferred helper missing")
if "shell.seat_browser_surface" not in bridge or "seat.browser.evaluate" not in seat_browser:
    raise SystemExit("seat browser overlay bridge/surface missing")
if "webkit_settings_set_enable_webrtc(browser_settings" not in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8"):
    raise SystemExit("seat browser overlay must enable WebRTC")
if "seat browser media permission allowed" not in seat_browser:
    raise SystemExit("seat browser must allow UserMedia after WebRTC is on")
if "empiier_seat_browser_chrome_build" not in seat_browser:
    raise SystemExit("seat browser overlay must build native chrome")
if "empiier_native_bridge_seat_browser_command" not in seat_browser:
    raise SystemExit("seat browser overlay must expose chrome commands")
if "empiier_native_bridge_seat_browser_command" not in bridge:
    raise SystemExit("seat browser surface must dispatch chrome commands")
if "empiierosMeet" not in auth or "meet_hostmap_unavailable" not in meet:
    raise SystemExit("FoFline Meet stage hook/surface missing")
if "EmperyDesktopSeatBrowser" not in auth or "empiierosSeatBrowser" not in seat_browser:
    raise SystemExit("FoFline Seat Browser host hook/surface missing")
if "enable_webrtc" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8"):
    raise SystemExit("Meet surface must enable WebRTC")
_root = Path(".").resolve()
ts = Path(os.environ.get(
    "ELITEEW_TS_ROOT",
    str(_root / ".." / "powered_by_emperyworld" / "pack-organization" / "trusted-substrate"),
)).resolve()
if ts.is_dir():
    frag = (ts / "config/buildroot/fragments/30-display-tauri-wayland.frag").read_text(
        encoding="utf-8"
    )
    ext = (ts / "config/buildroot/external/external.mk").read_text(encoding="utf-8")
    if "BR2_PACKAGE_WEBKITGTK_MULTIMEDIA=y" not in frag:
        raise SystemExit("Trusted Substrate must enable WebKit multimedia for Meet")
    if "ENABLE_WEB_RTC=ON" not in ext:
        raise SystemExit("Trusted Substrate must turn on WebKit ENABLE_WEB_RTC")
    bump = (ts / "config/buildroot/patches/0005-webkitgtk-2.52.6.patch").read_text(
        encoding="utf-8"
    )
    if "WEBKITGTK_VERSION = 2.52.6" not in bump or "2.54.0" in bump:
        raise SystemExit("Trusted Substrate must pin WebKitGTK 2.52.6, not 2.54")
    if "USE_LIBRICE=OFF" not in ext or "ENABLE_MEDIA_STREAM=ON" not in ext:
        raise SystemExit("Meet WebRTC must keep libnice and media-stream on 2.52")
    if "ENABLE_SPEECH_SYNTHESIS=OFF" not in ext:
        raise SystemExit("WebKit 2.52 must not require Flite speech synthesis")
    if "BR2_PACKAGE_GST1_PLUGINS_GOOD_PLUGIN_RTP=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-good RTP (rtpfunnel)")
    if "BR2_PACKAGE_GST1_PLUGINS_GOOD_PLUGIN_RTPMANAGER=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-good rtpmanager (rtpbin)")
    if "BR2_PACKAGE_GST1_PLUGINS_BASE_PLUGIN_VIDEORATE=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-base videorate")
    if "BR2_PACKAGE_GST1_PLUGINS_BASE_PLUGIN_VIDEOTESTSRC=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-base videotestsrc")
    if "BR2_PACKAGE_GST1_PLUGINS_BASE_PLUGIN_AUDIOTESTSRC=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-base audiotestsrc")
    if "BR2_PACKAGE_GST1_PLUGINS_GOOD_PLUGIN_INTERLEAVE=y" not in frag:
        raise SystemExit("Trusted Substrate must enable gst-plugins-good interleave")
    dtls = (
        ts / "config/buildroot/patches/package-dropins/webkitgtk"
        / "0002-webrtc-dtls-pem-before-sdp.patch"
    ).read_text(encoding="utf-8")
    if "Propagated DTLS PEM to webrtcbin" not in dtls:
        raise SystemExit("WebKit 2.52.6 must propagate DTLS PEM before SDP")
    if "dtlsAcceptPeerCertificate" not in dtls or "dtlssrtpenc" not in dtls:
        raise SystemExit("WebKit DTLS PEM must cover encoder and accept peer cert")
    if "X509_digest" not in dtls or "sha-256" not in dtls:
        raise SystemExit("WebKit generateCertificate must fill sha-256 fingerprints")
    if "2.54" in dtls:
        raise SystemExit("Meet DTLS dropin must stay on 2.52.6, not 2.54")
    if "0002-webrtc-dtls-pem-before-sdp" not in ext:
        raise SystemExit("external.mk must name the WebKit DTLS PEM dropin")
proxy = Path("runtime/shell-gui/fofline_api_proxy.c").read_text(encoding="utf-8")
if "empiier_fofline_api_proxy" not in proxy or "/api/v1/" not in proxy:
    raise SystemExit("FoFline API scheme proxy missing")
if '"Origin"' not in proxy or "https://app.eliteew.ru" not in proxy:
    raise SystemExit("FoFline API proxy must hop as app.eliteew.ru")
if 'soup_message_headers_replace(headers, "X-SSO-Replace-Confirm", "1")' not in proxy:
    raise SystemExit("FoFline API proxy must plant SSO replace confirm")
if "empiierosOidc" not in auth or "hostOidc('refresh')" not in auth:
    raise SystemExit("FoFline product OIDC refresh must call Runtime, not replay bootstrap")
secure = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
if "fofline edge start" not in secure or "__empiierBootSrc" not in secure:
    raise SystemExit("FoFline edge observe / boot-err source missing")
bridge = Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
if "fofline product meet navigation kind=iframe" not in bridge:
    raise SystemExit("product decide_policy must allow Meet iframe subframes")
if "fofline product media permission allowed" not in Path(
    "runtime/shell-gui/meet_surface.c"
).read_text(encoding="utf-8"):
    raise SystemExit("product WebView must allow Meet iframe media")
meet_bounds = Path("runtime/shell-gui/meet_bounds.c").read_text(encoding="utf-8")
if "meet_stage_bounds_invalid" not in meet_bounds:
    raise SystemExit("Meet stage must clip from pack bounds")
if "gtk_widget_set_halign(meet_box, GTK_ALIGN_FILL)" in secure:
    raise SystemExit("Meet overlay must not FILL the product WebView")
if "gtk_widget_set_halign(meet_box, GTK_ALIGN_START)" not in secure:
    raise SystemExit("Meet overlay must START-align for bounds clip")
if "setMeetStageBounds" not in auth or "meet_stage_bounds_required" not in auth:
    raise SystemExit("FoFline Meet hook must take bounds, not a bare URL")
if "enter-fullscreen" not in Path("runtime/shell-gui/meet_surface.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("Meet WebView must swallow JavaScript fullscreen")
meet_src = Path("runtime/shell-gui/meet_surface.c").read_text(encoding="utf-8")
if "empiierosMeetDiag" not in meet_src or "setRemoteDescription" not in meet_src:
    raise SystemExit("Meet WebView must log RTCPeerConnection failures")
if "dummyTracks" in meet_src or "ensureTracks" in meet_src or "addTrack" in meet_src:
    raise SystemExit("Meet WebView must not add dummy tracks")
if "webkit_web_view_terminate_web_process" not in meet_src:
    raise SystemExit("Meet hide must recycle the WebProcess")
if "fofline meet stage process recycled" not in meet_src:
    raise SystemExit("Meet hide must log process recycled")
if "addTransceiver" not in meet_src or "direction:'sendrecv'" not in meet_src:
    raise SystemExit("Meet inject must add sendrecv transceivers before createAnswer")
if "lowerCodecs" not in meet_src or "toLowerCase" not in meet_src:
    raise SystemExit("Meet WebView must lowercase WebKit rtpmap codecs for Jitsi")
if "wrapGet" not in meet_src or "localDescription" not in meet_src:
    raise SystemExit("Meet WebView must fixDesc localDescription getters")
if "fofline meet new-window ignored" not in meet_src:
    raise SystemExit("Meet new-window must not navigate the live conference")
if "empiier_native_bridge_conceal_meet" not in meet_src:
    raise SystemExit("Meet hide-on-small-bounds must not blank the WebView")
if "empiier_overlay_park" not in meet_src or "empiier_overlay_raise" not in meet_src:
    raise SystemExit("Meet conceal/show must park overlay input")
bounds_src = Path("runtime/shell-gui/meet_bounds.c").read_text(encoding="utf-8")
if "empiier_native_bridge_conceal_meet" not in bounds_src:
    raise SystemExit("small Meet bounds must conceal, not blank")
if bounds_src.count("empiier_native_bridge_hide_meet") != 0:
    raise SystemExit("meet_bounds.c must not blank Meet on a small hole")
if "user-content-manager" not in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("Meet WebView must own a UserContentManager for diag")
if "reason=%s" not in proxy or "copy_json_reason" not in proxy:
    raise SystemExit("FoFline API proxy must log small 4xx reason codes")
if (
    "path_is_content_download" not in proxy
    or "content=octet-stream" not in proxy
    or "api_proxy_partial_content" not in proxy
    or "Access-Control-Allow-Origin" not in proxy
    or "Access-Control-Allow-Credentials" not in proxy
    or "sniff_audio_type" not in proxy
    or "media_range" not in proxy
):
    raise SystemExit("FoFline API proxy must serve content downloads as octet-stream")
if "__empiierosAudioBlob" not in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
) or "__empiierosNeedLoad" not in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("FoFline audio must fetch content downloads as blob URLs")
if "empiier_boot_splash_attach" not in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("OS start splash must attach on the GtkOverlay")
splash = Path("runtime/shell-gui/boot_splash.c").read_text(encoding="utf-8")
if "alsasink" not in splash or "SPLASH_VOLUME 0.25" not in splash:
    raise SystemExit("boot splash must play ALSA at 25 percent")
if "empiier_product_chrome_show" not in splash or "revealed hud" not in splash:
    raise SystemExit("boot splash must reveal HUD with show_all")
if "gtk_widget_show(splash->product_bar)" in splash:
    raise SystemExit("boot splash HUD reveal must use show_all")
if not Path("runtime/shell-gui/product-chrome/start_os.mp4").is_file():
    raise SystemExit("boot splash video start_os.mp4 missing")
if "gtk_overlay_add_overlay" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8"):
    raise SystemExit("product chrome must ride the GTK overlay")
secure_overlay = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
if "gtk_overlay_set_overlay_pass_through" not in secure_overlay:
    raise SystemExit("hidden WebKit overlays must start pass-through")
if "empiier_overlay_park" not in Path("runtime/shell-gui/native_bridge.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("overlay park helper missing")
if "gdk_window_input_shape_combine_region" not in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8"):
    raise SystemExit("parked WebKit overlay must drop the input shape")
if "empiier_overlay_park" not in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
):
    raise SystemExit("auth hide must park overlay input")
if "kind=card" not in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8"):
    raise SystemExit("auth show must stay a card overlay, not a second window")
auth_bounds = Path("runtime/shell-gui/auth_bounds.c").read_text(encoding="utf-8")
if "empiieros-auth-dimmer" not in auth_bounds or "GTK_ALIGN_CENTER" not in auth_bounds:
    raise SystemExit("auth card dimmer/center missing")
if "gtk_window_new" in auth_bounds or "gtk_window_unfullscreen" in auth_bounds:
    raise SystemExit("auth card must not open or unfullscreen a top-level")
if "empiier_overlay_park" not in Path(
    "runtime/shell-gui/seat_browser_surface.c"
).read_text(encoding="utf-8"):
    raise SystemExit("seat browser hide must park overlay input")
if (
    "gtk_stack_new" not in Path("runtime/shell-gui/seat_browser_surface.c").read_text(
        encoding="utf-8"
    )
    or "related-view" not in Path("runtime/shell-gui/seat_browser_surface.c").read_text(
        encoding="utf-8"
    )
    or "seat.browser.tab.open" not in Path("runtime/shell-gui/seat_browser_surface.c").read_text(
        encoding="utf-8"
    )
    or "EMPIIEROS_BROWSER_TAB_CAP" not in Path("runtime/shell-gui/native_bridge.h").read_text(
        encoding="utf-8"
    )
):
    raise SystemExit("seat browser tabs must be GtkStack of related WebViews, not a URL list")
if (
    "seat.browser.download.begin" not in Path(
        "runtime/shell-gui/seat_browser_surface.c"
    ).read_text(encoding="utf-8")
    or "downloads not wired" in Path("runtime/shell-gui/seat_browser_surface.c").read_text(
        encoding="utf-8"
    )
):
    raise SystemExit("seat browser downloads must ask Runtime before WebKit destination")
if (
    "empiieros-seat-panel" not in chrome
    or "always-on seat chrome" not in chrome
    or "gnss_not_on_seat" not in chrome
    or "gtk_popover_set_modal" not in chrome
    or "chrome_input_shape" not in chrome
    or "movable no-peek" not in chrome
    or "GTK_POS_LEFT" not in chrome
    or "chrome_peek_height" in chrome
    or "chrome_apply_clip" in chrome
    or "chrome_schedule_peek" in chrome
    or "gtk_scrolled_window_new" in chrome
    or "CHROME_PEEK_VISIBLE_DEN" in chrome
    or "chrome_peek_margin" in chrome
    or "schedule_collapse" in chrome
    or "margin-top: -48px" in chrome
    or "gtk_widget_set_margin_end" not in chrome
    or "click-launcher.wav" not in chrome
    or '"aplay"' not in chrome
    or "chrome_hide_flyouts" not in chrome
    or "HUD_FLY_VOLUME" not in chrome
    or "Открыть EliteEW" not in chrome
    or "on_panel_clicked" in chrome[
        chrome.find("static void on_volume_clicked") : chrome.find(
            "static void on_shield_icon_clicked"
        )
    ]
):
    raise SystemExit("product chrome always-on seat panel/click media missing")
if not Path("runtime/shell-gui/product-chrome/launcher.png").is_file():
    raise SystemExit("product chrome launcher icon missing")
if not Path("runtime/shell-gui/product-chrome/click-eliteew.wav").is_file():
    raise SystemExit("product chrome EliteEW click missing")

def _wav_peak(path: Path) -> int:
    import struct
    import wave

    with wave.open(str(path), "rb") as wav:
        raw = wav.readframes(wav.getnframes())
    samples = struct.unpack("<" + "h" * (len(raw) // 2), raw)
    return max(abs(sample) for sample in samples) if samples else 0

if _wav_peak(Path("runtime/shell-gui/product-chrome/click-eliteew.wav")) < 8000:
    raise SystemExit("click-eliteew.wav is inaudible on ALSA")
if _wav_peak(Path("runtime/shell-gui/product-chrome/click-launcher.wav")) < 8000:
    raise SystemExit("click-launcher.wav is inaudible on ALSA")
print("[package] native overlays clear no_show_all before show_all")
PY

printf '[package] hardened native Shell bridge cross-build\n'
read -r -a CFLAGS <<<"$("$PKG_CONFIG" --cflags webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0)"
read -r -a LIBS <<<"$("$PKG_CONFIG" --libs webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0)"
GUI_BIN="$(mktemp -p "$ELITEEW_TS_WORK_ROOT" empiieros-shell-gui.XXXXXX)"
STAGE="$(mktemp -d -p "$ELITEEW_TS_WORK_ROOT" empiieros-overlay.XXXXXX)"
cleanup() {
  [[ -e "$GUI_BIN" ]] && /usr/bin/rm -f -- "$GUI_BIN"
  [[ -n "${AUDIO_HELPER:-}" && -e "$AUDIO_HELPER" ]] && /usr/bin/rm -f -- "$AUDIO_HELPER"
  [[ -n "${WG_BIN:-}" && -e "$WG_BIN" ]] && /usr/bin/rm -f -- "$WG_BIN"
  [[ -d "$STAGE" ]] && /usr/bin/rm -rf -- "$STAGE"
  return 0
}
trap cleanup EXIT INT TERM

"$CC" \
  -std=c11 \
  -O2 \
  -D_FORTIFY_SOURCE=2 \
  -fstack-protector-strong \
  -fPIE \
  -Wall \
  -Wextra \
  -Werror \
  "${CFLAGS[@]}" \
  runtime/shell-gui/secure_main.c \
  runtime/shell-gui/native_bridge.c \
  runtime/shell-gui/boot_splash.c \
  runtime/shell-gui/product_chrome.c \
  runtime/shell-gui/auth_surface.c \
  runtime/shell-gui/auth_bounds.c \
  runtime/shell-gui/seat_browser_surface.c \
  runtime/shell-gui/meet_surface.c \
  runtime/shell-gui/meet_bounds.c \
  runtime/shell-gui/fofline_api_proxy.c \
  "${LIBS[@]}" \
  -pie \
  -Wl,-z,relro,-z,now \
  -o "$GUI_BIN"
"$STRIP" "$GUI_BIN"

for binary in "$RUNTIME_BIN" "$AIR_BIN" "$PORTAL_BIN" "$SPA_BIN" "$SEAT_FW_BIN" "$WG_BIN" "$GUI_BIN"; do
  "$OBJDUMP" -f "$binary" | rg -q 'file format elf64-x86-64' || {
    echo "unexpected target binary format: $binary" >&2
    exit 1
  }
done

install -d \
  "$STAGE/usr/local/bin" \
  "$STAGE/opt/empiieros/shell" \
  "$STAGE/opt/empiieros"
printf '[package] allowlisted ALSA volume helper\n'
AUDIO_HELPER="$(mktemp -p "$ELITEEW_TS_WORK_ROOT" empiieros-audio-volume.XXXXXX)"
read -r -a ALSA_CFLAGS <<<"$("$PKG_CONFIG" --cflags alsa)"
read -r -a ALSA_LIBS <<<"$("$PKG_CONFIG" --libs alsa)"
"$CC" \
  -std=c11 \
  -O2 \
  -D_FORTIFY_SOURCE=2 \
  -fstack-protector-strong \
  -fPIE \
  -Wall \
  -Wextra \
  -Werror \
  "${ALSA_CFLAGS[@]}" \
  runtime/helpers/empiieros-audio-volume.c \
  runtime/helpers/hda_speaker_amp.c \
  "${ALSA_LIBS[@]}" \
  -pie \
  -Wl,-z,relro,-z,now \
  -o "$AUDIO_HELPER"
"$STRIP" "$AUDIO_HELPER"
"$OBJDUMP" -f "$AUDIO_HELPER" | rg -q 'file format elf64-x86-64' || {
  echo "unexpected target binary format: $AUDIO_HELPER" >&2
  exit 1
}

install -m 0755 "$RUNTIME_BIN" "$STAGE/usr/local/bin/empiieros-runtime"
install -m 0755 "$GUI_BIN" "$STAGE/usr/local/bin/empiieros-shell-gui"
install -d \
  "$STAGE/usr/local/sbin" \
  "$STAGE/etc/systemd/system" \
  "$STAGE/etc/systemd/system/multi-user.target.wants"
install -m 0755 "$AIR_BIN" "$STAGE/usr/local/sbin/empiieros-air-enroll"
install -m 0644 runtime/helpers/empiieros-air-enroll.service \
  "$STAGE/etc/systemd/system/empiieros-air-enroll.service"
install -m 0755 "$PORTAL_BIN" "$STAGE/usr/local/sbin/empiieros-portal-up"
install -m 0644 runtime/helpers/empiieros-portal-up.service \
  "$STAGE/etc/systemd/system/empiieros-portal-up.service"
install -m 0755 "$SPA_BIN" "$STAGE/usr/local/sbin/empiieros-spa-fetch"
install -m 0644 runtime/helpers/empiieros-spa-fetch.service \
  "$STAGE/etc/systemd/system/empiieros-spa-fetch.service"
install -m 0755 "$OTA_BIN" "$STAGE/usr/local/sbin/empiieros-ota-fetch"
install -m 0644 runtime/helpers/empiieros-ota-fetch.service \
  "$STAGE/etc/systemd/system/empiieros-ota-fetch.service"
install -d "$STAGE/etc/systemd/system/empiieros-runtime.service.d"
install -m 0644 runtime/helpers/ota-cutover.conf \
  "$STAGE/etc/systemd/system/empiieros-runtime.service.d/ota-cutover.conf"
install -m 0755 "$WG_BIN" "$STAGE/usr/local/sbin/empiieros-wg"
install -m 0755 "$AUDIO_HELPER" "$STAGE/usr/local/sbin/empiieros-audio-volume"
install -m 0644 runtime/helpers/empiieros-audio-volume.service \
  "$STAGE/etc/systemd/system/empiieros-audio-volume.service"
install -m 0644 runtime/helpers/empiieros-audio-amp.service \
  "$STAGE/etc/systemd/system/empiieros-audio-amp.service"
# Overlay forbids symlinks — wants marker is a regular-file copy.
# post-build replaces it with a real symlink (same Seat Shield pattern).
install -m 0644 runtime/helpers/empiieros-audio-amp.service \
  "$STAGE/etc/systemd/system/multi-user.target.wants/empiieros-audio-amp.service"
install -m 0755 runtime/helpers/empiieros-wifi-radio \
  "$STAGE/usr/local/sbin/empiieros-wifi-radio"
install -m 0755 runtime/helpers/empiieros-time-sync \
  "$STAGE/usr/local/sbin/empiieros-time-sync"
install -m 0755 runtime/helpers/empiieros-wifi-restore \
  "$STAGE/usr/local/sbin/empiieros-wifi-restore"
printf '[package] Seat Shield nft table + libexec helper\n'
install -d \
  "$STAGE/usr/libexec" \
  "$STAGE/usr/share/empiieros/seat-shield" \
  "$STAGE/etc/systemd/system/multi-user.target.wants"
install -m 0755 "$SEAT_FW_BIN" "$STAGE/usr/libexec/empiieros-seat-shield-fw"
install -m 0755 "$SEAT_WATCH_BIN" "$STAGE/usr/libexec/empiieros-seat-shield-watch"
install -m 0755 "$SEAT_PROBE_BIN" "$STAGE/usr/libexec/empiieros-seat-shield-probe"
install -m 0644 runtime/helpers/seat-shield/seat_shield.nft \
  "$STAGE/usr/share/empiieros/seat-shield/seat_shield.nft"
install -m 0644 runtime/helpers/seat-shield/README.md \
  "$STAGE/usr/share/empiieros/seat-shield/README.md"
install -m 0644 runtime/helpers/seat-shield/empiieros-seat-shield-nftables.service \
  "$STAGE/etc/systemd/system/empiieros-seat-shield-nftables.service"
# Overlay forbids symlinks — install a wants copy (Substrate prelicense pattern).
install -m 0644 runtime/helpers/seat-shield/empiieros-seat-shield-nftables.service \
  "$STAGE/etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-nftables.service"
install -m 0644 runtime/helpers/seat-shield/empiieros-seat-shield-watch.service \
  "$STAGE/etc/systemd/system/empiieros-seat-shield-watch.service"
install -m 0644 runtime/helpers/seat-shield/empiieros-seat-shield-watch.service \
  "$STAGE/etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-watch.service"
# Drop stale Vite bundles. A reused overlay otherwise keeps old assets/*.js,
# and the installer clock gate (`find | head -n 1`) can miss the live bundle.
rm -rf "$STAGE/opt/empiieros/shell/assets"
install -d "$STAGE/opt/empiieros/shell"
cp -a shell-ui/dist/. "$STAGE/opt/empiieros/shell/"
install -d \
  "$STAGE/usr/share/empiieros/product-chrome" \
  "$STAGE/opt/empiieros/shell/product-chrome"
install -m 0644 \
  runtime/shell-gui/product-chrome/launcher.png \
  runtime/shell-gui/product-chrome/eliteew.png \
  runtime/shell-gui/product-chrome/hud-panel.png \
  runtime/shell-gui/product-chrome/hud-shield.png \
  runtime/shell-gui/product-chrome/hud-shield-alert.png \
  runtime/shell-gui/product-chrome/hud-ota.png \
  runtime/shell-gui/product-chrome/hud-volume.png \
  runtime/shell-gui/product-chrome/panel-mic.png \
  runtime/shell-gui/product-chrome/panel-cam.png \
  runtime/shell-gui/product-chrome/panel-gps.png \
  runtime/shell-gui/product-chrome/start_os.mp4 \
  runtime/shell-gui/product-chrome/click-launcher.wav \
  runtime/shell-gui/product-chrome/click-eliteew.wav \
  runtime/shell-gui/product-chrome/ORIGIN.md \
  "$STAGE/usr/share/empiieros/product-chrome/"
install -m 0644 \
  runtime/shell-gui/product-chrome/launcher.png \
  runtime/shell-gui/product-chrome/eliteew.png \
  runtime/shell-gui/product-chrome/hud-panel.png \
  runtime/shell-gui/product-chrome/hud-shield.png \
  runtime/shell-gui/product-chrome/hud-shield-alert.png \
  runtime/shell-gui/product-chrome/hud-ota.png \
  runtime/shell-gui/product-chrome/hud-volume.png \
  runtime/shell-gui/product-chrome/panel-mic.png \
  runtime/shell-gui/product-chrome/panel-cam.png \
  runtime/shell-gui/product-chrome/panel-gps.png \
  runtime/shell-gui/product-chrome/start_os.mp4 \
  runtime/shell-gui/product-chrome/click-launcher.wav \
  runtime/shell-gui/product-chrome/click-eliteew.wav \
  runtime/shell-gui/product-chrome/ORIGIN.md \
  "$STAGE/opt/empiieros/shell/product-chrome/"
find "$STAGE/opt/empiieros/shell" -type d -exec chmod 0755 {} +
find "$STAGE/opt/empiieros/shell" -type f -exec chmod 0644 {} +
if find "$STAGE" -type l -print -quit | rg -q .; then
  echo "generated overlay contains a forbidden symbolic link" >&2
  exit 1
fi

if [[ -n "$OPERATOR_KEY" ]]; then
  install -d -m 0700 "$STAGE/home/empiieros-maint/.ssh"
  install -m 0600 "$OPERATOR_KEY" "$STAGE/home/empiieros-maint/.ssh/authorized_keys"
fi

python3 - "$ROOT" "$STAGE" "$CC" "$OPERATOR_KEY" <<'PY'
from __future__ import annotations

from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1])
stage = Path(sys.argv[2])
compiler = Path(sys.argv[3])
key_path = Path(sys.argv[4]) if sys.argv[4] else None

inputs = [
    root / "scripts/package-shell-gui-rootfs.sh",
    root / "runtime/Cargo.toml",
    root / "runtime/Cargo.lock",
    root / "runtime/src/core.rs",
    root / "runtime/src/daemon.rs",
    root / "runtime/src/protocol.rs",
    root / "runtime/src/state_store.rs",
    root / "runtime/src/command_runner.rs",
    root / "runtime/src/system_probe.rs",
    root / "runtime/src/network_control.rs",
    root / "runtime/src/wifi_control.rs",
    root / "runtime/src/power_control.rs",
    root / "runtime/src/observed_domains.rs",
    root / "runtime/src/event_log.rs",
    root / "runtime/src/capsule_v3.rs",
    root / "runtime/src/device_request.rs",
    root / "runtime/src/launcher_air.rs",
    root / "runtime/src/launcher_capsule.rs",
    root / "runtime/src/launcher_pin.rs",
    root / "runtime/src/launcher_status.rs",
    root / "runtime/src/bin/air_enroll_helper.rs",
    root / "runtime/src/bin/portal_up_helper.rs",
    root / "runtime/src/launcher_portal.rs",
    root / "runtime/src/launcher_inner.rs",
    root / "runtime/src/wg_conf.rs",
    root / "runtime/helpers/empiieros-air-enroll.service",
    root / "runtime/helpers/empiieros-portal-up.service",
    root / "runtime/src/bin/spa_fetch_helper.rs",
    root / "runtime/src/launcher_spa.rs",
    root / "runtime/src/bin/ota_fetch_helper.rs",
    root / "runtime/src/launcher_ota.rs",
    root / "runtime/src/ota_cutover.rs",
    root / "runtime/src/ota_capsule_store.rs",
    root / "runtime/helpers/empiieros-ota-fetch.service",
    root / "runtime/helpers/ota-cutover.conf",
    root / "runtime/src/spa_pack.rs",
    root / "runtime/src/https_vip.rs",
    root / "runtime/src/launcher_product.rs",
    root / "runtime/src/portal_hostmap.rs",
    root / "runtime/helpers/empiieros-spa-fetch.service",
    root / "runtime/helpers/empiieros-audio-volume.c",
    root / "runtime/helpers/hda_speaker_amp.c",
    root / "runtime/helpers/hda_speaker_amp.h",
    root / "runtime/helpers/empiieros-audio-volume.service",
    root / "runtime/helpers/empiieros-audio-amp.service",
    root / "runtime/src/seat_shield.rs",
    root / "runtime/seat_shield_core/Cargo.toml",
    root / "runtime/seat_shield_core/Cargo.lock",
    root / "runtime/seat_shield_probe/Cargo.toml",
    root / "runtime/seat_shield_probe/Cargo.lock",
    root / "runtime/helpers/seat-shield/seat_shield.nft",
    root / "runtime/helpers/seat-shield/README.md",
    root / "runtime/helpers/seat-shield/empiieros-seat-shield-nftables.service",
    root / "runtime/helpers/seat-shield/empiieros-seat-shield-watch.service",
    root / "runtime/shell-gui/secure_main.c",
    root / "runtime/shell-gui/native_bridge.c",
    root / "runtime/shell-gui/native_bridge.h",
    root / "runtime/shell-gui/boot_splash.c",
    root / "runtime/shell-gui/boot_splash.h",
    root / "runtime/shell-gui/product_chrome.c",
    root / "runtime/shell-gui/auth_surface.c",
    root / "runtime/shell-gui/auth_bounds.c",
    root / "runtime/shell-gui/auth_bounds.h",
    root / "runtime/shell-gui/seat_browser_surface.c",
    root / "runtime/shell-gui/meet_surface.c",
    root / "runtime/shell-gui/meet_bounds.c",
    root / "runtime/shell-gui/meet_bounds.h",
    root / "runtime/shell-gui/fofline_api_proxy.c",
    root / "runtime/shell-gui/fofline_api_proxy.h",
    root / "runtime/shell-gui/product-chrome/launcher.png",
    root / "runtime/shell-gui/product-chrome/eliteew.png",
    root / "runtime/shell-gui/product-chrome/hud-panel.png",
    root / "runtime/shell-gui/product-chrome/hud-shield.png",
    root / "runtime/shell-gui/product-chrome/hud-ota.png",
    root / "runtime/shell-gui/product-chrome/hud-volume.png",
    root / "runtime/shell-gui/product-chrome/click-launcher.wav",
    root / "runtime/shell-gui/product-chrome/click-eliteew.wav",
    root / "runtime/shell-gui/product-chrome/start_os.mp4",
    root / "runtime/shell-gui/product-chrome/ORIGIN.md",
    root / "runtime/src/launcher_oidc.rs",
    root / "runtime/src/oidc_urls.rs",
    root / "runtime/src/oidc_pkce.rs",
    root / "runtime/src/oidc_dpop.rs",
    root / "runtime/src/https_auth.rs",
    root / "shell-ui/package.json",
    root / "shell-ui/package-lock.json",
    root / "shell-ui/index.html",
    root / "shell-ui/vite.config.ts",
]
inputs.extend(sorted((root / "shell-ui/src").rglob("*")))
inputs.extend(sorted((root / "runtime/seat_shield_core/src").rglob("*")))
inputs.extend(sorted((root / "runtime/seat_shield_probe/src").rglob("*")))
inputs = [path for path in inputs if path.is_file()]


def digest(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


manifest = {
    "schema": 1,
    "profile": "empiieros-shell",
    "source_commit": subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip(),
    "source_tree_dirty": bool(
        subprocess.run(
            ["git", "status", "--porcelain=v1", "--untracked-files=all"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    ),
    "source_inputs": {
        str(path.relative_to(root)): digest(path)
        for path in sorted(set(inputs))
    },
    "outputs": {
        "usr/local/bin/empiieros-runtime": digest(stage / "usr/local/bin/empiieros-runtime"),
        "usr/local/bin/empiieros-shell-gui": digest(stage / "usr/local/bin/empiieros-shell-gui"),
        "usr/local/sbin/empiieros-air-enroll": digest(stage / "usr/local/sbin/empiieros-air-enroll"),
        "etc/systemd/system/empiieros-air-enroll.service": digest(
            stage / "etc/systemd/system/empiieros-air-enroll.service"
        ),
        "usr/local/sbin/empiieros-portal-up": digest(stage / "usr/local/sbin/empiieros-portal-up"),
        "etc/systemd/system/empiieros-portal-up.service": digest(
            stage / "etc/systemd/system/empiieros-portal-up.service"
        ),
        "usr/local/sbin/empiieros-spa-fetch": digest(stage / "usr/local/sbin/empiieros-spa-fetch"),
        "etc/systemd/system/empiieros-spa-fetch.service": digest(
            stage / "etc/systemd/system/empiieros-spa-fetch.service"
        ),
        "usr/local/sbin/empiieros-ota-fetch": digest(stage / "usr/local/sbin/empiieros-ota-fetch"),
        "etc/systemd/system/empiieros-ota-fetch.service": digest(
            stage / "etc/systemd/system/empiieros-ota-fetch.service"
        ),
        "etc/systemd/system/empiieros-runtime.service.d/ota-cutover.conf": digest(
            stage / "etc/systemd/system/empiieros-runtime.service.d/ota-cutover.conf"
        ),
        "usr/local/sbin/empiieros-audio-volume": digest(
            stage / "usr/local/sbin/empiieros-audio-volume"
        ),
        "etc/systemd/system/empiieros-audio-volume.service": digest(
            stage / "etc/systemd/system/empiieros-audio-volume.service"
        ),
        "etc/systemd/system/empiieros-audio-amp.service": digest(
            stage / "etc/systemd/system/empiieros-audio-amp.service"
        ),
        "etc/systemd/system/multi-user.target.wants/empiieros-audio-amp.service": digest(
            stage
            / "etc/systemd/system/multi-user.target.wants/empiieros-audio-amp.service"
        ),
        "usr/local/sbin/empiieros-wg": digest(stage / "usr/local/sbin/empiieros-wg"),
        "usr/libexec/empiieros-seat-shield-fw": digest(
            stage / "usr/libexec/empiieros-seat-shield-fw"
        ),
        "usr/libexec/empiieros-seat-shield-watch": digest(
            stage / "usr/libexec/empiieros-seat-shield-watch"
        ),
        "usr/libexec/empiieros-seat-shield-probe": digest(
            stage / "usr/libexec/empiieros-seat-shield-probe"
        ),
        "usr/share/empiieros/seat-shield/seat_shield.nft": digest(
            stage / "usr/share/empiieros/seat-shield/seat_shield.nft"
        ),
        "etc/systemd/system/empiieros-seat-shield-nftables.service": digest(
            stage / "etc/systemd/system/empiieros-seat-shield-nftables.service"
        ),
        "etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-nftables.service": digest(
            stage
            / "etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-nftables.service"
        ),
        "etc/systemd/system/empiieros-seat-shield-watch.service": digest(
            stage / "etc/systemd/system/empiieros-seat-shield-watch.service"
        ),
        "etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-watch.service": digest(
            stage
            / "etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-watch.service"
        ),
        "opt/empiieros/shell/index.html": digest(stage / "opt/empiieros/shell/index.html"),
        "opt/empiieros/shell/product-chrome/launcher.png": digest(
            stage / "opt/empiieros/shell/product-chrome/launcher.png"
        ),
        "opt/empiieros/shell/product-chrome/eliteew.png": digest(
            stage / "opt/empiieros/shell/product-chrome/eliteew.png"
        ),
        "opt/empiieros/shell/product-chrome/hud-shield.png": digest(
            stage / "opt/empiieros/shell/product-chrome/hud-shield.png"
        ),
        "opt/empiieros/shell/product-chrome/hud-ota.png": digest(
            stage / "opt/empiieros/shell/product-chrome/hud-ota.png"
        ),
        "opt/empiieros/shell/product-chrome/hud-volume.png": digest(
            stage / "opt/empiieros/shell/product-chrome/hud-volume.png"
        ),
        "opt/empiieros/shell/product-chrome/click-launcher.wav": digest(
            stage / "opt/empiieros/shell/product-chrome/click-launcher.wav"
        ),
        "opt/empiieros/shell/product-chrome/click-eliteew.wav": digest(
            stage / "opt/empiieros/shell/product-chrome/click-eliteew.wav"
        ),
        "usr/share/empiieros/product-chrome/launcher.png": digest(
            stage / "usr/share/empiieros/product-chrome/launcher.png"
        ),
    },
    "toolchain": subprocess.run(
        [str(compiler), "--version"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()[0],
    "operator_ssh_key_sha256": digest(key_path) if key_path is not None else None,
}
(stage / "opt/empiieros/build-manifest.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY

MANIFEST_SHA="$(sha256sum "$STAGE/opt/empiieros/build-manifest.json" | awk '{print $1}')"
STORE="$ELITEEW_TS_WORK_ROOT/empiieros-overlays"
DEST="$STORE/$MANIFEST_SHA"
CURRENT="$ELITEEW_TS_WORK_ROOT/empiieros-rootfs-overlay"
mkdir -p "$STORE"
if [[ -e "$DEST" ]]; then
  /usr/bin/rm -rf -- "$STAGE"
else
  mv "$STAGE" "$DEST"
fi
STAGE=""
ln -sfn "$DEST" "${CURRENT}.next"
mv -Tf "${CURRENT}.next" "$CURRENT"

python3 - "$DEST" <<'PY'
from hashlib import sha256
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
manifest = json.loads((root / "opt/empiieros/build-manifest.json").read_text(encoding="utf-8"))
for relative, expected in manifest["outputs"].items():
    observed = sha256((root / relative).read_bytes()).hexdigest()
    if observed != expected:
        raise SystemExit(f"overlay output digest mismatch: {relative}")
PY

printf '[package] overlay=%s\n' "$DEST"
printf '[package] manifest_sha256=%s\n' "$MANIFEST_SHA"

if [[ "$BUILD_IMAGE" -eq 1 ]]; then
  ELITEEW_TS_EMPIIEROS_OVERLAY="$CURRENT" \
    bash "$SUBSTRATE_ROOT/scripts/buildroot-build.sh" --resume
fi

printf 'PASS  deterministic EmpiierOS overlay prepared\n'
