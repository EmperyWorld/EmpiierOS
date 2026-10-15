#!/usr/bin/env bash
# Build in-tree WireGuard (kernel module) and wireguard-tools `wg` from the
# Trusted Substrate factory source trees. No distro packages. No wg-quick.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SUBSTRATE="${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}"
# shellcheck disable=SC1091
source "$SUBSTRATE/scripts/work-root.sh"
OUT="${ELITEEW_TS_BUILDROOT_OUT:?}"
KDIR="$OUT/build/linux-6.12.20"
TOOLS="$OUT/build/wireguard-tools-1.0.20210914"
HOST_BIN="$OUT/host/bin"
CC="$HOST_BIN/x86_64-buildroot-linux-gnu-gcc"
STRIP="$HOST_BIN/x86_64-buildroot-linux-gnu-strip"
STAGE="$ROOT/runtime/target/metal/wireguard"
FRAGMENT="$SUBSTRATE/config/buildroot/external/board/eliteew-ts/linux-wireguard.fragment"

[[ -x "$CC" ]] || { echo "missing cross gcc: $CC" >&2; exit 1; }
[[ -d "$KDIR" ]] || { echo "missing kernel tree: $KDIR" >&2; exit 1; }
[[ -d "$TOOLS/src" ]] || { echo "missing wireguard-tools source: $TOOLS" >&2; exit 1; }
[[ -f "$FRAGMENT" ]] || { echo "missing WG fragment: $FRAGMENT" >&2; exit 1; }
rg -q '^CONFIG_WIREGUARD=m$' "$FRAGMENT" || { echo "fragment must set CONFIG_WIREGUARD=m" >&2; exit 1; }

install -d "$STAGE"
export PATH="$HOST_BIN:$PATH"
export ARCH=x86_64
export CROSS_COMPILE=x86_64-buildroot-linux-gnu-

printf '[wg-src] userspace wg from %s\n' "$TOOLS"
make -C "$TOOLS/src" -j"$(nproc)" \
  CC="$CC" \
  WITH_WGQUICK=no \
  WITH_SYSTEMDUNITS=no
install -m 0755 "$TOOLS/src/wg" "$STAGE/empiieros-wg"
"$STRIP" "$STAGE/empiieros-wg"
"$HOST_BIN/x86_64-buildroot-linux-gnu-objdump" -f "$STAGE/empiieros-wg" \
  | rg -q 'file format elf64-x86-64'
if rg -a -q 'wg-quick' "$STAGE/empiieros-wg"; then
  echo "userspace wg unexpectedly embeds wg-quick" >&2
  exit 1
fi

printf '[wg-src] merge kernel fragment\n'
if [[ ! -f "$KDIR/.config.empiieros-pre-wg" ]]; then
  cp -a "$KDIR/.config" "$KDIR/.config.empiieros-pre-wg"
fi
"$KDIR/scripts/kconfig/merge_config.sh" -m -O "$KDIR" "$KDIR/.config" "$FRAGMENT"
make -C "$KDIR" olddefconfig
rg -q '^CONFIG_WIREGUARD=m$' "$KDIR/.config" || {
  echo "olddefconfig dropped CONFIG_WIREGUARD=m" >&2
  exit 1
}

printf '[wg-src] kernel datapath modules from %s\n' "$KDIR"
make -C "$KDIR" -j"$(nproc)" modules_prepare
make -C "$KDIR" -j"$(nproc)" modules
install -d "$STAGE/modules"
for ko in \
  "$KDIR/lib/crypto/libchacha.ko" \
  "$KDIR/lib/crypto/libpoly1305.ko" \
  "$KDIR/lib/crypto/libchacha20poly1305.ko" \
  "$KDIR/lib/crypto/libcurve25519-generic.ko" \
  "$KDIR/arch/x86/crypto/chacha-x86_64.ko" \
  "$KDIR/arch/x86/crypto/poly1305-x86_64.ko" \
  "$KDIR/arch/x86/crypto/curve25519-x86_64.ko" \
  "$KDIR/net/ipv4/udp_tunnel.ko" \
  "$KDIR/net/ipv6/ip6_udp_tunnel.ko" \
  "$KDIR/drivers/net/wireguard/wireguard.ko"
do
  [[ -f "$ko" ]] || { echo "missing $ko" >&2; exit 1; }
  install -m 0644 "$ko" "$STAGE/modules/$(basename "$ko")"
done
if ! rg -a -q 'vermagic' "$STAGE/modules/wireguard.ko"; then
  echo "wireguard.ko missing vermagic" >&2
  exit 1
fi

sha256sum "$STAGE/empiieros-wg" "$STAGE/modules"/*.ko
printf '[wg-src] ok %s\n' "$STAGE"
