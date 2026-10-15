#!/usr/bin/env bash
# ---
# EmpiierOS · Empery World · https://empery.world
# Author: po&T && TZBrand
# ---
# Purpose:
# Living CP F9 artifact keypair. Durable directory — not /tmp, not git,
# not the installer. init never overwrites an existing SK. Prints PK only.
# Related:
# - docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md (Living CP artifact SK)
# - session/OTA-CAPSULE.md
# ---
set -euo pipefail

XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
DIR="${EMPIIEROS_OTA_OPERATOR_DIR:-$XDG_DATA_HOME/empiieros/ota}"
SK="$DIR/artifact.sk.hex"
PK="$DIR/artifact.pk.hex"
NOTE="$DIR/ORIGIN.txt"

usage() {
  cat <<'EOF'
Usage: scripts/empiieros-ota-operator-key.sh <init|show|pk>

  init  create the durable dir + keypair if missing (never rotate)
  show  print directory + PK fingerprint (never SK)
  pk    print PK hex only
EOF
}

die() {
  echo "ERROR: $*" >&2
  exit 2
}

is_sk_hex() {
  local raw="$1"
  [[ "$raw" =~ ^[0-9a-f]{64}$ ]]
}

derive_pk() {
  local sk_hex="$1"
  python3 - "$sk_hex" <<'PY'
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
import sys
sk = bytes.fromhex(sys.argv[1])
print(Ed25519PrivateKey.from_private_bytes(sk).public_key().public_bytes_raw().hex())
PY
}

cmd_init() {
  mkdir -p "$DIR"
  chmod 0700 "$DIR"
  if [[ -f "$SK" ]]; then
    raw="$(tr -d ' \r\n' <"$SK" | tr 'A-F' 'a-f')"
    if [[ "$raw" == "redacted" ]]; then
      die "living SK at $SK is REDACTED — refuse rotate; restore the real SK"
    fi
    is_sk_hex "$raw" || die "living SK at $SK is not 32-byte hex — refuse rotate"
    pk="$(derive_pk "$raw")"
    printf '%s\n' "$pk" >"$PK"
    chmod 0600 "$SK"
    chmod 0644 "$PK"
    printf '%s\n' "living CP F9 artifact keypair. Never git. Never installer. Never /tmp. SK mode 0600." >"$NOTE"
    chmod 0644 "$NOTE"
    echo "OK: reused living SK"
    echo "DIR=$DIR"
    echo "PK=$pk"
    return 0
  fi
  python3 - "$SK" "$PK" <<'PY'
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from pathlib import Path
import sys
sk_path = Path(sys.argv[1])
pk_path = Path(sys.argv[2])
sk = Ed25519PrivateKey.generate()
sk_path.write_text(sk.private_bytes_raw().hex() + "\n", encoding="utf-8")
pk_path.write_text(sk.public_key().public_bytes_raw().hex() + "\n", encoding="utf-8")
PY
  chmod 0600 "$SK"
  chmod 0644 "$PK"
  printf '%s\n' "living CP F9 artifact keypair. Never git. Never installer. Never /tmp. SK mode 0600." >"$NOTE"
  chmod 0644 "$NOTE"
  echo "OK: minted living SK"
  echo "DIR=$DIR"
  echo "PK=$(tr -d ' \r\n' <"$PK")"
}

cmd_show() {
  [[ -f "$PK" ]] || die "PK missing — run init"
  echo "DIR=$DIR"
  echo "PK=$(tr -d ' \r\n' <"$PK")"
  if [[ -f "$SK" ]]; then
    raw="$(tr -d ' \r\n' <"$SK" | tr 'A-F' 'a-f')"
    if [[ "$raw" == "redacted" ]]; then
      echo "SK=REDACTED"
    elif is_sk_hex "$raw"; then
      echo "SK=present"
    else
      echo "SK=malformed"
    fi
  else
    echo "SK=missing"
  fi
}

cmd_pk() {
  [[ -f "$PK" ]] || die "PK missing — run init"
  tr -d ' \r\n' <"$PK"
  echo
}

case "${1:-}" in
  init) cmd_init ;;
  show) cmd_show ;;
  pk) cmd_pk ;;
  -h|--help|"") usage ;;
  *) usage >&2; exit 2 ;;
esac
