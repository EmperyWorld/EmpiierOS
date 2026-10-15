#!/bin/sh
# Identify a station-chromium *source* build and copy its runtime files
# onto a staff rootfs. Does not copy Fedora/AppImage .so or host libc.
# The chrome ELF is not stored in this git.
set -eu

PIN_SHA256=98307979de788ecf6d728038865b3947e2bd89bcacea78128dd17a2d394736c8
PIN_VERSION='EliteEW 154.0.8037.57'

usage() {
  printf 'usage: %s check <chrome>\n' "$0" >&2
  printf '       %s install <chrome> <source-root> <dest-dir>\n' "$0" >&2
  printf '       %s lab-overlay <rootfs>\n' "$0" >&2
  exit 2
}

repo_root() {
  CDPATH= cd -- "$(dirname "$0")/../../" && pwd
}

refuse_inside_repo() {
  root=$(repo_root)
  case "$1" in
    "$root"|"$root"/*)
      printf 'linkornew-chromium: refusing path inside EmpiierOS git: %s\n' "$1" >&2
      exit 1
      ;;
  esac
}

refuse_official_out() {
  root=$(repo_root)
  ts="${ELITEEW_TS_ROOT:-$root/../powered_by_emperyworld/pack-organization/trusted-substrate}"
  case "$1" in
    "$ts/out"|"$ts/out"/*)
      printf 'linkornew-chromium: refusing official out/\n' "$1" >&2
      exit 1
      ;;
  esac
}

check_chrome() {
  chrome=$1
  refuse_inside_repo "$chrome"
  if [ ! -f "$chrome" ] || [ -L "$chrome" ]; then
    target=$(readlink -f "$chrome" 2>/dev/null || true)
    if [ -z "$target" ] || [ ! -f "$target" ]; then
      printf 'linkornew-chromium: not a regular chrome binary: %s\n' "$chrome" >&2
      exit 1
    fi
    chrome=$target
  fi
  refuse_inside_repo "$chrome"
  hash=$(sha256sum "$chrome" | awk '{print $1}')
  version=$("$chrome" --version | head -n 1 | tr -d '\r')
  version=$(printf '%s' "$version" | sed 's/[[:space:]]*$//')
  if [ "$hash" != "$PIN_SHA256" ] || [ "$version" != "$PIN_VERSION" ]; then
    printf 'linkornew-chromium: hash or version mismatch\n' >&2
    printf '  have %s %s\n' "$hash" "$version" >&2
    printf '  pin  %s %s\n' "$PIN_SHA256" "$PIN_VERSION" >&2
    exit 1
  fi
  printf 'linkornew-chromium: %s %s\n' "$version" "$hash"
}

credits_html() {
  station=$1
  for candidate in \
    "$station/licenses/about_credits.html" \
    "$station/build/src/out/Default/gen/components/resources/about_credits.html"
  do
    if [ -f "$candidate" ]; then
      bytes=$(wc -c < "$candidate")
      if [ "$bytes" -gt 100000 ]; then
        printf '%s\n' "$candidate"
        return 0
      fi
    fi
  done
  return 1
}

license_dump_open() {
  station=$1
  if credits_html "$station" >/dev/null; then
    return 1
  fi
  shipped=$station/licenses/SHIPPED.txt
  if [ ! -f "$shipped" ]; then
    return 0
  fi
  grep -q 'Manual follow-up required' "$shipped"
}

write_wrapper() {
  dest=$1
  cat >"$dest/chrome" <<'EOF'
#!/bin/sh
HERE=/usr/lib/station-chromium
cd "$HERE" || exit 1
unset LD_LIBRARY_PATH
unset LD_PRELOAD
export PATH="$HERE:/usr/bin:/bin"
export GDK_BACKEND=wayland
exec "$HERE/chrome.bin" --ozone-platform=wayland --disable-gpu --no-sandbox --disable-dev-shm-usage --dns-over-https-mode=off "$@"
EOF
  chmod 0755 "$dest/chrome"
}

copy_build_runtime() {
  src=$1
  dest=$2
  mkdir -p "$dest/locales"
  install -m 0755 "$src/chrome" "$dest/chrome.bin"
  for name in \
    chrome_100_percent.pak \
    chrome_200_percent.pak \
    resources.pak \
    icudtl.dat \
    snapshot_blob.bin \
    v8_context_snapshot.bin \
    vk_swiftshader_icd.json \
    libEGL.so \
    libGLESv2.so \
    libvk_swiftshader.so \
    libvulkan.so.1 \
    chrome_crashpad_handler
  do
    if [ -e "$src/$name" ]; then
      cp -a "$src/$name" "$dest/"
    fi
  done
  if [ -d "$src/locales" ]; then
    cp -a "$src/locales/." "$dest/locales/"
  fi
  write_wrapper "$dest"
  printf '%s\n' "$PIN_SHA256" >"$dest/chrome.sha256"
}

lab_overlay() {
  rootfs=$1
  refuse_inside_repo "$rootfs"
  refuse_official_out "$rootfs"
  dest=$rootfs/usr/lib/station-chromium
  refuse_inside_repo "$dest"
  refuse_official_out "$dest"
  station=${LINKORNEW_STATION_CHROMIUM:-$HOME/Files/fi/station-chromium}
  src=$station/build/src/out/Default
  chrome=$src/chrome
  if [ ! -f "$chrome" ]; then
    printf 'linkornew-chromium: lab overlay skipped (no source chrome at %s)\n' "$chrome" >&2
    return 0
  fi
  check_chrome "$chrome"
  if license_dump_open "$station"; then
    printf 'linkornew-chromium: refusing overlay: chromium license dump is incomplete\n' >&2
    return 1
  fi
  mkdir -p "$dest" "$rootfs/var/lib/empiieros-runtime/linkorn/chrome"
  copy_build_runtime "$src" "$dest"
  credits=$(credits_html "$station")
  install -m 0644 "$credits" "$dest/about_credits.html"
  chmod 0777 "$rootfs/var/lib/empiieros-runtime/linkorn/chrome" || true
  printf 'linkornew-chromium: source runtime %s\n' "$dest" >&2
}

if [ "$#" -lt 2 ]; then
  usage
fi

case "$1" in
  check)
    [ "$#" -eq 2 ] || usage
    check_chrome "$2"
    ;;
  install)
    [ "$#" -eq 4 ] || usage
    refuse_inside_repo "$4"
    refuse_official_out "$4"
    check_chrome "$2"
    if license_dump_open "$3"; then
      printf 'linkornew-chromium: refusing install: chromium license dump is incomplete\n' >&2
      exit 1
    fi
    dest=$4
    refuse_inside_repo "$dest"
    mkdir -p "$dest"
    chrome=$2
    if [ -L "$chrome" ]; then
      chrome=$(readlink -f "$chrome")
    fi
    install -m 0755 "$chrome" "$dest/chrome"
    credits=$(credits_html "$3")
    install -m 0644 "$credits" "$dest/about_credits.html"
    printf '%s\n' "$PIN_SHA256" > "$dest/chrome.sha256"
    printf 'linkornew-chromium: installed %s\n' "$dest/chrome"
    ;;
  lab-overlay)
    [ "$#" -eq 2 ] || usage
    lab_overlay "$2"
    ;;
  *)
    usage
    ;;
esac
