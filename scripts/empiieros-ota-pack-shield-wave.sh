#!/usr/bin/env bash
# Pack + optional publish os_component OTA: shell UI + shell-gui + runtime + helpers.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PUBLISH=0
APP_BUILD_ID="rights-about-20260923"
VERSION="0.8.6-rights-about"
OUT_DIR=""

usage() {
  cat <<'EOF'
Usage: scripts/empiieros-ota-pack-shield-wave.sh [--publish] [--out-dir DIR]

Builds Buildroot-cross empiieros-runtime + empiieros-shell-gui + seat-shield
helpers, shell-ui dist, stages runtime/ shell/ helpers/ tree, packs signed
os_component capsule.

  --publish   ingest into prod-core (EMPIERO_OS_OTA_PUBLISH_SSH) or local stack
  --out-dir   pack output (default: $TMPDIR/empiieros-ota-shield-$$)

Seat: Updates → Request → confirm APPLY_OTA. Includes runtime restart (deferred).
Shell assets reload via ota_reload_shell after apply. GUI binary restart when
runtime/empiieros-shell-gui is in the capsule.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --publish) PUBLISH=1; shift ;;
    --out-dir)
      [[ $# -ge 2 ]] || { echo "missing --out-dir" >&2; exit 2; }
      OUT_DIR="$2"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown: $1" >&2; usage >&2; exit 2 ;;
  esac
done

WORK="${OUT_DIR:-${TMPDIR:-/tmp}/empiieros-ota-shield-$$}"
TREE="$WORK/capsule-tree"
PACK="$WORK/pack"
mkdir -p "$WORK"

cd "$ROOT"
npm --prefix shell-ui run build
python3 - <<'PY'
from pathlib import Path
import re

html = Path("shell-ui/dist/index.html").read_text(encoding="utf-8")
if "script-src empiier:" not in html:
    raise SystemExit("dist/index.html lost kiosk CSP")
js = next(Path("shell-ui/dist/assets").glob("index-*.js"))
body = js.read_text(encoding="utf-8")
if "taskbarNotifications" not in body and "seatShieldNotifyTitle" not in body:
    raise SystemExit("shell bundle missing shield notification strings")
css = next(Path("shell-ui/dist/assets").glob("index-*.css"))
if "secure-notification-toast" not in css.read_text(encoding="utf-8"):
    raise SystemExit("shell bundle missing toast styles")
if "secure-app-window-body" not in body:
    raise SystemExit("shell bundle missing host window scrollport class")
if ".secure-taskbar-notify-flyout,.secure-access-points" not in body:
    raise SystemExit("shell bundle missing viewport-locked wheel scroll fallback selectors")
# Production SecureApp always installs the DOM wheel walker (bridge or not).
if "installViewportLockedWheelScroll" not in Path("shell-ui/src/secure/SecureApp.tsx").read_text(
    encoding="utf-8"
):
    raise SystemExit("SecureApp missing viewport-locked wheel scroll install")
# GTK must not own scroll-event (swallows wheel even when returning FALSE).
gui = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
if 'g_signal_connect(view_widget, "scroll-event"' in gui or "on_scroll_event" in gui:
    raise SystemExit("secure_main still connects scroll-event; remove GTK wheel handler")
if "otaNextInstall" not in Path("shell-ui/src/secure/locale/ru.ts").read_text(encoding="utf-8"):
    raise SystemExit("OTA UI missing next-step strings")
if "ota_next_action" not in Path("runtime/src/launcher_ota.rs").read_text(encoding="utf-8"):
    raise SystemExit("launcher.ota_status missing ota_next_action")
PY

SUBSTRATE_ROOT="${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}"
[[ -d "$SUBSTRATE_ROOT" ]] || {
  echo "Trusted Substrate root not found: $SUBSTRATE_ROOT (required for cross runtime/helpers)" >&2
  exit 1
}
# shellcheck source=/dev/null
source "$SUBSTRATE_ROOT/scripts/work-root.sh"
HOST_BIN="${ELITEEW_TS_BUILDROOT_OUT}/host/bin"
CC="${HOST_BIN}/x86_64-buildroot-linux-gnu-gcc"
STRIP="${HOST_BIN}/x86_64-buildroot-linux-gnu-strip"
PKG_CONFIG="${HOST_BIN}/pkg-config"
if [[ ! -x "$CC" ]] || ! "$PKG_CONFIG" --exists webkit2gtk-4.1 gtk+-3.0 2>/dev/null; then
  printf '[ota-pack] preparing Buildroot target SDK\n'
  bash "$SUBSTRATE_ROOT/scripts/buildroot-build.sh" --prepare-empiieros-sdk
fi
[[ -x "$CC" ]] || { echo "cross gcc missing: $CC" >&2; exit 1; }

# Factory-only: Buildroot host cargo/rustc (not Fedora host rustc).
export PATH="${HOST_BIN}:${PATH}"
case "$(command -v rustc)" in
  "${HOST_BIN}/rustc") ;;
  *)
    echo "factory rustc required under ${HOST_BIN}/rustc; got: $(command -v rustc)" >&2
    exit 1
    ;;
esac

EMPIIEROS_HARDEN_RUSTFLAGS="-C relocation-model=pic -C link-arg=-Wl,-z,relro,-z,now -C link-arg=-Wl,-z,noexecstack"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }${EMPIIEROS_HARDEN_RUSTFLAGS}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/runtime/target}"
export CARGO_TARGET_DIR

printf '[ota-pack] cross runtime\n'
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin empiieros-runtime

printf '[ota-pack] cross seat-shield helpers\n'
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/seat_shield_core/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin seat-shield-fw \
    --bin seat-shield-watch

CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC" \
  cargo build \
    --locked \
    --release \
    --manifest-path runtime/seat_shield_probe/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --bin seat-shield-probe

RUNTIME_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/empiieros-runtime"
SEAT_FW_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-fw"
SEAT_WATCH_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-watch"
SEAT_PROBE_BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/seat-shield-probe"
for path in "$RUNTIME_BIN" "$SEAT_FW_BIN" "$SEAT_WATCH_BIN" "$SEAT_PROBE_BIN"; do
  [[ -x "$path" ]] || { echo "missing cross binary: $path" >&2; exit 1; }
done

"$PKG_CONFIG" --exists webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0 || {
  echo "Buildroot WebKit/GStreamer pkg-config missing" >&2
  exit 1
}
printf '[ota-pack] cross empiieros-shell-gui\n'
read -r -a GUI_CFLAGS <<<"$("$PKG_CONFIG" --cflags webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0)"
read -r -a GUI_LIBS <<<"$("$PKG_CONFIG" --libs webkit2gtk-4.1 gtk+-3.0 gstreamer-1.0 gstreamer-video-1.0)"
GUI_BIN="$WORK/empiieros-shell-gui.build"
"$CC" \
  -std=c11 \
  -O2 \
  -D_FORTIFY_SOURCE=2 \
  -fstack-protector-strong \
  -fPIE \
  -Wall \
  -Wextra \
  -Werror \
  "${GUI_CFLAGS[@]}" \
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
  "${GUI_LIBS[@]}" \
  -pie \
  -Wl,-z,relro,-z,now \
  -o "$GUI_BIN"
"$STRIP" "$GUI_BIN"
python3 - "$GUI_BIN" <<'PY'
import subprocess
import sys

blob = subprocess.check_output(["strings", sys.argv[1]], text=True, errors="replace")
# Wheel must reach WebKit/DOM — no GTK scroll-event connect (see secure_main).
if "on_scroll_event" in blob or 'scroll-event' in blob:
    raise SystemExit("shell-gui still embeds scroll-event handler symbols")
if "Do not connect scroll-event" not in blob and "overflow panels scroll via" not in blob.lower():
    # Comment may be stripped; require absence of old syslog marker path.
    if "shell scroll observed" in blob:
        raise SystemExit("shell-gui still has observe-only scroll syslog path")
print("shell-gui_scroll_policy=dom_only")
PY

rm -rf "$TREE"
mkdir -p "$TREE/runtime" "$TREE/shell" "$TREE/helpers"
install -m 0755 "$RUNTIME_BIN" "$TREE/runtime/empiieros-runtime"
install -m 0755 "$GUI_BIN" "$TREE/runtime/empiieros-shell-gui"
install -m 0755 "$SEAT_FW_BIN" "$TREE/helpers/seat-shield-fw"
install -m 0755 "$SEAT_WATCH_BIN" "$TREE/helpers/seat-shield-watch"
install -m 0755 "$SEAT_PROBE_BIN" "$TREE/helpers/seat-shield-probe"
cp -a shell-ui/dist/. "$TREE/shell/"

SK_FILE="${XDG_DATA_HOME:-$HOME/.local/share}/empiieros/ota/artifact.sk.hex"
[[ -f "$SK_FILE" ]] || { echo "OTA SK missing — run scripts/empiieros-ota-operator-key.sh init" >&2; exit 1; }

python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" pack \
  --out-dir "$PACK" \
  --channel os_component \
  --version "$VERSION" \
  --app-build-id "$APP_BUILD_ID" \
  --sk-file "$SK_FILE" \
  --src "$TREE"

python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" ingest \
  --capsule "$PACK/capsule.json" \
  --blob-dir "$PACK/blobs" \
  --pk-file "$PACK/artifact.pk.hex" \
  --verify-only

echo "PACK_OK dir=$PACK app_build_id=$APP_BUILD_ID"
sha256sum "$TREE/runtime/empiieros-runtime" | awk '{print "runtime_sha256="$1}'
sha256sum "$TREE/helpers/seat-shield-fw" | awk '{print "seat_fw_sha256="$1}'
sha256sum "$TREE/helpers/seat-shield-watch" | awk '{print "seat_watch_sha256="$1}'
sha256sum "$TREE/shell/assets/"index-*.js | awk '{print "shell_js="$1}'
sha256sum "$TREE/runtime/empiieros-shell-gui" | awk '{print "shell_gui_sha256="$1}'

if [[ "$PUBLISH" -eq 1 ]]; then
  PUBLISH_ARGS=(
    --capsule "$PACK/capsule.json"
    --blob-dir "$PACK/blobs"
    --pk-file "$PACK/artifact.pk.hex"
  )
  SSH_TARGET="${EMPIERO_OS_OTA_PUBLISH_SSH:-prod-core}"
  PUBLISH_ARGS+=(--ssh "$SSH_TARGET")
  python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" publish "${PUBLISH_ARGS[@]}"
  EDGE="${EMPIERO_OS_OTA_EDGE:-https://app.eliteew.ru/empiieros-ota}"
  curl -sf --connect-timeout 12 "$EDGE/v1/current.json" -o "$WORK/current.json"
  python3 - <<PY
import json
from pathlib import Path
doc = json.loads(Path("$WORK/current.json").read_text())
p = doc["payload"]
print("EDGE_OK app_build_id=", p.get("app_build_id"), "files=", len(p.get("files", [])))
for row in p.get("files", []):
    print(" ", row.get("path"), row.get("sha256")[:16] + "...")
PY
fi
