#!/bin/sh
# Plant a self-contained CPython runtime into a LinkornEW stage/root.
# Required because EmpiierOS Buildroot staff image has no python3.
# Same-major glibc required (lab root and Fedora host are both 2.43 today).
set -eu

usage() {
  printf 'usage: %s <dest-root>\n' "$0" >&2
  exit 2
}

if [ "$#" -ne 1 ]; then
  usage
fi

dest=$1
case "$dest" in
  /|/etc|/usr|/usr/local)
    printf 'plant-cpython: refusing system path %s\n' "$dest" >&2
    exit 1
    ;;
esac

HOST_PY="${LINKORNEW_HOST_PYTHON:-/usr/bin/python3.14}"
HOST_LIBPY="${LINKORNEW_HOST_LIBPYTHON:-/lib64/libpython3.14.so.1.0}"
HOST_STDLIB="${LINKORNEW_HOST_STDLIB:-/usr/lib64/python3.14}"

if [ ! -x "$HOST_PY" ] || [ ! -f "$HOST_LIBPY" ] || [ ! -d "$HOST_STDLIB" ]; then
  printf 'plant-cpython: host python 3.14 layout missing\n' >&2
  exit 1
fi

# glibc fingerprint (best-effort): refuse obvious musl destinations
if [ -f "$dest/lib/ld-musl-x86_64.so.1" ] || [ -f "$dest/lib64/ld-musl-x86_64.so.1" ]; then
  printf 'plant-cpython: refusing musl root (need glibc image)\n' >&2
  exit 1
fi

prefix="$dest/usr/lib/linkornew/cpython"
ver=3.14
rm -rf "$prefix"
mkdir -p "$prefix/bin" "$prefix/lib" "$prefix/lib64"

install -m 0755 "$HOST_PY" "$prefix/bin/python${ver}"
ln -sfn "python${ver}" "$prefix/bin/python3"
ln -sfn "python${ver}" "$prefix/bin/python"

install -m 0755 "$HOST_LIBPY" "$prefix/lib64/libpython${ver}.so.1.0"
ln -sfn "libpython${ver}.so.1.0" "$prefix/lib64/libpython${ver}.so"
ln -sfn "../lib64/libpython${ver}.so.1.0" "$prefix/lib/libpython${ver}.so.1.0"

# Fedora CPython uses platlibdir=lib64 → stdlib under PREFIX/lib64/pythonX.Y
mkdir -p "$prefix/lib64/python${ver}"
tar -C "$HOST_STDLIB" \
  --exclude='__pycache__' \
  --exclude='test' \
  --exclude='tests' \
  --exclude='idle_test' \
  --exclude='site-packages' \
  -cf - . | tar -C "$prefix/lib64/python${ver}" -xf -
# Compat symlink for tools that look under lib/
ln -sfn "../lib64/python${ver}" "$prefix/lib/python${ver}"

# Ship host glibc used by Fedora libpython (Buildroot guest is often older).
# Run python via planted ld-linux so guest /lib64/libc is not mixed in.
for lib in \
  /lib64/ld-linux-x86-64.so.2 \
  /lib64/libc.so.6 \
  /lib64/libm.so.6 \
  /lib64/libpthread.so.0 \
  /lib64/libdl.so.2 \
  /lib64/librt.so.1 \
  /lib64/libutil.so.1
do
  if [ -e "$lib" ]; then
    cp -a "$lib" "$prefix/lib64/" 2>/dev/null || install -m 0755 "$lib" "$prefix/lib64/$(basename "$lib")"
  fi
done
# Resolve any ld-linux symlink to a real file name the wrapper can exec
if [ -L "$prefix/lib64/ld-linux-x86-64.so.2" ]; then
  real=$(readlink -f /lib64/ld-linux-x86-64.so.2)
  install -m 0755 "$real" "$prefix/lib64/ld-linux-x86-64.so.2"
fi

# Wrapper: resolve venv symlinks; exec via planted dynamic loader + lib path.
cat > "$prefix/bin/python-linkornew" <<'EOF'
#!/bin/sh
set -eu
script=$0
while [ -L "$script" ]; do
  target=$(readlink "$script")
  case "$target" in
    /*) script=$target ;;
    *) script=$(dirname "$script")/$target ;;
  esac
done
here=$(CDPATH= cd -- "$(dirname "$script")/.." && pwd)
export PYTHONHOME="$here"
export LD_LIBRARY_PATH="$here/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
loader="$here/lib64/ld-linux-x86-64.so.2"
if [ -x "$loader" ]; then
  exec "$loader" --library-path "$here/lib64" "$here/bin/python3.14" "$@"
fi
exec "$here/bin/python3.14" "$@"
EOF
chmod 0755 "$prefix/bin/python-linkornew"

# Smoke on build host
if ! "$prefix/bin/python-linkornew" -c 'import sys; print(sys.version.split()[0])' >/tmp/plant-cpython-ver.txt 2>/tmp/plant-cpython-err.txt; then
  printf 'plant-cpython: planted interpreter failed on build host\n' >&2
  cat /tmp/plant-cpython-err.txt >&2 || true
  exit 1
fi

printf 'plant-cpython: %s (python %s)\n' "$prefix" "$(cat /tmp/plant-cpython-ver.txt)"
