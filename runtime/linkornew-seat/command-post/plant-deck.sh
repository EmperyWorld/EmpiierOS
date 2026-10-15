#!/bin/sh
# Plant Command Deck static build into a LinkornEW stage or live root.
# Source: command-post/command-deck/dist (Vite build). Tokens stay out.
set -eu

usage() {
  printf 'usage: %s <command-deck-dist-dir> <dest-root>\n' "$0" >&2
  printf '  example: %s /data/repos/ew_s/command-post/command-deck/dist /tmp/linkornew-stage\n' "$0" >&2
  exit 2
}

if [ "$#" -ne 2 ]; then
  usage
fi

src=$1
dest=$2
case "$src" in
  /*) ;;
  *)
    printf 'plant-deck: source must be absolute\n' >&2
    exit 1
    ;;
esac
case "$dest" in
  /|/etc|/usr|/usr/local)
    printf 'plant-deck: refusing system path %s\n' "$dest" >&2
    exit 1
    ;;
esac
if [ ! -f "$src/index.html" ]; then
  printf 'plant-deck: missing %s/index.html (run npm run build in command-deck)\n' "$src" >&2
  exit 1
fi

target="$dest/usr/lib/linkornew/command-deck"
mkdir -p "$target"
# Replace tree atomically enough for stage trees: clear then copy.
find "$target" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
cp -a "$src"/. "$target"/
printf 'plant-deck: planted %s -> %s\n' "$src" "$target"
