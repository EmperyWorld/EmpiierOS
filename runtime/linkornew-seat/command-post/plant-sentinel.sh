#!/bin/sh
# Plant Command Post sentinel into LinkornEW stage (loopback L5).
# Requires planted CPython + controller plant layout. Does NOT copy tokens.
set -eu

usage() {
  printf 'usage: %s <command-post-repo> <dest-root>\n' "$0" >&2
  exit 2
}

if [ "$#" -ne 2 ]; then
  usage
fi

src=$1
dest=$2
here=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
# shellcheck disable=SC1091
. "$here/plant-venv-lib.sh"

case "$src" in /*) ;; *) printf 'plant-sentinel: source must be absolute\n' >&2; exit 1 ;; esac
case "$dest" in
  /|/etc|/usr|/usr/local)
    printf 'plant-sentinel: refusing system path %s\n' "$dest" >&2
    exit 1
    ;;
esac
if [ ! -f "$src/sentinel/sentinel_agent/main.py" ]; then
  printf 'plant-sentinel: not a Command Post tree: %s\n' "$src" >&2
  exit 1
fi
if [ ! -x "$src/sentinel/.venv/bin/python" ]; then
  printf 'plant-sentinel: need built sentinel/.venv on build host\n' >&2
  exit 1
fi

cpython="$dest/usr/lib/linkornew/cpython"
if [ ! -x "$cpython/bin/python-linkornew" ]; then
  "$here/plant-cpython.sh" "$dest"
fi

cp_root="$dest/usr/lib/linkornew/command-post"
sent="$cp_root/sentinel"
proto="$cp_root/protocol/python"
state="$dest/var/lib/linkornew/command-post"

if [ ! -d "$proto/nexus_protocol" ]; then
  mkdir -p "$proto"
  cp -a "$src/protocol/python/nexus_protocol" "$proto/"
  cp -a "$src/protocol/python/pyproject.toml" "$proto/"
fi

mkdir -p "$sent" "$state"
find "$sent" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || true
cp -a "$src/sentinel/sentinel_agent" "$sent/"
cp -a "$src/sentinel/pyproject.toml" "$sent/"
cp -a "$src/sentinel/uv.lock" "$sent/"

plant_venv_from_donor "$cpython" "$sent/.venv" "$src/sentinel/.venv"

cat > "$state/sentinel.env.example" <<'EOF'
# Copy keys into sentinel.env (mode 0600). Prefer same registration token as controller.
NEXUS_SENTINEL_AGENT_ID=sentinel-linkornew
NEXUS_SENTINEL_HOST_ID=linkornew-seat
NEXUS_SENTINEL_CONTROLLER_URL=http://127.0.0.1:8088
NEXUS_REGISTRATION_TOKEN=
NEXUS_SENTINEL_REGISTRATION_TOKEN=
NEXUS_SENTINEL_REQUIRE_CONTROLLER_AUTH=true
NEXUS_SENTINEL_LISTEN_HOST=127.0.0.1
NEXUS_SENTINEL_LISTEN_PORT=8090
NEXUS_SENTINEL_GENERATION_ONLY=true
NEXUS_SENTINEL_EXECUTION_ENABLED=false
EOF

SP="$sent/.venv/lib/python3.14/site-packages"
PYTHONPATH="$SP:$proto:$sent" "$sent/.venv/bin/python" -c 'from sentinel_agent.main import app; print("sentinel_import_ok")' \
  >/tmp/plant-sentinel-smoke.txt 2>/tmp/plant-sentinel-smoke.err \
  || {
    printf 'plant-sentinel: import smoke failed\n' >&2
    cat /tmp/plant-sentinel-smoke.err >&2 || true
    exit 1
  }

printf 'plant-sentinel: planted %s\n' "$sent"
printf 'plant-sentinel: tokens -> %s/sentinel.env\n' "$state"
