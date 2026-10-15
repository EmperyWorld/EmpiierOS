#!/bin/sh
# Shared: build a venv under plant that uses planted CPython and host site-packages
# (third-party only). App code comes from PYTHONPATH, not editable installs.
set -eu

# Args: <cpython-prefix> <venv-dir> <donor-venv>
plant_venv_from_donor() {
  cpython=$1
  venv=$2
  donor=$3
  py=$cpython/bin/python-linkornew
  if [ ! -x "$py" ]; then
    printf 'plant-venv: missing %s (run plant-cpython.sh first)\n' "$py" >&2
    return 1
  fi
  if [ ! -d "$donor/lib/python3.14/site-packages" ]; then
    printf 'plant-venv: donor site-packages missing: %s\n' "$donor" >&2
    return 1
  fi

  rm -rf "$venv"
  # Manual venv layout — avoid recursive wrappers from python -m venv + retarget.
  mkdir -p "$venv/bin" "$venv/lib/python3.14/site-packages"
  sp="$venv/lib/python3.14/site-packages"

  tar -C "$donor/lib/python3.14/site-packages" \
    --exclude='__editable__*' \
    --exclude='__editable___*' \
    --exclude='empery_nexus_controller*' \
    --exclude='empery_nexus_sentinel*' \
    --exclude='nexus_protocol*' \
    --exclude='_virtualenv*' \
    -cf - . | tar -C "$sp" -xf -

  # Relative symlink so the image works when dest was a fuse mount path.
  # Layout: <prefix>/command-post/{controller,sentinel}/.venv/bin → <prefix>/cpython/bin
  rel_py="../../../../cpython/bin/python-linkornew"
  ln -sfn "$rel_py" "$venv/bin/python"
  ln -sfn "$rel_py" "$venv/bin/python3"
  ln -sfn "$rel_py" "$venv/bin/python3.14"

  # Verify the relative link resolves on the build host
  if [ ! -x "$venv/bin/python" ]; then
    printf 'plant-venv: relative python link broken at %s -> %s\n' "$venv/bin/python" "$rel_py" >&2
    return 1
  fi

  cat > "$venv/pyvenv.cfg" <<EOF
home = /usr/lib/linkornew/cpython/bin
include-system-site-packages = false
version = 3.14.7
executable = /usr/lib/linkornew/cpython/bin/python3.14
command = plant-venv-from-donor
EOF
}
