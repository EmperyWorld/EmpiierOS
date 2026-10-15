#!/bin/sh
# Start planted Command Post controller on LinkornEW (loopback).
set -eu

CP_ROOT="${LINKORNEW_CP_ROOT:-/usr/lib/linkornew/command-post}"
CTRL="${CP_ROOT}/controller"
STATE="${LINKORNEW_CP_STATE:-/var/lib/linkornew/command-post}"
HOST_FILE="${STATE}/nexus.host"
ENV_FILE="${STATE}/controller.env"
GEN_FILE="${STATE}/generation_only"
PY="${CTRL}/.venv/bin/python"

if [ ! -x "$PY" ]; then
  printf 'linkornew-cp-controller: plant missing (%s)\n' "$PY" >&2
  exit 1
fi
if [ ! -f "$CTRL/app/main.py" ]; then
  printf 'linkornew-cp-controller: plant missing app/main.py\n' >&2
  exit 1
fi
if [ ! -f "$HOST_FILE" ]; then
  printf 'linkornew-cp-controller: missing %s\n' "$HOST_FILE" >&2
  exit 1
fi

host=$(tr -d '[:space:]' < "$HOST_FILE")
case "$host" in
  ''|0.0.0.0|'::'|'*')
    printf 'linkornew-cp-controller: refusing bind %s\n' "$host" >&2
    exit 1
    ;;
esac
case "$host" in
  *[!0-9.]*)
    printf 'linkornew-cp-controller: refusing bind %s\n' "$host" >&2
    exit 1
    ;;
esac

gen=true
if [ -f "$GEN_FILE" ]; then
  gen=$(tr -d '[:space:]' < "$GEN_FILE")
fi
case "$gen" in
  true|false) ;;
  *)
    printf 'linkornew-cp-controller: bad generation_only=%s\n' "$gen" >&2
    exit 1
    ;;
esac

export NEXUS_HOST="$host"
export NEXUS_PORT="${NEXUS_PORT:-8088}"
export NEXUS_GENERATION_ONLY="$gen"
export NEXUS_RUNTIME_DIR="${NEXUS_RUNTIME_DIR:-$STATE/runtime}"
export NEXUS_SQLITE_PATH="${NEXUS_SQLITE_PATH:-$NEXUS_RUNTIME_DIR/nexus-controller.sqlite3}"
SP="${CTRL}/.venv/lib/python3.14/site-packages"
export PYTHONPATH="${SP}:${CP_ROOT}/protocol/python:${CTRL}${PYTHONPATH:+:$PYTHONPATH}"

mkdir -p "$NEXUS_RUNTIME_DIR"

if [ -f "$ENV_FILE" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      ''|\#*) continue ;;
      *=*)
        key=${line%%=*}
        val=${line#*=}
        case "$key" in
          *[!A-Za-z0-9_]*) continue ;;
        esac
        export "$key=$val"
        ;;
    esac
  done < "$ENV_FILE"
fi

if [ -z "${NEXUS_REGISTRATION_TOKEN:-}" ] || [ -z "${NEXUS_OPERATOR_TOKEN:-}" ]; then
  printf 'linkornew-cp-controller: need tokens in %s\n' "$ENV_FILE" >&2
  exit 1
fi

export NEXUS_HOST="$host"
cd "$CTRL"
exec "$PY" -m uvicorn app.main:app --host "$NEXUS_HOST" --port "$NEXUS_PORT"
