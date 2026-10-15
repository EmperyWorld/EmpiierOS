#!/usr/bin/env bash
# Patch live-installer USB: replace eliteew-ts-installer GUI + eliteew-ts-install-apply
# + build manifests. Does NOT reflash the stick and does NOT rewrite rootfs.tar.gz.
set -euo pipefail

EMPIIEROS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS_ROOT="${ELITEEW_TS_ROOT:-${EMPIIEROS_ROOT}/../powered_by_emperyworld/pack-organization/trusted-substrate}"
LIVE="${EMPIERO_INSTALLER_ROOT:-/run/media/owned/eliteew-ts}"
BUILD="${ELITEEW_TS_BUILD_INSTALLER:-1}"

usage() {
  cat <<'EOF'
Usage: scripts/empiieros-installer-media-plant-installer-gui.sh [--root PATH] [--no-build]

Build (zig GLIBC ceiling) via trusted-substrate build-installer-from-source.sh,
then install to mounted live root:
  usr/local/bin/eliteew-ts-installer
  usr/local/bin/eliteew-ts-install-apply
  opt/eliteew-ts-installer/build-manifest.json

Requires: live-installer stamp, pkexec/polkit for root-owned ext4 mount.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) LIVE="${2:-}"; shift 2 ;;
    --no-build) BUILD=0; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -e "${LIVE}/etc/eliteew-trusted-substrate/live-installer" ]] || {
  echo "FATAL: not live-installer media: ${LIVE}" >&2
  exit 1
}

OVL="${TS_ROOT}/config/buildroot/external/board/eliteew-ts/rootfs_overlay"
BIN="${OVL}/usr/local/bin/eliteew-ts-installer"
APPLY="${OVL}/usr/local/bin/eliteew-ts-install-apply"
MAN="${OVL}/opt/eliteew-ts-installer/build-manifest.json"

if [[ "$BUILD" -eq 1 ]]; then
  bash "${TS_ROOT}/scripts/build-installer-from-source.sh"
fi

[[ -x "$BIN" && -f "$MAN" && -f "$APPLY" ]] || {
  echo "FATAL: overlay installer/apply outputs missing (build first)" >&2
  exit 1
}

before="$(sha256sum "${LIVE}/usr/local/bin/eliteew-ts-installer" 2>/dev/null | awk '{print $1}' || true)"
before_apply="$(sha256sum "${LIVE}/usr/local/bin/eliteew-ts-install-apply" 2>/dev/null | awk '{print $1}' || true)"
new="$(sha256sum "$BIN" | awk '{print $1}')"
new_apply="$(sha256sum "$APPLY" | awk '{print $1}')"
if [[ "$before" == "$new" && "$before_apply" == "$new_apply" ]]; then
  echo "OK: USB installer+apply already match overlay"
  echo "installer_sha256=${new}"
  echo "apply_sha256=${new_apply}"
  exit 0
fi

pkexec /bin/sh -c "
  install -m 0755 '$BIN' '$LIVE/usr/local/bin/eliteew-ts-installer' &&
  install -m 0755 '$APPLY' '$LIVE/usr/local/bin/eliteew-ts-install-apply' &&
  install -d -m 0755 '$LIVE/opt/eliteew-ts-installer' &&
  install -m 0644 '$MAN' '$LIVE/opt/eliteew-ts-installer/build-manifest.json' &&
  sync
"
after="$(sha256sum "${LIVE}/usr/local/bin/eliteew-ts-installer" | awk '{print $1}')"
after_apply="$(sha256sum "${LIVE}/usr/local/bin/eliteew-ts-install-apply" | awk '{print $1}')"
[[ "$after" == "$new" ]] || {
  echo "FATAL: plant mismatch after=${after} expected=${new}" >&2
  exit 1
}
[[ "$after_apply" == "$new_apply" ]] || {
  echo "FATAL: apply plant mismatch after=${after_apply} expected=${new_apply}" >&2
  exit 1
}

echo "PLANT_OK live=${LIVE}"
echo "installer_sha256_before=${before:-none}"
echo "installer_sha256_after=${after}"
echo "apply_sha256_before=${before_apply:-none}"
echo "apply_sha256_after=${after_apply}"
