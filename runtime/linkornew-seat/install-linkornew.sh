#!/bin/sh
# Stage a LinkornEW root tree. Not the FoFline installer.
# No disk wipe, no air activation code, no capsule, no first-boot PIN.
set -eu

usage() {
  printf 'usage: %s stage <dest-dir>\n' "$0" >&2
  exit 2
}

here=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo=$(CDPATH= cd -- "$here/../.." && pwd)

refuse() {
  printf 'linkornew-install: %s\n' "$1" >&2
  exit 1
}

if [ "${1:-}" != "stage" ] || [ "$#" -ne 2 ]; then
  usage
fi

dest=$2
case "$dest" in
  /|/etc|/usr|/usr/local|/dev/*)
    refuse "refusing system path $dest"
    ;;
  "$repo"|"$repo"/*)
    refuse "refusing path inside EmpiierOS git"
    ;;
esac
if [ -b "$dest" ]; then
  refuse "refusing block device"
fi
if [ -f "$dest/seat/first_boot_pin.json" ] || [ -f "$dest/var/lib/empiieros-runtime/seat/first_boot_pin.json" ]; then
  refuse "refusing a tree that already has a FoFline PIN record"
fi

mkdir -p "$dest"
profile_dir=$dest/etc/empiieros
mkdir -p "$profile_dir"
printf 'linkornew\n' > "$profile_dir/seat-profile"

install -d "$dest/home/eliteew-ts"
install -m 0644 "$here/bash_profile" "$dest/home/eliteew-ts/.bash_profile"
install -d "$dest/usr/lib/linkornew" "$dest/etc/systemd/system"
install -m 0755 "$here/linkornew-session" "$dest/usr/lib/linkornew/linkornew-session"
install -m 0755 "$here/command-post/cp-guard.sh" "$dest/usr/lib/linkornew/cp-guard.sh"
mkdir -p "$dest/usr/lib/linkornew/config" "$dest/opt/linkornew/shell"
cp -a "$here/config/labwc" "$dest/usr/lib/linkornew/config/"
cp -a "$here/desk/." "$dest/opt/linkornew/shell/"
if [ -x "$here/console" ]; then
  install -m 0755 "$here/console" "$dest/usr/lib/linkornew/console"
fi
if [ -x "$here/linkornew-seat" ]; then
  install -m 0755 "$here/linkornew-seat" "$dest/usr/lib/linkornew/linkornew-seat"
fi
if [ -x "$here/linkornew-host" ]; then
  install -m 0755 "$here/linkornew-host" "$dest/usr/lib/linkornew/linkornew-host"
fi
if [ -x "$here/command-post/cp-deck-serve" ]; then
  install -m 0755 "$here/command-post/cp-deck-serve" "$dest/usr/lib/linkornew/cp-deck-serve"
fi
if [ -x "$here/command-post/cp-controller-run.sh" ]; then
  install -m 0755 "$here/command-post/cp-controller-run.sh" "$dest/usr/lib/linkornew/cp-controller-run.sh"
fi
if [ -x "$here/command-post/cp-sentinel-run.sh" ]; then
  install -m 0755 "$here/command-post/cp-sentinel-run.sh" "$dest/usr/lib/linkornew/cp-sentinel-run.sh"
fi
# Optional: pre-planted Deck static tree
if [ -d "$here/command-post/command-deck-dist" ] && [ -f "$here/command-post/command-deck-dist/index.html" ]; then
  install -d "$dest/usr/lib/linkornew/command-deck"
  cp -a "$here/command-post/command-deck-dist/." "$dest/usr/lib/linkornew/command-deck/"
fi
# Optional: pre-planted CP tree (cpython + controller + sentinel) from plant-*.sh
if [ -d "$here/command-post/command-post-plant/controller" ]; then
  install -d "$dest/usr/lib/linkornew/command-post"
  cp -a "$here/command-post/command-post-plant/." "$dest/usr/lib/linkornew/command-post/"
fi
if [ -d "$here/command-post/cpython-plant/bin" ]; then
  install -d "$dest/usr/lib/linkornew/cpython"
  cp -a "$here/command-post/cpython-plant/." "$dest/usr/lib/linkornew/cpython/"
fi
install -m 0644 "$here/command-post/cp-controller.service" "$dest/etc/systemd/system/cp-controller.service"
install -m 0644 "$here/command-post/cp-sentinel.service" "$dest/etc/systemd/system/cp-sentinel.service"
install -m 0644 "$here/command-post/cp-command-deck.service" "$dest/etc/systemd/system/cp-command-deck.service"
install -m 0644 "$here/systemd/linkornew-session.service" "$dest/etc/systemd/system/linkornew-session.service"
install -d "$dest/etc/systemd/resolved.conf.d" "$dest/etc/systemd/network"
install -m 0644 "$here/systemd/resolved-linkornew.conf" \
  "$dest/etc/systemd/resolved.conf.d/linkornew.conf"
install -m 0644 "$here/systemd/10-linkornew-wired.network" \
  "$dest/etc/systemd/network/10-linkornew-wired.network"
install -d "$dest/etc/systemd/system/multi-user.target.wants"
ln -sfn /etc/systemd/system/linkornew-session.service \
  "$dest/etc/systemd/system/multi-user.target.wants/linkornew-session.service"
if [ -x "$dest/usr/lib/linkornew/cp-deck-serve" ]; then
  ln -sfn /etc/systemd/system/cp-command-deck.service \
    "$dest/etc/systemd/system/multi-user.target.wants/cp-command-deck.service"
fi
if [ -x "$dest/usr/lib/linkornew/cp-controller-run.sh" ] \
  && [ -x "$dest/usr/lib/linkornew/command-post/controller/.venv/bin/python" ] \
  && [ -x "$dest/usr/lib/linkornew/cpython/bin/python-linkornew" ]; then
  ln -sfn /etc/systemd/system/cp-controller.service \
    "$dest/etc/systemd/system/multi-user.target.wants/cp-controller.service"
fi
if [ -x "$dest/usr/lib/linkornew/cp-sentinel-run.sh" ] \
  && [ -x "$dest/usr/lib/linkornew/command-post/sentinel/.venv/bin/python" ] \
  && [ -x "$dest/usr/lib/linkornew/cpython/bin/python-linkornew" ]; then
  ln -sfn /etc/systemd/system/cp-sentinel.service \
    "$dest/etc/systemd/system/multi-user.target.wants/cp-sentinel.service"
fi
install -d "$dest/var/lib/linkornew/command-post"
if [ ! -f "$dest/var/lib/linkornew/command-post/nexus.host" ]; then
  printf '127.0.0.1\n' > "$dest/var/lib/linkornew/command-post/nexus.host"
fi
if [ ! -f "$dest/var/lib/linkornew/command-post/generation_only" ]; then
  printf 'true\n' > "$dest/var/lib/linkornew/command-post/generation_only"
fi
for n in 1 2 3 4 5 6; do
  target="$dest/etc/systemd/system/getty@tty${n}.service"
  if ! ln -sfn /dev/null "$target" 2>/dev/null; then
    rm -f "$target"
    printf '/dev/null\n' > "${target}.mask"
  fi
done

printf 'linkornew-install: staged %s\n' "$dest"
