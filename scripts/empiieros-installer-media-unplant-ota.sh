#!/usr/bin/env bash
# Unplant OTA key crutches from a mounted live-installer USB and plant
# overlay apply (no plant_seat_ota_pk). Soft-deletes ESP leftovers into
# EmpiierOS trash/. Live-root mutation needs root (stamp chown).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS="${EMPIERO_TS_ROOT:-${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}}"
STAMP="${TS}/scripts/stamp-live-installer-media.sh"
OVL_APPLY="${TS}/config/buildroot/external/board/eliteew-ts/rootfs_overlay/usr/local/bin/eliteew-ts-install-apply"

ESP="${EMPIERO_INSTALLER_ESP:-/run/media/owned/ELITEEWTS}"
LIVE="${EMPIERO_INSTALLER_ROOT:-/run/media/owned/eliteew-ts}"

usage() {
  cat <<'EOF'
Usage: scripts/empiieros-installer-media-unplant-ota.sh [--esp PATH] [--root PATH]

Trash ESP OTA crutches. As root, run stamp-live-installer-media.sh on the
live root (strip artifact.pk.hex, plant overlay apply). Never copies SK.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --esp) ESP="${2:-}"; shift 2 ;;
    --root) LIVE="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -f "$STAMP" && -f "$OVL_APPLY" ]] || {
  echo "FATAL: overlay apply or stamp missing under ${TS}" >&2
  exit 1
}
if grep -qE 'plant_seat_ota_pk|require_seat_ota_pk' "$OVL_APPLY"; then
  echo "FATAL: overlay apply still plants OTA PK" >&2
  exit 1
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
dest="${ROOT}/trash/${stamp}-usb-ota-install-crutch"
mkdir -p "$dest/esp" "$dest/live"

trash_if() {
  local src="$1" sub="$2"
  [[ -e "$src" || -L "$src" ]] || return 0
  mkdir -p "${dest}/${sub}"
  cp -a -- "$src" "${dest}/${sub}/$(basename "$src")"
  echo "copied: $src -> ${dest}/${sub}/$(basename "$src")"
}

if [[ -d "$ESP" ]]; then
  trash_if "${ESP}/artifact.pk.hex" esp
  trash_if "${ESP}/artifact.sk.hex" esp
  trash_if "${ESP}/empiieros-ota-plant-seat-pk.py" esp
  trash_if "${ESP}/eliteew-ts-install-apply" esp
  trash_if "${ESP}/_wtest" esp
  if [[ -w "$ESP" ]]; then
    rm -f "${ESP}/artifact.pk.hex" "${ESP}/artifact.sk.hex" \
      "${ESP}/empiieros-ota-plant-seat-pk.py" "${ESP}/eliteew-ts-install-apply" \
      "${ESP}/_wtest"
  fi
fi

if [[ -d "$LIVE" ]]; then
  trash_if "${LIVE}/usr/local/share/eliteew-ts-installer/artifact.pk.hex" live
  trash_if "${LIVE}/usr/local/share/eliteew-ts-installer/artifact.sk.hex" live
  trash_if "${LIVE}/usr/local/bin/eliteew-ts-install-apply" live
fi

if [[ "$(id -u)" -ne 0 ]]; then
  echo "need root to stamp live root apply. rerun:"
  echo "  sudo -n $0 --esp $(printf '%q' "$ESP") --root $(printf '%q' "$LIVE")"
  exit 2
fi

[[ -d "$LIVE" ]] || {
  echo "FATAL: live root not mounted: ${LIVE}" >&2
  exit 1
}

bash "$STAMP" "$LIVE"

if [[ -d "$ESP" ]]; then
  rm -f "${ESP}/artifact.pk.hex" "${ESP}/artifact.sk.hex" \
    "${ESP}/empiieros-ota-plant-seat-pk.py" "${ESP}/eliteew-ts-install-apply" \
    "${ESP}/_wtest"
fi

leftover="$(find "$ESP" "$LIVE" -xdev \( -name 'artifact.pk.hex' -o -name 'artifact.sk.hex' \) -print 2>/dev/null || true)"
if [[ -n "$leftover" ]]; then
  echo "FATAL: leftover OTA key still on media:" >&2
  printf '%s\n' "$leftover" >&2
  exit 1
fi
if grep -qE 'plant_seat_ota_pk|require_seat_ota_pk' \
  "${LIVE}/usr/local/bin/eliteew-ts-install-apply"; then
  echo "FATAL: live apply still plants OTA PK" >&2
  exit 1
fi
cmp -s "$OVL_APPLY" "${LIVE}/usr/local/bin/eliteew-ts-install-apply" || {
  echo "FATAL: live apply ≠ overlay apply" >&2
  exit 1
}

echo "unplant-ota: live apply matches overlay; no artifact.pk/sk on media"
echo "trash: ${dest}/"
