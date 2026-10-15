#!/usr/bin/env bash
# ---
# EmpiierOS · Empery World · https://empery.world
# Author: po&T && TZBrand
# ---
# Purpose:
# E3 lab Measure: pack+publish → plant PK → fetch live edge bytes → stage.
# Does not claim portal-VIP helper metal (IPAddressAllow 10.88.89.1).
# Apply cutover is E4 (`ota_cutover`). Not MATCH.
# ---
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EDGE="${EMPIERO_OS_OTA_EDGE:-https://app.eliteew.ru/empiieros-ota}"
WORK="${TMPDIR:-/tmp}/empiieros-e3-measure-$$"
mkdir -p "$WORK/blobs" "$WORK/state"
cleanup() {
  if [[ -f "$WORK/keys/artifact.sk.hex" ]]; then
    printf 'REDACTED\n' >"$WORK/keys/artifact.sk.hex"
  fi
}
trap cleanup EXIT

printf 'e3-stage-measure\n' >"$WORK/bin"
python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" sign-fixture --out-dir "$WORK/keys" --note e3
python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" pack \
  --out-dir "$WORK/pack" \
  --sk-file "$WORK/keys/artifact.sk.hex" \
  --file "helpers/empiieros-ota-fetch=$WORK/bin" \
  --app-build-id e3-measure \
  --version 0.0.1-e3
python3 "$ROOT/scripts/empiieros-ota-cp-ingest.py" publish \
  --capsule "$WORK/pack/capsule.json" \
  --blob-dir "$WORK/pack/blobs" \
  --pk-file "$WORK/pack/artifact.pk.hex" \
  ${EMPIERO_OS_OTA_PUBLISH_SSH:+--ssh "$EMPIERO_OS_OTA_PUBLISH_SSH"}

# Live edge fetch (same bytes VIP would serve after strip).
curl -sf --connect-timeout 8 "$EDGE/v1/current.json" -o "$WORK/fetched-current.json"
DIGEST="$(python3 -c "import json;print(json.load(open('$WORK/fetched-current.json'))['payload']['files'][0]['sha256'])")"
curl -sf --connect-timeout 8 "$EDGE/v1/blobs/$DIGEST" -o "$WORK/blobs/$DIGEST"
# Attest grant path honesty
ATTEST_CODE="$(curl -s -o "$WORK/attest.json" -w '%{http_code}' --connect-timeout 8 \
  -X POST "$EDGE/v1/attest" -H 'Content-Type: application/json' \
  -d '{"kind":"empiieros_ota_attest_v1"}')"
test "$ATTEST_CODE" = "200"
python3 -c "import json;d=json.load(open('$WORK/attest.json')); assert d.get('grant')=='portal_path_v1'"

python3 "$ROOT/scripts/empiieros-ota-plant-seat-pk.py" \
  --state-dir "$WORK/state" \
  --pk-file "$WORK/pack/artifact.pk.hex"

PK="$(tr -d ' \n' <"$WORK/pack/artifact.pk.hex")"
export EMPIIEROS_OTA_E3_LIVE=1
export EMPIIEROS_OTA_E3_STATE="$WORK/state"
export EMPIIEROS_OTA_E3_CAPSULE="$WORK/fetched-current.json"
export EMPIIEROS_OTA_E3_BLOBS="$WORK/blobs"
export EMPIIEROS_OTA_E3_PK="$PK"
cargo test --manifest-path "$ROOT/runtime/Cargo.toml" --lib \
  e3_stage_from_env_capsule_and_blobs -- --nocapture
test -f "$WORK/state/ota-capsule/current/manifest.json"
test -f "$WORK/state/ota/artifact.pk.hex"
echo "OK: E3 stage Measure (live edge fetch + plant PK → staged; cutover is E4)"
