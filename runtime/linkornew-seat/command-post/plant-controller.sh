#!/bin/sh
# Plant Command Post controller into LinkornEW stage (loopback L5).
# Requires planted CPython (plant-cpython.sh). Does NOT copy tokens.
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

case "$src" in /*) ;; *) printf 'plant-controller: source must be absolute\n' >&2; exit 1 ;; esac
case "$dest" in
  /|/etc|/usr|/usr/local)
    printf 'plant-controller: refusing system path %s\n' "$dest" >&2
    exit 1
    ;;
esac
if [ ! -f "$src/controller/app/main.py" ] || [ ! -d "$src/protocol/python/nexus_protocol" ]; then
  printf 'plant-controller: not a Command Post tree: %s\n' "$src" >&2
  exit 1
fi
if [ ! -x "$src/controller/.venv/bin/python" ]; then
  printf 'plant-controller: need built controller/.venv on build host\n' >&2
  exit 1
fi

cpython="$dest/usr/lib/linkornew/cpython"
if [ ! -x "$cpython/bin/python-linkornew" ]; then
  "$here/plant-cpython.sh" "$dest"
fi

cp_root="$dest/usr/lib/linkornew/command-post"
ctrl="$cp_root/controller"
proto="$cp_root/protocol/python"
state="$dest/var/lib/linkornew/command-post"

mkdir -p "$ctrl" "$proto" "$state"
find "$ctrl" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || true
find "$proto" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || true

cp -a "$src/controller/app" "$ctrl/"
cp -a "$src/controller/pyproject.toml" "$ctrl/"
cp -a "$src/controller/uv.lock" "$ctrl/"
cp -a "$src/protocol/python/nexus_protocol" "$proto/"
cp -a "$src/protocol/python/pyproject.toml" "$proto/"
if [ -f "$src/protocol/python/uv.lock" ]; then
  cp -a "$src/protocol/python/uv.lock" "$proto/"
fi

plant_venv_from_donor "$cpython" "$ctrl/.venv" "$src/controller/.venv"

printf '127.0.0.1\n' > "$state/nexus.host"
printf 'true\n' > "$state/generation_only"
chmod 0644 "$state/nexus.host" "$state/generation_only"
cat > "$state/controller.env.example" <<'EOF'
# Copy to controller.env (mode 0600). Never commit real tokens.
NEXUS_REGISTRATION_TOKEN=
NEXUS_OPERATOR_TOKEN=
NEXUS_OPERATOR_DEFAULT_ROLE=operator
EOF

# Import smoke with planted interpreter + venv site-packages on PYTHONPATH
SP="$ctrl/.venv/lib/python3.14/site-packages"
PYTHONPATH="$SP:$proto:$ctrl" "$ctrl/.venv/bin/python" -c 'from app.main import app; print("controller_import_ok")' \
  >/tmp/plant-controller-smoke.txt 2>/tmp/plant-controller-smoke.err \
  || {
    printf 'plant-controller: import smoke failed\n' >&2
    cat /tmp/plant-controller-smoke.err >&2 || true
    exit 1
  }

printf 'plant-controller: planted %s\n' "$cp_root"
printf 'plant-controller: tokens -> %s/controller.env\n' "$state"
