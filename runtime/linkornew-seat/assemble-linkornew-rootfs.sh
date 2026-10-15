#!/bin/sh
# Overlay LinkornEW onto a factory rootfs tree.
# Does not modify official out/.
# Factory tar must be extracted as root (--numeric-owner) so setuid and
# uid 0 survive. A user extract makes EmpiierOS privileged runtime exit
# on state_store_open_failed (directory owner != euid 0) and drops login setuid.
set -eu

usage() {
  printf 'usage: %s <extracted-rootfs>\n' "$0" >&2
  printf '       %s --overlay <extracted-rootfs>\n' "$0" >&2
  printf '       %s --from-tar <rootfs.tar.gz> <dest-dir>\n' "$0" >&2
  exit 2
}

here=$(CDPATH= cd -- "$(dirname "$0")" && pwd)

refuse() {
  printf 'linkornew-rootfs: %s\n' "$1" >&2
  exit 1
}

require_factory_identity() {
  root=$1
  [ -x "$root/usr/local/bin/empiieros-runtime" ] || refuse "missing empiieros-runtime in $root"
  # Factory /usr/bin/login is 0755 (agetty is already root). User tar extract
  # drops setuid on dbus-daemon-launch-helper and makes state uid 1000.
  [ -u "$root/usr/libexec/dbus-daemon-launch-helper" ] || refuse "dbus helper is not setuid; extract factory tar as root --numeric-owner"
  owner=$(stat -c '%u' "$root/var/lib/empiieros-runtime")
  [ "$owner" = "0" ] || refuse "state dir uid=$owner (runtime requires euid 0)"
}

overlay() {
  root=$1
  stage=$(mktemp -d)
  "$here/install-linkornew.sh" stage "$stage"
  if [ -d "$root/home/eliteew-ts" ]; then
    chmod 0755 "$root/home/eliteew-ts" || true
  fi
  # mktemp dirs are 0700. `tar -cf - .` would chmod the image `/` to 0700 and
  # every User= unit would die CHDIR (networkd, this seat).
  (
    cd "$stage" || exit 1
    tar -cf - -- *
  ) | tar -C "$root" -xf -
  chmod 0755 "$root"
  rm -rf "$stage"

  if [ -z "${LINKORNEW_STATION_CHROMIUM:-}" ]; then
    if [ -n "${SUDO_USER:-}" ]; then
      sudo_home=$(getent passwd "$SUDO_USER" | cut -d: -f6)
      if [ -n "$sudo_home" ] && [ -f "$sudo_home/Files/fi/station-chromium/build/src/out/Default/chrome" ]; then
        LINKORNEW_STATION_CHROMIUM=$sudo_home/Files/fi/station-chromium
        export LINKORNEW_STATION_CHROMIUM
      fi
    fi
  fi
  "$here/plant-chromium.sh" lab-overlay "$root"

  # Optional Command Post L5 plant (deck+cpython+controller+sentinel).
  # Set LINKORNEW_COMMAND_POST=/path/to/command-post (build host tree).
  if [ -n "${LINKORNEW_COMMAND_POST:-}" ]; then
    if [ ! -f "$LINKORNEW_COMMAND_POST/controller/app/main.py" ]; then
      refuse "LINKORNEW_COMMAND_POST is not a Command Post tree: $LINKORNEW_COMMAND_POST"
    fi
    plant_cp="$here/command-post"
    "$plant_cp/plant-cpython.sh" "$root"
    if [ -f "$LINKORNEW_COMMAND_POST/command-deck/dist/index.html" ]; then
      "$plant_cp/plant-deck.sh" "$LINKORNEW_COMMAND_POST/command-deck/dist" "$root"
    else
      refuse "command-deck/dist missing under $LINKORNEW_COMMAND_POST (build Deck first)"
    fi
    "$plant_cp/plant-controller.sh" "$LINKORNEW_COMMAND_POST" "$root"
    "$plant_cp/plant-sentinel.sh" "$LINKORNEW_COMMAND_POST" "$root"
    install -m 0755 "$plant_cp/cp-controller-run.sh" "$root/usr/lib/linkornew/cp-controller-run.sh"
    install -m 0755 "$plant_cp/cp-sentinel-run.sh" "$root/usr/lib/linkornew/cp-sentinel-run.sh"
    if [ -x "$here/command-post/cp-deck-serve" ]; then
      install -m 0755 "$here/command-post/cp-deck-serve" "$root/usr/lib/linkornew/cp-deck-serve"
    fi
    install -m 0644 "$plant_cp/cp-controller.service" "$root/etc/systemd/system/cp-controller.service"
    install -m 0644 "$plant_cp/cp-sentinel.service" "$root/etc/systemd/system/cp-sentinel.service"
    install -m 0644 "$plant_cp/cp-command-deck.service" "$root/etc/systemd/system/cp-command-deck.service"
    mkdir -p "$root/etc/systemd/system/multi-user.target.wants"
    ln -sfn /etc/systemd/system/cp-controller.service \
      "$root/etc/systemd/system/multi-user.target.wants/cp-controller.service"
    ln -sfn /etc/systemd/system/cp-sentinel.service \
      "$root/etc/systemd/system/multi-user.target.wants/cp-sentinel.service"
    ln -sfn /etc/systemd/system/cp-command-deck.service \
      "$root/etc/systemd/system/multi-user.target.wants/cp-command-deck.service"
    printf 'linkornew-rootfs: planted Command Post from %s\n' "$LINKORNEW_COMMAND_POST"
    printf 'linkornew-rootfs: write tokens under %s/var/lib/linkornew/command-post/\n' "$root"
  fi

  guest_libs=${LINKORNEW_GUEST_LIBS:-}
  if [ -z "$guest_libs" ] && [ -n "${SUDO_USER:-}" ]; then
    sudo_home=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    if [ -f "$sudo_home/linkornew-lab/guest-libs-prefix/usr/lib/libnss3.so" ]; then
      guest_libs=$sudo_home/linkornew-lab/guest-libs-prefix
    fi
  fi
  if [ -z "$guest_libs" ] && [ -f "${HOME}/linkornew-lab/guest-libs-prefix/usr/lib/libnss3.so" ]; then
    guest_libs=${HOME}/linkornew-lab/guest-libs-prefix
  fi
  if [ -n "$guest_libs" ]; then
    mkdir -p "$root/usr/lib"
    for so in libnspr4.so libplc4.so libplds4.so libnss3.so libnssutil3.so libsmime3.so libssl3.so libnssckbi.so libfreebl3.so libfreeblpriv3.so libsoftokn3.so libsqlite3.so libcups.so.2; do
      if [ -f "$guest_libs/usr/lib/$so" ] && [ ! -L "$guest_libs/usr/lib/$so" ]; then
        install -m 0755 "$guest_libs/usr/lib/$so" "$root/usr/lib/$so"
      fi
    done
  fi

  if [ ! -s "$root/etc/machine-id" ]; then
    tr -d '-' </proc/sys/kernel/random/uuid | tr -d '\n' >"$root/etc/machine-id"
    printf '\n' >>"$root/etc/machine-id"
  fi

  router=$root/usr/local/bin/eliteew-ts-session-router
  if [ -f "$router" ] && ! grep -q 'seat-profile' "$router"; then
    tmp=$(mktemp)
    awk '
      $0 == "if [[ -e \"$DISABLE_KIOSK\" ]]; then" && !done {
        print "if [[ -f /etc/empiieros/seat-profile ]] && grep -qx linkornew /etc/empiieros/seat-profile; then"
        print "  if [[ -x /usr/lib/linkornew/linkornew-session ]]; then"
        print "    exec /usr/lib/linkornew/linkornew-session"
        print "  fi"
        print "  tty_msg \"missing linkornew session — holding tty1\""
        print "  sleep infinity"
        print "fi"
        print ""
        done = 1
      }
      { print }
    ' "$router" > "$tmp"
    rm -f "$router"
    cp "$tmp" "$router"
    rm -f "$tmp"
    chmod 0755 "$router"
  fi
}

if [ "${1:-}" = "--overlay" ]; then
  [ "$#" -eq 2 ] || usage
  dest=$2
  [ -d "$dest/usr/local/bin" ] || refuse "not a rootfs: $dest"
  overlay "$dest"
  printf 'linkornew-rootfs: profile=%s src=overlay\n' "$(tr -d '\n' < "$dest/etc/empiieros/seat-profile")"
  exit 0
fi

if [ "${1:-}" = "--from-tar" ]; then
  [ "$#" -eq 3 ] || usage
  tar=$2
  dest=$3
  [ "$(id -u)" -eq 0 ] || refuse "run --from-tar as root"
  [ -f "$tar" ] || refuse "missing tar $tar"
  [ ! -e "$dest" ] || refuse "dest exists: $dest"
  mkdir -p "$dest"
  tar --numeric-owner -C "$dest" -xzf "$tar"
  require_factory_identity "$dest"
  overlay "$dest"
  require_factory_identity "$dest"
  printf 'linkornew-rootfs: profile=%s src=tar\n' "$(tr -d '\n' < "$dest/etc/empiieros/seat-profile")"
  exit 0
fi

[ "$#" -eq 1 ] || usage
root=$1
[ -d "$root/usr/local/bin" ] || usage
[ "$(id -u)" -eq 0 ] || refuse "run as root so overlay files stay uid 0"
require_factory_identity "$root"
overlay "$root"
require_factory_identity "$root"
printf 'linkornew-rootfs: profile=%s\n' "$(tr -d '\n' < "$root/etc/empiieros/seat-profile")"
