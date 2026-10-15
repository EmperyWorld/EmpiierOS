#!/bin/sh
# Bind check for the LinkornEW Command Post units.
# Does not start Command Post. Does not read a token.
set -eu

action=${1:-}
host_file=${2:-}
generation_file=${3:-}

case "$action" in
  controller|sentinel|deck) ;;
  *)
    printf 'linkornew-cp: usage\n' >&2
    exit 2
    ;;
esac

if [ "${LINKORNEW_GENERATION_ONLY:-true}" != "true" ]; then
  printf 'linkornew-cp: refusing executor unlock in this unit\n' >&2
  exit 1
fi

if [ -n "$generation_file" ] && [ -f "$generation_file" ]; then
  mode=$(tr -d '[:space:]' < "$generation_file")
  if [ "$mode" != "true" ]; then
    printf 'linkornew-cp: refusing executor unlock in this unit\n' >&2
    exit 1
  fi
fi

if [ "$action" = "controller" ]; then
  if [ ! -f "$host_file" ]; then
    printf 'linkornew-cp: command_post_not_installed\n' >&2
    exit 1
  fi
  host=$(tr -d '[:space:]' < "$host_file")
  case "$host" in
    ''|0.0.0.0|'::'|'*')
      printf 'linkornew-cp: refusing bind %s\n' "$host" >&2
      exit 1
      ;;
  esac
  case "$host" in
    *[!0-9.]*)
      printf 'linkornew-cp: refusing bind %s\n' "$host" >&2
      exit 1
      ;;
  esac
fi

if [ "$action" = "deck" ]; then
  url=${LINKORNEW_DECK_URL:-http://127.0.0.1:4191}
  case "$url" in
    http://127.0.0.1:4191) ;;
    *)
      printf 'linkornew-cp: refusing deck %s\n' "$url" >&2
      exit 1
      ;;
  esac
  printf 'linkornew-cp: deck %s\n' "$url"
  exit 0
fi

printf 'linkornew-cp: %s bind accepted\n' "$action"
exit 0
