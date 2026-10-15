#!/usr/bin/env bash
# Plant LinkornEW onto a *copy* of factory rootfs.ext2 and boot it in QEMU.
# Does not write official out/, NVMe, or a USB stick.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS="${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}"
SRC="${ELITEEW_TS_QEMU_ROOTFS:-$TS/out/rootfs.ext2}"
KERNEL="${ELITEEW_TS_QEMU_KERNEL:-$TS/out/bzImage}"
DISK="${LINKORNEW_QEMU_DISK:-$HOME/linkornew-lab/linkornew.ext2}"
MNT="${LINKORNEW_QEMU_MNT:-$HOME/linkornew-lab/mnt}"

usage() {
  printf 'usage: %s [--no-run]\n' "$0" >&2
  exit 2
}

run=1
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-run) run=0; shift ;;
    *) usage ;;
  esac
done

command -v qemu-system-x86_64 >/dev/null || { printf 'qemu-linkornew: missing qemu-system-x86_64\n' >&2; exit 1; }
command -v fuse2fs >/dev/null || { printf 'qemu-linkornew: missing fuse2fs\n' >&2; exit 1; }
[[ -s "$SRC" ]] || { printf 'qemu-linkornew: missing factory ext2 %s\n' "$SRC" >&2; exit 1; }
[[ -s "$KERNEL" ]] || { printf 'qemu-linkornew: missing kernel %s\n' "$KERNEL" >&2; exit 1; }
[[ "$DISK" != "$SRC" ]] || { printf 'qemu-linkornew: refusing to overlay official out/\n' >&2; exit 1; }
[[ "$DISK" != "$TS/out/"* ]] || { printf 'qemu-linkornew: refusing official out/\n' >&2; exit 1; }

mkdir -p "$(dirname "$DISK")" "$MNT"
if [[ ! -f "$DISK" ]]; then
  printf 'qemu-linkornew: copying sparse factory ext2 → %s\n' "$DISK" >&2
  cp --sparse=always "$SRC" "$DISK"
fi

if mountpoint -q "$MNT" 2>/dev/null; then
  fusermount -u "$MNT" 2>/dev/null || fusermount3 -u "$MNT"
fi
fuse2fs -o fakeroot,rw "$DISK" "$MNT"
cleanup() {
  if mountpoint -q "$MNT" 2>/dev/null; then
    fusermount -u "$MNT" 2>/dev/null || fusermount3 -u "$MNT" || true
  fi
}
trap cleanup EXIT

# Plant CP during assemble when set (deck + cpython + controller + sentinel).
if [[ -n "${LINKORNEW_COMMAND_POST:-}" ]]; then
  export LINKORNEW_COMMAND_POST
  printf 'qemu-linkornew: LINKORNEW_COMMAND_POST=%s\n' "$LINKORNEW_COMMAND_POST" >&2
fi

"$ROOT/runtime/linkornew-seat/assemble-linkornew-rootfs.sh" --overlay "$MNT"
"$ROOT/runtime/linkornew-seat/plant-chromium.sh" lab-overlay "$MNT"

# Tokens + ensure CP units if plant was requested.
if [[ -n "${LINKORNEW_COMMAND_POST:-}" ]]; then
  # Fallback plant if assemble skipped (old tree without hook).
  if [[ ! -x "$MNT/usr/lib/linkornew/cp-controller-run.sh" ]]; then
    CP="$LINKORNEW_COMMAND_POST"
    plant="$ROOT/runtime/linkornew-seat/command-post"
    "$plant/plant-cpython.sh" "$MNT"
    "$plant/plant-deck.sh" "$CP/command-deck/dist" "$MNT"
    "$plant/plant-controller.sh" "$CP" "$MNT"
    "$plant/plant-sentinel.sh" "$CP" "$MNT"
    install -m 0755 "$plant/cp-controller-run.sh" "$MNT/usr/lib/linkornew/cp-controller-run.sh"
    install -m 0755 "$plant/cp-sentinel-run.sh" "$MNT/usr/lib/linkornew/cp-sentinel-run.sh"
    [[ -x "$ROOT/runtime/linkornew-seat/command-post/cp-deck-serve" ]] \
      && install -m 0755 "$ROOT/runtime/linkornew-seat/command-post/cp-deck-serve" \
        "$MNT/usr/lib/linkornew/cp-deck-serve"
    install -m 0644 "$plant/cp-controller.service" "$MNT/etc/systemd/system/cp-controller.service"
    install -m 0644 "$plant/cp-sentinel.service" "$MNT/etc/systemd/system/cp-sentinel.service"
    install -m 0644 "$plant/cp-command-deck.service" "$MNT/etc/systemd/system/cp-command-deck.service"
    mkdir -p "$MNT/etc/systemd/system/multi-user.target.wants"
    ln -sfn /etc/systemd/system/cp-controller.service \
      "$MNT/etc/systemd/system/multi-user.target.wants/cp-controller.service"
    ln -sfn /etc/systemd/system/cp-sentinel.service \
      "$MNT/etc/systemd/system/multi-user.target.wants/cp-sentinel.service"
    ln -sfn /etc/systemd/system/cp-command-deck.service \
      "$MNT/etc/systemd/system/multi-user.target.wants/cp-command-deck.service"
  fi
  state="$MNT/var/lib/linkornew/command-post"
  mkdir -p "$state"
  printf '127.0.0.1\n' >"$state/nexus.host"
  printf 'true\n' >"$state/generation_only"
  token_env="${LINKORNEW_CP_TOKEN_ENV:-}"
  if [[ -z "$token_env" && -f "${LINKORNEW_COMMAND_POST}/podman/env/controller.env" ]]; then
    token_env="${LINKORNEW_COMMAND_POST}/podman/env/controller.env"
  fi
  if [[ -n "$token_env" && -f "$token_env" ]]; then
    python3 - "$token_env" "$state" <<'PY'
import sys
from pathlib import Path
src, state = Path(sys.argv[1]), Path(sys.argv[2])
vals = {}
for line in src.read_text().splitlines():
    line = line.strip()
    if not line or line.startswith("#") or "=" not in line:
        continue
    k, v = line.split("=", 1)
    vals[k] = v
need = ["NEXUS_REGISTRATION_TOKEN", "NEXUS_OPERATOR_TOKEN", "NEXUS_OPERATOR_DEFAULT_ROLE"]
missing = [k for k in need if not vals.get(k)]
if missing:
    raise SystemExit(f"qemu-linkornew: token env missing {missing}")
(state / "controller.env").write_text("\n".join(f"{k}={vals[k]}" for k in need) + "\n")
(state / "controller.env").chmod(0o600)
tok = vals["NEXUS_REGISTRATION_TOKEN"]
(state / "sentinel.env").write_text(
    "\n".join(
        [
            "NEXUS_SENTINEL_AGENT_ID=sentinel-linkornew",
            "NEXUS_SENTINEL_HOST_ID=linkornew-seat",
            "NEXUS_SENTINEL_CONTROLLER_URL=http://127.0.0.1:8088",
            f"NEXUS_REGISTRATION_TOKEN={tok}",
            f"NEXUS_SENTINEL_REGISTRATION_TOKEN={tok}",
            "NEXUS_SENTINEL_REQUIRE_CONTROLLER_AUTH=true",
            "NEXUS_SENTINEL_LISTEN_HOST=127.0.0.1",
            "NEXUS_SENTINEL_LISTEN_PORT=8090",
            "NEXUS_SENTINEL_GENERATION_ONLY=true",
            "NEXUS_SENTINEL_EXECUTION_ENABLED=false",
            "",
        ]
    )
)
(state / "sentinel.env").chmod(0o600)
print("qemu-linkornew: CP tokens planted (not printed)", file=sys.stderr)
PY
  else
    printf 'qemu-linkornew: WARNING no token env; controller will refuse start\n' >&2
  fi
fi

trap - EXIT
cleanup

profile=$(debugfs -R 'cat /etc/empiieros/seat-profile' "$DISK" 2>/dev/null | tr -d '\n' || true)
[[ "$profile" == "linkornew" ]] || { printf 'qemu-linkornew: overlay missing seat-profile\n' >&2; exit 1; }

printf 'qemu-linkornew: disk=%s profile=linkornew\n' "$DISK" >&2
[[ "$run" -eq 1 ]] || exit 0

display=( -display gtk,zoom-to-fit=on -device virtio-vga,xres=1280,yres=720 )
if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  display=( -display vnc=127.0.0.1:1 -device virtio-vga,xres=1280,yres=720 )
  printf 'qemu-linkornew: no GUI session; VNC 127.0.0.1:5901\n' >&2
fi

SERIAL="${LINKORNEW_QEMU_SERIAL:-$HOME/linkornew-lab/qemu-serial.log}"
: >"$SERIAL"
host_cpus=$(nproc)
smp=$((host_cpus / 2))
[[ "$smp" -ge 1 ]] || smp=1
printf 'qemu-linkornew: serial %s mem=12288 smp=%s (host nproc=%s)\n' "$SERIAL" "$smp" "$host_cpus" >&2

exec qemu-system-x86_64 \
  -machine accel=kvm:tcg \
  -cpu max \
  -m 12288 \
  -smp "$smp" \
  -kernel "$KERNEL" \
  -drive "file=${DISK},format=raw,if=virtio" \
  -append 'root=/dev/vda rw rootwait console=tty1 console=ttyS0,115200 systemd.log_target=console systemd.journald.forward_to_console=1' \
  "${display[@]}" \
  -serial "file:${SERIAL}" \
  -usb \
  -device usb-tablet \
  -device usb-kbd \
  -nic user,model=e1000e \
  -no-reboot
