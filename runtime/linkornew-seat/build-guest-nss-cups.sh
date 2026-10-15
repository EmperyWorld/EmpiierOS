#!/bin/sh
# Build NSPR/NSS (MPL-2.0) and libcups (Apache-2.0) from upstream source
# for the staff rootfs glibc (max GLIBC_2.41). Does not copy Fedora .so files.
# Output is not stored in EmpiierOS git. Not official out/. Not MATCH.
set -eu

NSS_URL='https://ftp.mozilla.org/pub/security/nss/releases/NSS_3_101_RTM/src/nss-3.101-with-nspr-4.35.tar.gz'
CUPS_URL='https://github.com/OpenPrinting/cups/releases/download/v2.4.11/cups-2.4.11-source.tar.gz'
LAB=${LINKORNEW_LAB:-$HOME/linkornew-lab}
SRC=$LAB/deps-src
BUILD=$LAB/deps-build
MAX_GLIBC=2.41

usage() {
  printf 'usage: %s --prefix DIR\n' "$0" >&2
  exit 2
}

prefix=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --prefix) prefix=$2; shift 2 ;;
    *) usage ;;
  esac
done
[ -n "$prefix" ] || usage
[ -d "$prefix" ] || { printf 'linkornew-guest-libs: missing prefix %s\n' "$prefix" >&2; exit 1; }

case "$prefix" in
  /usr|/usr/lib|/lib|/lib64)
    printf 'linkornew-guest-libs: refusing host prefix\n' >&2
    exit 1
    ;;
esac

glibc_too_new() {
  file=$1
  python3 - "$file" "$MAX_GLIBC" <<'PY'
import re, sys
path, cap = sys.argv[1], sys.argv[2]
data = open(path, "rb").read()
found = set(re.findall(rb"GLIBC_(\d+\.\d+)", data))
def ver(s):
    return tuple(int(x) for x in s.split("."))
capv = ver(cap)
bad = sorted((m.decode() for m in found if ver(m.decode()) > capv), key=ver)
if bad:
    print("too-new", path, ",".join(bad), file=sys.stderr)
    sys.exit(1)
PY
}

refuse_host_blob() {
  file=$1
  # Fedora copies are a different inode/path; we still reject if the file
  # is a hardlink or identical path under /usr/lib64.
  case "$file" in
    /usr/lib64/*|/lib64/libnss3.so|/lib64/libcups.so*)
      printf 'linkornew-guest-libs: refusing host blob %s\n' "$file" >&2
      exit 1
      ;;
  esac
}

fetch() {
  url=$1
  out=$2
  if [ -f "$out" ]; then
    return 0
  fi
  mkdir -p "$(dirname "$out")"
  curl -fL --retry 3 -o "$out.part" "$url"
  mv "$out.part" "$out"
}

mkdir -p "$SRC" "$BUILD"
fetch "$NSS_URL" "$SRC/nss-3.101-with-nspr-4.35.tar.gz"
fetch "$CUPS_URL" "$SRC/cups-2.4.11-source.tar.gz"

rm -rf "$BUILD/cups"
mkdir -p "$BUILD/nss" "$BUILD/cups"
if [ ! -d "$BUILD/nss/nss" ]; then
  tar -C "$BUILD/nss" --strip-components=1 -xzf "$SRC/nss-3.101-with-nspr-4.35.tar.gz"
fi
# gcc-14+ with -Werror: NSPR 4.35 / NSS 3.101 tests and freebl loader.
sed -i 's/c = strrchr(referencePath, PR_GetDirectorySeparator());/c = (char *)strrchr(referencePath, PR_GetDirectorySeparator());/' \
  "$BUILD/nss/nss/lib/freebl/genload.c"
tar -C "$BUILD/cups" --strip-components=1 -xzf "$SRC/cups-2.4.11-source.tar.gz"

# NSPR/NSS — Mozilla source. Tests off. 64-bit.
# Host gcc treats old NSPR test pointer mismatches as errors; libs do not
# need those tests.
(
  cd "$BUILD/nss/nss" || exit 1
  export BUILD_OPT=1
  export USE_64=1
  export NSS_DISABLE_GTESTS=1
  export NSS_DISABLE_WERROR=1
  export CC="${CC:-gcc}"
  make nss_build_all -j"$(nproc)" XCFLAGS="-Wno-error"
)

nss_lib=$(find "$BUILD/nss" -type d -path '*/dist/*/lib' | head -n 1)
[ -n "$nss_lib" ] && [ -f "$nss_lib/libnss3.so" ]
printf 'linkornew-guest-libs: nss libdir %s\n' "$nss_lib" >&2

# CUPS — library only. OpenSSL from the compiler's headers; we check glibc
# symbols on the .so and do not copy /usr/lib64/libcups.so.2.
(
  cd "$BUILD/cups" || exit 1
  ./configure \
    --prefix=/usr \
    --disable-pam \
    --disable-dbus \
    --disable-gssapi \
    --disable-systemd \
    --disable-avahi \
    --disable-dnssd \
    --disable-libusb \
    --with-dnssd=no \
    --with-tls=openssl
  make -C cups -j"$(nproc)"
)

mkdir -p "$prefix/usr/lib"
for so in \
  libnspr4.so libplc4.so libplds4.so \
  libnss3.so libnssutil3.so libsmime3.so libssl3.so \
  libnssckbi.so libfreebl3.so libfreeblpriv3.so libsoftokn3.so \
  libsqlite3.so
do
  if [ -e "$nss_lib/$so" ]; then
    src=$(readlink -f "$nss_lib/$so")
    refuse_host_blob "$src"
    glibc_too_new "$src"
    install -m 0755 "$src" "$prefix/usr/lib/$so"
  fi
done

cups_so=$(readlink -f "$(find "$BUILD/cups/cups" -name 'libcups.so.2*' ! -type l | head -n 1)")
[ -n "$cups_so" ]
refuse_host_blob "$cups_so"
glibc_too_new "$cups_so"
cp -a "$cups_so" "$prefix/usr/lib/libcups.so.2"

printf 'linkornew-guest-libs: planted nss+cups under %s/usr/lib\n' "$prefix" >&2
