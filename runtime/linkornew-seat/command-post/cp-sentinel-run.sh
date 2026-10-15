#!/bin/sh
# Start planted Command Post sentinel on LinkornEW (loopback :8090).
set -eu

CP_ROOT="${LINKORNEW_CP_ROOT:-/usr/lib/linkornew/command-post}"
SENT="${CP_ROOT}/sentinel"
STATE="${LINKORNEW_CP_STATE:-/var/lib/linkornew/command-post}"
ENV_FILE="${STATE}/sentinel.env"
CTRL_ENV="${STATE}/controller.env"
PY="${SENT}/.venv/bin/python"

if [ ! -x "$PY" ]; then
  printf 'linkornew-cp-sentinel: plant missing (%s)\n' "$PY" >&2
  exit 1
fi
if [ ! -f "$SENT/sentinel_agent/main.py" ]; then
  printf 'linkornew-cp-sentinel: plant missing sentinel_agent\n' >&2
  exit 1
fi

export PYTHONPATH="${CP_ROOT}/protocol/python:${SENT}${PYTHONPATH:+:$PYTHONPATH}"
SP="${SENT}/.venv/lib/python3.14/site-packages"
export PYTHONPATH="${SP}:${PYTHONPATH}"
export NEXUS_SENTINEL_LISTEN_HOST="${NEXUS_SENTINEL_LISTEN_HOST:-127.0.0.1}"
export NEXUS_SENTINEL_LISTEN_PORT="${NEXUS_SENTINEL_LISTEN_PORT:-8090}"
export NEXUS_SENTINEL_CONTROLLER_URL="${NEXUS_SENTINEL_CONTROLLER_URL:-http://127.0.0.1:8088}"

load_env() {
  file=$1
  [ -f "$file" ] || return 0
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      ''|\#*) continue ;;
      *=*)
        key=${line%%=*}
        val=${line#*=}
        case "$key" in
          *[!A-Za-z0-9_]*) continue ;;
        esac
        # Do not override already-set values from sentinel.env
        eval "cur=\${$key:-}"
        if [ -z "$cur" ]; then
          export "$key=$val"
        fi
        ;;
    esac
  done < "$file"
}

# Prefer sentinel.env; fall back to registration token from controller.env
load_env "$ENV_FILE"
if [ -z "${NEXUS_REGISTRATION_TOKEN:-}" ] && [ -z "${NEXUS_SENTINEL_REGISTRATION_TOKEN:-}" ]; then
  load_env "$CTRL_ENV"
fi
if [ -z "${NEXUS_SENTINEL_REGISTRATION_TOKEN:-}" ] && [ -n "${NEXUS_REGISTRATION_TOKEN:-}" ]; then
  export NEXUS_SENTINEL_REGISTRATION_TOKEN="$NEXUS_REGISTRATION_TOKEN"
fi

if [ -z "${NEXUS_SENTINEL_REGISTRATION_TOKEN:-}" ] && [ -z "${NEXUS_REGISTRATION_TOKEN:-}" ]; then
  printf 'linkornew-cp-sentinel: need registration token in %s\n' "$ENV_FILE" >&2
  exit 1
fi

export NEXUS_SENTINEL_AGENT_ID="${NEXUS_SENTINEL_AGENT_ID:-sentinel-linkornew}"
export NEXUS_SENTINEL_HOST_ID="${NEXUS_SENTINEL_HOST_ID:-linkornew-seat}"
export NEXUS_SENTINEL_GENERATION_ONLY="${NEXUS_SENTINEL_GENERATION_ONLY:-true}"
export NEXUS_SENTINEL_EXECUTION_ENABLED="${NEXUS_SENTINEL_EXECUTION_ENABLED:-false}"
export NEXUS_SENTINEL_REQUIRE_CONTROLLER_AUTH="${NEXUS_SENTINEL_REQUIRE_CONTROLLER_AUTH:-true}"

case "$NEXUS_SENTINEL_LISTEN_HOST" in
  0.0.0.0|'::'|'*')
    printf 'linkornew-cp-sentinel: refusing bind %s\n' "$NEXUS_SENTINEL_LISTEN_HOST" >&2
    exit 1
    ;;
esac

cd "$SENT"
exec "$PY" -m uvicorn sentinel_agent.main:app \
  --host "$NEXUS_SENTINEL_LISTEN_HOST" \
  --port "$NEXUS_SENTINEL_LISTEN_PORT"
