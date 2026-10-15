#!/usr/bin/env bash
# Stage a LinkornEW payload. This is not the FoFline packager and it does
# not write official out/. One profile per invocation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
profile=""
dest=""

usage() {
  printf 'usage: %s --profile linkornew --dest DIR\n' "$0" >&2
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -ge 2 ]] || usage
      if [[ -n "$profile" && "$profile" != "$2" ]]; then
        printf 'linkornew-package: refusing dual profile %s and %s\n' "$profile" "$2" >&2
        exit 1
      fi
      profile=$2
      shift 2
      ;;
    --dest)
      [[ $# -ge 2 ]] || usage
      dest=$2
      shift 2
      ;;
    --build-image)
      printf 'linkornew-package: refusing FoFline image assemble\n' >&2
      exit 1
      ;;
    -h|--help)
      usage
      ;;
    *)
      printf 'linkornew-package: unknown argument %s\n' "$1" >&2
      exit 2
      ;;
  esac
done

[[ -n "$dest" ]] || usage
if [[ "$profile" != "linkornew" ]]; then
  printf 'linkornew-package: refusing profile %s\n' "${profile:-empty}" >&2
  exit 1
fi

case "$dest" in
  "$ROOT"|"$ROOT"/*)
    printf 'linkornew-package: refusing destination inside the git repo\n' >&2
    exit 1
    ;;
  */out|*/out/*)
    printf 'linkornew-package: refusing official out/\n' >&2
    exit 1
    ;;
esac

"$ROOT/runtime/linkornew-seat/install-linkornew.sh" stage "$dest"
list=$(mktemp)
find "$dest" \( -type f -o -type l \) | sed "s#^$dest/##" | sort > "$list"
{
  printf 'profile=linkornew\n'
  while read -r rel; do
    if [[ -L "$dest/$rel" ]]; then
      printf 'link %s %s\n' "$rel" "$(readlink "$dest/$rel")"
    else
      printf 'file %s %s\n' "$rel" "$(sha256sum "$dest/$rel" | awk '{print $1}')"
    fi
  done < "$list"
} > "$dest/linkornew-manifest.txt"
rm -f "$list"
stage_hash=$(sha256sum "$dest/linkornew-manifest.txt" | awk '{print $1}')
printf 'linkornew-package: staged profile=linkornew manifest=%s\n' "$stage_hash"
printf '%s\n' "$stage_hash" > "$dest/linkornew-manifest.sha256"
