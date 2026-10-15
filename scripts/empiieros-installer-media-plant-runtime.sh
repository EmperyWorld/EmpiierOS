#!/usr/bin/env bash
# Replace SoT tar Runtime on a mounted live-installer USB, plus the
# live apply sources (share + live bin). Mux only — not official out/.
# Never plants artifact PK/SK.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${EMPIERO_RUNTIME_BIN:-$ROOT/runtime/target/release/empiieros-runtime}"
LIVE="${EMPIERO_INSTALLER_ROOT:-/run/media/owned/eliteew-ts}"
TAR="${LIVE}/opt/eliteew-ts-install/rootfs.tar.gz"
SHA_FILE="${TAR}.sha256"
MEMBER="./usr/local/bin/empiieros-runtime"

usage() {
  cat <<'EOF'
Usage: scripts/empiieros-installer-media-plant-runtime.sh [--root PATH] [--bin PATH]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) LIVE="${2:-}"; TAR="${LIVE}/opt/eliteew-ts-install/rootfs.tar.gz"; SHA_FILE="${TAR}.sha256"; shift 2 ;;
    --bin) BIN="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -x "$BIN" && ! -L "$BIN" ]] || {
  echo "FATAL: release Runtime missing: ${BIN}" >&2
  exit 1
}
[[ -f "$TAR" && ! -L "$TAR" ]] || {
  echo "FATAL: SoT tar missing: ${TAR}" >&2
  exit 1
}
[[ -e "${LIVE}/etc/eliteew-trusted-substrate/live-installer" ]] || {
  echo "FATAL: not live-installer media: ${LIVE}" >&2
  exit 1
}
if find "$LIVE" -xdev \( -name 'artifact.pk.hex' -o -name 'artifact.sk.hex' \) -print 2>/dev/null | grep -q .; then
  echo "FATAL: leftover OTA key on media; unplant first" >&2
  exit 1
fi
if ! rg -a -q 'plant_from_persisted_inner' "$BIN"; then
  echo "FATAL: binary lacks plant_from_persisted_inner" >&2
  exit 1
fi
if ! rg -a -q -- '--on-active=3s' "$BIN"; then
  echo "FATAL: binary lacks deferred self-restart" >&2
  exit 1
fi
for needle in \
  'oidc dpop iat missing Date' \
  operator_wall_clock \
  portal_hostmap_required \
  wall_clock_required
do
  rg -a -q -- "$needle" "$BIN" || {
    echo "FATAL: binary lacks IdP Date marker: ${needle}" >&2
    exit 1
  }
done
if rg -a -q 'date_header_missing' "$BIN"; then
  echo "FATAL: binary still has date_header_missing" >&2
  exit 1
fi

install_live() {
  local dest="$1"
  local dir
  dir="$(dirname "$dest")"
  if [[ -w "$dir" && ( ! -e "$dest" || -w "$dest" ) ]]; then
    install -m 0755 "$BIN" "$dest"
  else
    pkexec --disable-internal-agent /bin/sh -c \
      "install -d -m 0755 '$dir' && install -m 0755 '$BIN' '$dest' && chown 0:0 '$dest'"
  fi
  [[ "$(sha256sum "$dest" | awk '{print $1}')" == "$(sha256sum "$BIN" | awk '{print $1}')" ]] \
    || { echo "FATAL: live plant mismatch: ${dest}" >&2; exit 1; }
}

WORK="$(mktemp -d /tmp/empiieros-sot-runtime.XXXXXX)"
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT

mkdir -p "${WORK}/tree/usr/local/bin"
install -m 0755 "$BIN" "${WORK}/tree/usr/local/bin/empiieros-runtime"

echo "decompress ${TAR}"
gzip -dc "$TAR" >"${WORK}/rootfs.tar"
python3 - "$WORK/rootfs.tar" "$MEMBER" <<'PY'
import sys, tarfile
archive, member = sys.argv[1], sys.argv[2]
with tarfile.open(archive, "r:") as t:
    names = t.getnames()
if member not in names:
    raise SystemExit(f"FATAL: tar member missing: {member}")
PY
tar --delete -f "${WORK}/rootfs.tar" "$MEMBER"
(
  cd "${WORK}/tree"
  tar -uf "${WORK}/rootfs.tar" --owner=0 --group=0 --numeric-owner "$MEMBER"
)

echo "compress"
gzip -n -c "${WORK}/rootfs.tar" >"${WORK}/rootfs.tar.gz"
HASH="$(sha256sum "${WORK}/rootfs.tar.gz" | awk '{print $1}')"
python3 - "$WORK/rootfs.tar.gz" "$MEMBER" "$WORK/member.bin" <<'PY'
import sys, tarfile, subprocess
archive, member, out_bin = sys.argv[1], sys.argv[2], sys.argv[3]
with tarfile.open(archive, "r:gz") as t:
    names = [n for n in t.getnames() if n.endswith("empiieros-runtime") and "var/lib" not in n]
    if names != [member]:
        raise SystemExit(f"runtime members={names!r}")
    data = t.extractfile(t.getmember(member)).read()
open(out_bin, "wb").write(data)
for needle in ("plant_from_persisted_inner", "plant_from_license", "--on-active=3s"):
    probe = subprocess.check_output(["rg", "-a", "-c", "--", needle, out_bin], text=True)
    if int(probe.strip() or "0") < 1:
        raise SystemExit(f"planted member lacks {needle}")
print("member_ok", member, "bytes", len(data))
PY

if [[ -w "$TAR" ]]; then
  cat "${WORK}/rootfs.tar.gz" >"$TAR"
else
  pkexec --disable-internal-agent /bin/sh -c "cat '${WORK}/rootfs.tar.gz' >'${TAR}'"
fi
sync
printf '%s\n' "$HASH" >"${WORK}/sha256"
if [[ -w "$SHA_FILE" ]]; then
  cp -f "${WORK}/sha256" "$SHA_FILE"
else
  pkexec --disable-internal-agent /bin/sh -c "cp -f '${WORK}/sha256' '${SHA_FILE}' && chown root:root '${SHA_FILE}' && chmod 0644 '${SHA_FILE}'"
fi
[[ "$(tr -d ' \n' <"$SHA_FILE")" == "$HASH" ]] || {
  echo "FATAL: sha256 witness mismatch" >&2
  exit 1
}
SHARE="${LIVE}/usr/local/share/eliteew-ts-installer/empiieros-runtime"
LIVE_BIN="${LIVE}/usr/local/bin/empiieros-runtime"
echo "plant live apply sources"
install_live "$SHARE"
install_live "$LIVE_BIN"

echo "plant-runtime: SoT sha256=${HASH}"
echo "plant-runtime: binary $(sha256sum "$BIN" | awk '{print $1}')"
echo "plant-runtime: share $(sha256sum "$SHARE" | awk '{print $1}')"
echo "plant-runtime: live $(sha256sum "$LIVE_BIN" | awk '{print $1}')"
