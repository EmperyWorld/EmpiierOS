#!/usr/bin/env bash
# Pinned-identity SSH harness for the temporary EmpiierOS metal laboratory.
# It never commits, builds an image, writes USB media, or changes SSH policy.
set -euo pipefail

HOST="${EMPIIEROS_METAL_HOST:-192.168.8.10}"
USER_NAME="${EMPIIEROS_METAL_USER:-eliteew-ts}"
EXPECTED_FINGERPRINT="${EMPIIEROS_METAL_HOSTKEY_SHA256:-}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
KNOWN_HOSTS="${EMPIIEROS_METAL_KNOWN_HOSTS:-${RUNTIME_DIR}/empiieros-metal-known-hosts}"
CONTROL_PATH="${EMPIIEROS_METAL_CONTROL_PATH:-${RUNTIME_DIR}/empiieros-metal-%C.sock}"
CONTROL_PERSIST="${EMPIIEROS_METAL_CONTROL_PERSIST:-12h}"

# ControlMaster slaves otherwise block on ksshaskpass ("Allow shared connection")
# even with BatchMode=yes. This harness is non-interactive by contract.
export SSH_ASKPASS_REQUIRE=never
unset SSH_ASKPASS

usage() {
  cat <<'EOF'
Usage:
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh connect
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh check
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh probe
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-runtime BINARY UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh rollback-runtime TRANSACTION
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-helper BINARY UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-seat-shield FW NFT UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-seat-shield-watch BIN UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-portal BINARY UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-spa BINARY UNIT
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-wg WG_BIN MODULE_DIR
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-shell GUI ASSETS
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh rollback-shell TRANSACTION
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh deploy-webkit-gst TREE
  EMPIIEROS_METAL_HOSTKEY_SHA256=SHA256:... scripts/metal-live.sh rollback-webkit-gst TRANSACTION

The host key is fetched into a dedicated runtime file and must match the
operator-confirmed SHA-256 fingerprint. Runtime and Shell deployments are
transactional: previous binaries and assets are retained on metal and restored
automatically if activation proof fails. This harness never changes SSH policy.
EOF
}

fail() {
  printf 'metal-live: %s\n' "$*" >&2
  exit 1
}

validate_configuration() {
  [[ "$HOST" =~ ^[A-Za-z0-9._:-]+$ ]] || fail "invalid host"
  [[ "$USER_NAME" =~ ^[a-z_][a-z0-9_-]*$ ]] || fail "invalid user"
  [[ "$EXPECTED_FINGERPRINT" =~ ^SHA256:[A-Za-z0-9+/]+$ ]] \
    || fail "EMPIIEROS_METAL_HOSTKEY_SHA256 is required"
  [[ -d "$RUNTIME_DIR" && ! -L "$RUNTIME_DIR" ]] || fail "unsafe runtime directory"
}

prepare_known_host() {
  local scan fingerprint key_count
  scan="$(mktemp -p "$RUNTIME_DIR" empiieros-metal-keyscan.XXXXXX)"
  trap '[[ -n "${scan:-}" ]] && unlink "$scan" 2>/dev/null || true' RETURN
  ssh-keyscan -T 5 -t ed25519 "$HOST" 2>/dev/null >"$scan" \
    || fail "cannot obtain the metal ED25519 host key"
  key_count="$(awk '$2 == "ssh-ed25519" { count += 1 } END { print count + 0 }' "$scan")"
  [[ "$key_count" == 1 ]] || fail "expected exactly one ED25519 host key"
  fingerprint="$(ssh-keygen -lf "$scan" -E sha256 | awk 'NR == 1 { print $2 }')"
  [[ "$fingerprint" == "$EXPECTED_FINGERPRINT" ]] \
    || fail "host key mismatch: observed=${fingerprint}"
  install -m 0600 "$scan" "$KNOWN_HOSTS"
  trap - RETURN
  unlink "$scan"
}

ssh_options=(
  -o BatchMode=yes
  -o ConnectTimeout=8
  -o HostKeyAlgorithms=ssh-ed25519
  -o StrictHostKeyChecking=yes
  -o "UserKnownHostsFile=${KNOWN_HOSTS}"
  -o ServerAliveInterval=15
  -o ServerAliveCountMax=4
)
if [[ "${EMPIIEROS_METAL_NO_CONTROLMASTER:-0}" == 1 ]]; then
  ssh_options+=(-o ControlMaster=no)
else
  ssh_options+=(
    -o "ControlPath=${CONTROL_PATH}"
  )
fi

ensure_connection() {
  prepare_known_host
  if [[ "${EMPIIEROS_METAL_NO_CONTROLMASTER:-0}" == 1 ]]; then
    ssh "${ssh_options[@]}" "${USER_NAME}@${HOST}" true
    return 0
  fi
  if ssh "${ssh_options[@]}" -O check "${USER_NAME}@${HOST}" >/dev/null 2>&1; then
    return 0
  fi
  env -u SSH_ASKPASS SSH_ASKPASS_REQUIRE=never \
    ssh "${ssh_options[@]}" \
      -o ControlMaster=yes \
      -o "ControlPersist=${CONTROL_PERSIST}" \
      -MNf \
      "${USER_NAME}@${HOST}"
  ssh "${ssh_options[@]}" -O check "${USER_NAME}@${HOST}"
}

ssh_exec() {
  # Callers pass fixed commands or values constrained by the validators above.
  # shellcheck disable=SC2029
  ssh "${ssh_options[@]}" "${USER_NAME}@${HOST}" "$@"
}

scp_to_metal() {
  local source_path="$1"
  local destination="$2"
  scp -q "${ssh_options[@]}" -- "$source_path" "${USER_NAME}@${HOST}:${destination}"
}

probe() {
  ssh_exec 'sudo -n sh -s' <<'REMOTE'
set -eu
printf '%s\n' '=== identity ==='
hostname
uname -r
id
printf '%s\n' '=== system ==='
systemctl is-system-running || true
systemctl --failed --no-legend --no-pager || true
printf '%s\n' '=== runtime ==='
systemctl show empiieros-runtime.service \
  -p ActiveState -p SubState -p MainPID -p NRestarts -p ExecMainStatus \
  -p CapabilityBoundingSet -p AmbientCapabilities -p NoNewPrivileges
pid="$(systemctl show empiieros-runtime.service -p MainPID --value)"
if [ -n "$pid" ] && [ "$pid" != 0 ]; then
  awk '/^(Uid|Gid|CapInh|CapPrm|CapEff|CapBnd|CapAmb|NoNewPrivs|Seccomp|Seccomp_filters):/' \
    "/proc/$pid/status"
fi
stat -Lc '%A %U:%G %a %n' /run/empiieros /run/empiieros/runtime.sock || true
stat -Lc '%A %U:%G %a %n' /run/empiieros-shell /run/empiieros-shell/runtime.sock || true
systemctl is-active empiieros-shell-bridge.socket || true
pgrep -af 'systemd-socket-proxyd --connections-max=16 /run/empiieros/runtime.sock' || true
journalctl -b -u empiieros-runtime.service --no-pager -n 20
printf '%s\n' '=== shell ==='
pgrep -af 'empiieros-shell-gui|labwc|eliteew-ts-kiosk' || true
printf '%s\n' '=== wait-online ==='
systemctl is-active systemd-networkd-wait-online.service || true
systemctl show systemd-networkd-wait-online.service -p ExecStart -p Result -p ActiveState
printf '%s\n' '=== network ==='
ip -br link
ip -4 -br address
ip route
networkctl list --no-pager || true
printf '%s\n' '=== radio ==='
rfkill list || true
iw dev || true
ls -la /lib/firmware/ath10k/QCA6174/hw3.0 2>/dev/null || true
dmesg | awk '/ath10k|firmware-N.bin/ { print }' | tail -n 15
printf '%s\n' '=== hardware ==='
for path in /sys/class/power_supply/*/capacity /sys/class/power_supply/*/status \
  /sys/class/thermal/thermal_zone*/temp /sys/class/drm/card*-*/status; do
  if [ -r "$path" ]; then
    printf '%s=' "$path"
    cat "$path"
  fi
done
printf '%s\n' '=== artifacts ==='
sha256sum /usr/local/bin/empiieros-runtime /usr/local/bin/empiieros-shell-gui
REMOTE
}

deploy_helper() {
  local binary="$1"
  local unit="$2"
  local transaction stage binary_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "air helper binary must be an executable regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "air helper unit must be a regular file"
  rg -q '^ExecStart=/usr/local/sbin/empiieros-air-enroll$' "$unit" \
    || fail "air helper unit has an unexpected ExecStart"
  rg -q 'EMPIIEROS_CAPSULE_SIGNING_PK=' "$unit" \
    || fail "air helper unit must pin the living issuer verifying PK"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-air-helper-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-air-enroll"
  scp_to_metal "$unit" "${stage}/empiieros-air-enroll.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_unit_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/sbin/empiieros-air-enroll"
unit="/etc/systemd/system/empiieros-air-enroll.service"

actual_binary_hash="$(sha256sum "$stage/empiieros-air-enroll" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-air-enroll.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ]
[ "$actual_unit_hash" = "$expected_unit_hash" ]
install -d -o root -g root -m 0700 "$state"
if [ -f "$binary" ]; then
  cp -p "$binary" "$state/empiieros-air-enroll.before"
fi
if [ -f "$unit" ]; then
  cp -p "$unit" "$state/empiieros-air-enroll.service.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-air-enroll" "$binary"
install -o root -g root -m 0644 "$stage/empiieros-air-enroll.service" "$unit"
systemctl daemon-reload
test -x "$binary"
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '
printf 'air helper transaction applied: %s\n' "$transaction"
printf 'binary_sha256=%s\n' "$expected_binary_hash"
REMOTE
}

deploy_seat_shield() {
  local binary="$1"
  local nft_table="$2"
  local unit="$3"
  local transaction stage binary_hash nft_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "Seat Shield helper must be an executable regular file"
  [[ -f "$nft_table" && ! -L "$nft_table" ]] \
    || fail "Seat Shield nft table must be a regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "Seat Shield unit must be a regular file"
  rg -q 'blocked4' "$nft_table" || fail "seat_shield.nft missing blocked4"
  rg -q 'blocked6' "$nft_table" || fail "seat_shield.nft missing blocked6"
  rg -q '^ExecStart=/usr/sbin/nft -f /usr/share/empiieros/seat-shield/seat_shield.nft$' "$unit" \
    || fail "Seat Shield unit has an unexpected ExecStart"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-seat-shield-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  nft_hash="$(sha256sum "$nft_table" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-seat-shield-fw"
  scp_to_metal "$nft_table" "${stage}/seat_shield.nft"
  scp_to_metal "$unit" "${stage}/empiieros-seat-shield-nftables.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$nft_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_nft_hash="$4"
expected_unit_hash="$5"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/libexec/empiieros-seat-shield-fw"
nft_table="/usr/share/empiieros/seat-shield/seat_shield.nft"
unit="/etc/systemd/system/empiieros-seat-shield-nftables.service"
wants="/etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-nftables.service"

actual_binary_hash="$(sha256sum "$stage/empiieros-seat-shield-fw" | awk '{print $1}')"
actual_nft_hash="$(sha256sum "$stage/seat_shield.nft" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-seat-shield-nftables.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ]
[ "$actual_nft_hash" = "$expected_nft_hash" ]
[ "$actual_unit_hash" = "$expected_unit_hash" ]
test -x /usr/sbin/nft
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '

install -d -o root -g root -m 0700 "$state"
install -d -o root -g root -m 0755 /usr/libexec /usr/share/empiieros/seat-shield \
  /etc/systemd/system/multi-user.target.wants
if [ -f "$binary" ]; then
  cp -p "$binary" "$state/empiieros-seat-shield-fw.before"
fi
if [ -f "$nft_table" ]; then
  cp -p "$nft_table" "$state/seat_shield.nft.before"
fi
if [ -f "$unit" ]; then
  cp -p "$unit" "$state/empiieros-seat-shield-nftables.service.before"
fi
if [ -e "$wants" ]; then
  cp -a "$wants" "$state/wants.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-seat-shield-fw" "$binary"
install -o root -g root -m 0644 "$stage/seat_shield.nft" "$nft_table"
install -o root -g root -m 0644 "$stage/empiieros-seat-shield-nftables.service" "$unit"
ln -sfn ../empiieros-seat-shield-nftables.service "$wants"
test -L "$wants"
systemctl daemon-reload
systemctl start empiieros-seat-shield-nftables.service
systemctl is-active --quiet empiieros-seat-shield-nftables.service
/usr/sbin/nft list table inet seat_shield | grep -q blocked4
/usr/sbin/nft list table inet seat_shield | grep -q blocked6
test -x "$binary"
{
  printf 'transaction=%s\n' "$transaction"
  printf 'binary_sha256=%s\n' "$expected_binary_hash"
  printf 'nft_sha256=%s\n' "$expected_nft_hash"
  printf 'unit_sha256=%s\n' "$expected_unit_hash"
} >"$state/applied.env"
chmod 0600 "$state/applied.env"
printf 'seat-shield floor transaction applied: %s\n' "$transaction"
printf 'binary_sha256=%s\n' "$expected_binary_hash"
REMOTE
}

deploy_seat_shield_watch() {
  local binary="$1"
  local unit="$2"
  local transaction stage binary_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "Seat Shield watch helper must be an executable regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "Seat Shield watch unit must be a regular file"
  rg -q '^ExecStart=/usr/libexec/empiieros-seat-shield-watch --state-dir /var/lib/empiieros-runtime$' "$unit" \
    || fail "Seat Shield watch unit has an unexpected ExecStart"
  rg -q '^PrivateTmp=false$' "$unit" \
    || fail "Seat Shield watch unit must set PrivateTmp=false"
  rg -q 'CapabilityBoundingSet' "$unit" && fail "Seat Shield watch unit must not take net-admin capability"
  rg -q 'IPAddressDeny' "$unit" && fail "Seat Shield watch unit must not deny addresses"
  rg -q 'nft' "$unit" && fail "Seat Shield watch unit must not call nft"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-seat-shield-watch-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-seat-shield-watch"
  scp_to_metal "$unit" "${stage}/empiieros-seat-shield-watch.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_unit_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/libexec/empiieros-seat-shield-watch"
unit="/etc/systemd/system/empiieros-seat-shield-watch.service"
wants="/etc/systemd/system/multi-user.target.wants/empiieros-seat-shield-watch.service"

actual_binary_hash="$(sha256sum "$stage/empiieros-seat-shield-watch" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-seat-shield-watch.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ]
[ "$actual_unit_hash" = "$expected_unit_hash" ]
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '

install -d -o root -g root -m 0700 "$state"
install -d -o root -g root -m 0755 /usr/libexec \
  /etc/systemd/system/multi-user.target.wants
if [ -f "$binary" ]; then
  cp -p "$binary" "$state/empiieros-seat-shield-watch.before"
fi
if [ -f "$unit" ]; then
  cp -p "$unit" "$state/empiieros-seat-shield-watch.service.before"
fi
if [ -e "$wants" ]; then
  cp -a "$wants" "$state/wants.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-seat-shield-watch" "$binary"
install -o root -g root -m 0644 "$stage/empiieros-seat-shield-watch.service" "$unit"
ln -sfn ../empiieros-seat-shield-watch.service "$wants"
test -L "$wants"
systemctl daemon-reload
systemctl reset-failed empiieros-seat-shield-watch.service || true
systemctl restart empiieros-seat-shield-watch.service
systemctl is-active --quiet empiieros-seat-shield-watch.service
test -x "$binary"
# Must not share Runtime PrivateTmp. Host /tmp must stay visible to this unit.
systemctl show empiieros-seat-shield-watch.service -p PrivateTmp --value | grep -qx no
{
  printf 'transaction=%s\n' "$transaction"
  printf 'binary_sha256=%s\n' "$expected_binary_hash"
  printf 'unit_sha256=%s\n' "$expected_unit_hash"
} >"$state/applied.env"
chmod 0600 "$state/applied.env"
printf 'seat-shield watch transaction applied: %s\n' "$transaction"
printf 'binary_sha256=%s\n' "$expected_binary_hash"
REMOTE
}

deploy_portal() {
  local binary="$1"
  local unit="$2"
  local transaction stage binary_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "portal helper binary must be an executable regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "portal helper unit must be a regular file"
  rg -q '^ExecStart=/usr/local/sbin/empiieros-portal-up$' "$unit" \
    || fail "portal helper unit has an unexpected ExecStart"
  rg -q 'EMPIIEROS_PORTAL_HELPER=1' "$unit" \
    || fail "portal helper unit must mark the helper process"
  rg -q 'IPAddressDeny' "$unit" && fail "portal helper must not deny addresses"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-portal-helper-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-portal-up"
  scp_to_metal "$unit" "${stage}/empiieros-portal-up.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_unit_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/sbin/empiieros-portal-up"
unit="/etc/systemd/system/empiieros-portal-up.service"

actual_binary_hash="$(sha256sum "$stage/empiieros-portal-up" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-portal-up.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ]
[ "$actual_unit_hash" = "$expected_unit_hash" ]
install -d -o root -g root -m 0700 "$state"
if [ -f "$binary" ]; then
  cp -p "$binary" "$state/empiieros-portal-up.before"
fi
if [ -f "$unit" ]; then
  cp -p "$unit" "$state/empiieros-portal-up.service.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-portal-up" "$binary"
install -o root -g root -m 0644 "$stage/empiieros-portal-up.service" "$unit"
systemctl daemon-reload
test -x "$binary"
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '
printf 'portal helper transaction applied: %s\n' "$transaction"
printf 'binary_sha256=%s\n' "$expected_binary_hash"
REMOTE
}

deploy_spa() {
  local binary="$1"
  local unit="$2"
  local transaction stage binary_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "SPA helper binary must be an executable regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "SPA helper unit must be a regular file"
  rg -q '^ExecStart=/usr/local/sbin/empiieros-spa-fetch$' "$unit" \
    || fail "SPA helper unit has an unexpected ExecStart"
  rg -q 'EMPIIEROS_SPA_HELPER=1' "$unit" \
    || fail "SPA helper unit must mark the helper process"
  rg -q 'IPAddressDeny=any' "$unit" || fail "SPA helper must deny all then allow VIP"
  rg -q 'IPAddressAllow=10.88.89.1' "$unit" || fail "SPA helper must allow only the portal VIP"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-spa-helper-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-spa-fetch"
  scp_to_metal "$unit" "${stage}/empiieros-spa-fetch.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_unit_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/sbin/empiieros-spa-fetch"
unit="/etc/systemd/system/empiieros-spa-fetch.service"

actual_binary_hash="$(sha256sum "$stage/empiieros-spa-fetch" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-spa-fetch.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ]
[ "$actual_unit_hash" = "$expected_unit_hash" ]
install -d -o root -g root -m 0700 "$state"
if [ -f "$binary" ]; then
  cp -p "$binary" "$state/empiieros-spa-fetch.before"
fi
if [ -f "$unit" ]; then
  cp -p "$unit" "$state/empiieros-spa-fetch.service.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-spa-fetch" "$binary"
install -o root -g root -m 0644 "$stage/empiieros-spa-fetch.service" "$unit"
systemctl daemon-reload
test -x "$binary"
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '
printf 'spa helper transaction applied: %s\n' "$transaction"
printf 'binary_sha256=%s\n' "$expected_binary_hash"
REMOTE
}

deploy_wg() {
  local wg_bin="$1"
  local module_dir="$2"
  local transaction stage
  [[ -f "$wg_bin" && ! -L "$wg_bin" && -x "$wg_bin" ]] \
    || fail "empiieros-wg must be an executable regular file"
  [[ -d "$module_dir" && ! -L "$module_dir" ]] || fail "WG module dir missing"
  [[ -f "$module_dir/wireguard.ko" ]] || fail "wireguard.ko missing"
  rg -a -q 'wg-quick' "$wg_bin" && fail "empiieros-wg must not embed wg-quick"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-wg-src-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  ssh_exec "install -d -m 0700 '$stage/modules'"
  scp_to_metal "$wg_bin" "${stage}/empiieros-wg"
  scp_to_metal "$module_dir/libchacha.ko" "${stage}/modules/libchacha.ko"
  scp_to_metal "$module_dir/libpoly1305.ko" "${stage}/modules/libpoly1305.ko"
  scp_to_metal "$module_dir/chacha-x86_64.ko" "${stage}/modules/chacha-x86_64.ko"
  scp_to_metal "$module_dir/poly1305-x86_64.ko" "${stage}/modules/poly1305-x86_64.ko"
  scp_to_metal "$module_dir/libchacha20poly1305.ko" "${stage}/modules/libchacha20poly1305.ko"
  scp_to_metal "$module_dir/libcurve25519-generic.ko" "${stage}/modules/libcurve25519-generic.ko"
  scp_to_metal "$module_dir/curve25519-x86_64.ko" "${stage}/modules/curve25519-x86_64.ko"
  scp_to_metal "$module_dir/udp_tunnel.ko" "${stage}/modules/udp_tunnel.ko"
  scp_to_metal "$module_dir/ip6_udp_tunnel.ko" "${stage}/modules/ip6_udp_tunnel.ko"
  scp_to_metal "$module_dir/wireguard.ko" "${stage}/modules/wireguard.ko"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
kver="$(uname -r)"
extra="/lib/modules/${kver}/extra"
state="/var/lib/empiieros-live-transactions/$transaction"
install -d -o root -g root -m 0700 "$state"
install -d -o root -g root -m 0755 "$extra"
if [ -f /usr/local/sbin/empiieros-wg ]; then
  cp -p /usr/local/sbin/empiieros-wg "$state/empiieros-wg.before"
fi
install -o root -g root -m 0755 "$stage/empiieros-wg" /usr/local/sbin/empiieros-wg
for ko in libchacha.ko libpoly1305.ko chacha-x86_64.ko poly1305-x86_64.ko \
  libchacha20poly1305.ko libcurve25519-generic.ko curve25519-x86_64.ko \
  udp_tunnel.ko ip6_udp_tunnel.ko wireguard.ko
do
  install -o root -g root -m 0644 "$stage/modules/$ko" "$extra/$ko"
done
depmod -a "$kver"
for ko in libchacha libpoly1305 chacha-x86_64 poly1305-x86_64 \
  libchacha20poly1305 libcurve25519-generic curve25519-x86_64 \
  udp_tunnel ip6_udp_tunnel wireguard
do
  modprobe "$ko"
done
ip link add name wgprobe type wireguard
ip link delete wgprobe
test -d /sys/module/wireguard
grep -q 'IPAddressDeny=any' /etc/systemd/system/empiieros-runtime.service
ip -4 -o addr show dev eno1 scope global | grep -q ' inet '
printf 'wg source transaction applied: %s\n' "$transaction"
REMOTE
}

deploy_runtime() {
  local binary="$1"
  local unit="$2"
  local transaction stage binary_hash unit_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "Runtime binary must be an executable regular file"
  [[ -f "$unit" && ! -L "$unit" ]] || fail "Runtime unit must be a regular file"
  rg -q '^ExecStart=/usr/local/bin/empiieros-runtime$' "$unit" \
    || fail "Runtime unit has an unexpected ExecStart"
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-runtime-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  unit_hash="$(sha256sum "$unit" | awk '{print $1}')"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-runtime"
  scp_to_metal "$unit" "${stage}/empiieros-runtime.service"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$unit_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_unit_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/bin/empiieros-runtime"
unit="/etc/systemd/system/empiieros-runtime.service"
applied=0

start_kiosk_gui() {
  # The labwc autostart loop may already be holding after MAX_APP_RESTARTS.
  # pkill then cannot revive the seat; start the binary on the current
  # Wayland seat instead. EmpiierOS must not bind A-F4 to pkill.
  uid="$(id -u eliteew-ts)"
  runtime_dir="/tmp/runtime-${uid}"
  if [ -S "/run/user/${uid}/wayland-0" ]; then
    runtime_dir="/run/user/${uid}"
  fi
  sudo -n -u eliteew-ts env \
    HOME=/home/eliteew-ts \
    USER=eliteew-ts \
    XDG_CONFIG_HOME=/home/eliteew-ts/.config \
    XDG_RUNTIME_DIR="$runtime_dir" \
    WAYLAND_DISPLAY=wayland-0 \
    GDK_BACKEND=wayland,x11 \
    /usr/local/bin/empiieros-shell-gui >/dev/null 2>&1 &
}

wait_for_authorized_shell_snapshot() {
  local started_epoch="$1"
  local nudged=0
  local started_gui=0
  local attempt=0
  # A GUI that started before the socket bounce can sit on dead in-flight
  # requests. One nudge after 12s plus a 90s ceiling covers the 30s backoff.
  while [ "$attempt" -lt 90 ]; do
    if systemctl is-active --quiet empiieros-runtime.service \
      && systemctl is-active --quiet empiieros-shell-bridge.socket \
      && [ -S /run/empiieros-shell/runtime.sock ] \
      && journalctl -u empiieros-runtime.service --since "@$started_epoch" --no-pager \
        | awk '/id=shell-.* ok=true status=observed/ { found=1 } END { exit !found }'; then
      return 0
    fi
    if [ "$nudged" -eq 0 ] && [ "$attempt" -ge 12 ]; then
      pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
      nudged=1
    fi
    if [ "$started_gui" -eq 0 ] && [ "$attempt" -ge 16 ] \
      && ! pgrep -f '^/usr/local/bin/empiieros-shell-gui' >/dev/null; then
      start_kiosk_gui
      started_gui=1
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  return 1
}

restore() {
  trap - EXIT HUP INT TERM
  [ -f "$state/empiieros-runtime.before" ] || return 1
  [ -f "$state/empiieros-runtime.service.before" ] || return 1
  systemctl stop empiieros-runtime.service || true
  install -o root -g root -m 0755 "$state/empiieros-runtime.before" "$binary"
  install -o root -g root -m 0644 "$state/empiieros-runtime.service.before" "$unit"
  systemctl daemon-reload
  systemctl reset-failed empiieros-runtime.service
  restored_epoch="$(date +%s)"
  systemctl start empiieros-runtime.service
  systemctl start empiieros-shell-bridge.socket
  wait_for_authorized_shell_snapshot "$restored_epoch"
}

on_exit() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ "$applied" -ne 1 ]; then
    if ! restore; then
      printf '%s\n' 'CRITICAL: Runtime rollback did not recover authorized IPC' >&2
      exit 70
    fi
  fi
  exit "$status"
}
trap on_exit EXIT HUP INT TERM

actual_binary_hash="$(sha256sum "$stage/empiieros-runtime" | awk '{print $1}')"
actual_unit_hash="$(sha256sum "$stage/empiieros-runtime.service" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ] || {
  printf '%s\n' 'staged Runtime digest mismatch' >&2
  exit 1
}
[ "$actual_unit_hash" = "$expected_unit_hash" ] || {
  printf '%s\n' 'staged Runtime unit digest mismatch' >&2
  exit 1
}

install -d -o root -g root -m 0700 "$state"
cp -p "$binary" "$state/empiieros-runtime.before"
cp -p "$unit" "$state/empiieros-runtime.service.before"
sha256sum "$state/empiieros-runtime.before" "$state/empiieros-runtime.service.before" \
  >"$state/before.sha256"
install -o root -g root -m 0755 "$stage/empiieros-runtime" "$binary"
install -o root -g root -m 0644 "$stage/empiieros-runtime.service" "$unit"
systemctl daemon-reload
started_epoch="$(date +%s)"
systemctl reset-failed empiieros-runtime.service
systemctl restart empiieros-runtime.service
systemctl is-active --quiet empiieros-runtime.service
systemctl start empiieros-shell-bridge.socket

if wait_for_authorized_shell_snapshot "$started_epoch"; then
  applied=1
else
  printf '%s\n' 'authorized Shell snapshot was not observed; rolling back' >&2
  exit 1
fi

{
  printf 'transaction=%s\n' "$transaction"
  printf 'binary_sha256=%s\n' "$expected_binary_hash"
  printf 'unit_sha256=%s\n' "$expected_unit_hash"
  printf 'applied_epoch=%s\n' "$started_epoch"
} >"$state/applied.env"
chmod 0600 "$state/applied.env"
trap - EXIT HUP INT TERM
printf 'runtime transaction applied: %s\n' "$transaction"
REMOTE
}

asset_tree_hash() {
  local root="$1"
  (
    cd "$root"
    LC_ALL=C find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
  ) | sha256sum | awk '{print $1}'
}

deploy_shell() {
  local binary="$1"
  local assets="$2"
  local transaction stage binary_hash assets_hash
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "Shell GUI binary must be an executable regular file"
  [[ -d "$assets" && ! -L "$assets" && -f "$assets/index.html" && ! -L "$assets/index.html" ]] \
    || fail "Shell assets must be a regular directory containing index.html"
  if find "$assets" -type l -print -quit | rg -q .; then
    fail "Shell assets must not contain symbolic links"
  fi
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-shell-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  binary_hash="$(sha256sum "$binary" | awk '{print $1}')"
  assets_hash="$(asset_tree_hash "$assets")"

  ssh_exec "install -d -m 0700 '$stage'"
  scp_to_metal "$binary" "${stage}/empiieros-shell-gui"
  tar -C "$assets" -cf - . | ssh_exec "cat >'${stage}/shell-assets.tar'"

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$binary_hash' '$assets_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_binary_hash="$3"
expected_assets_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/bin/empiieros-shell-gui"
assets="/opt/empiieros/shell"
applied=0

tree_hash() {
  (
    cd "$1"
    LC_ALL=C find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
  ) | sha256sum | awk '{print $1}'
}

wait_for_gui() {
  local started="$1"
  local attempt=0
  while [ "$attempt" -lt 25 ]; do
    pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
    if [ -n "$pid" ] && [ "$pid" != "$started" ]; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  return 1
}

replace_assets() {
  rm -rf -- "$assets"
  install -d -o root -g root -m 0755 "$assets"
  tar -C "$assets" -xf "$1"
  find "$assets" -type d -exec chmod 0755 {} +
  find "$assets" -type f -exec chmod 0644 {} +
  chown -R root:root "$assets"
}

lock_labwc_kiosk_close() {
  rc=/home/eliteew-ts/.config/labwc/rc.xml
  [ -f "$rc" ] || return 0
  # Consume Alt+F4. Do not pkill the OS face.
  sed -i \
    's#command="pkill -f /usr/local/bin/empiieros-shell-gui"#command="true"#g' \
    "$rc"
}

reconfigure_labwc() {
  runtime_dir="$(getent passwd eliteew-ts | awk -F: '{print "/run/user/" $3}')"
  if [ -d "$runtime_dir" ]; then
    sudo -n -u eliteew-ts env \
      "XDG_RUNTIME_DIR=$runtime_dir" \
      WAYLAND_DISPLAY=wayland-0 \
      labwc --reconfigure >/dev/null 2>&1 || true
  fi
}

revive_gui() {
  old_pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
  pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
  if wait_for_gui "$old_pid"; then
    return 0
  fi
  # The kiosk app loop may already be holding after MAX_APP_RESTARTS.
  # Relight the GUI on the current seat. Do not kill labwc or getty.
  uid="$(id -u eliteew-ts)"
  runtime_dir="/tmp/runtime-${uid}"
  if [ -S "/run/user/${uid}/wayland-0" ]; then
    runtime_dir="/run/user/${uid}"
  fi
  # fs.protected_regular blocks root from truncating another
  # user's file in sticky /tmp. Recreate as the seat user.
  rm -f /tmp/empiieros-meet-gst.log
  sudo -n -u eliteew-ts touch /tmp/empiieros-meet-gst.log
  chmod 0644 /tmp/empiieros-meet-gst.log
  chown eliteew-ts:eliteew-ts /tmp/empiieros-meet-gst.log
  sudo -n -u eliteew-ts env \
    HOME=/home/eliteew-ts \
    USER=eliteew-ts \
    XDG_CONFIG_HOME=/home/eliteew-ts/.config \
    XDG_RUNTIME_DIR="$runtime_dir" \
    WAYLAND_DISPLAY=wayland-0 \
    GDK_BACKEND=wayland,x11 \
    GST_DEBUG_NO_COLOR=1 \
    GST_DEBUG=2,webrtcbin:5,webkitwebrtc*:6,GST_ELEMENT_FACTORY:4,dtls*:5 \
    GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log \
    /usr/local/bin/empiieros-shell-gui >/dev/null 2>&1 &
  wait_for_gui ""
}

restore() {
  trap - EXIT HUP INT TERM
  [ -f "$state/empiieros-shell-gui.before" ] || return 1
  [ -f "$state/shell-assets.before.tar" ] || return 1
  install -o root -g root -m 0755 "$state/empiieros-shell-gui.before" "$binary"
  replace_assets "$state/shell-assets.before.tar"
  if [ -f "$state/rc.xml.before" ]; then
    install -o eliteew-ts -g eliteew-ts -m 0644 \
      "$state/rc.xml.before" /home/eliteew-ts/.config/labwc/rc.xml || true
  fi
  lock_labwc_kiosk_close
  reconfigure_labwc
  revive_gui
}

on_exit() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ "$applied" -ne 1 ]; then
    if ! restore; then
      printf '%s\n' 'CRITICAL: Shell rollback did not recover the GUI process' >&2
      exit 70
    fi
  fi
  exit "$status"
}
trap on_exit EXIT HUP INT TERM

actual_binary_hash="$(sha256sum "$stage/empiieros-shell-gui" | awk '{print $1}')"
[ "$actual_binary_hash" = "$expected_binary_hash" ] || {
  printf '%s\n' 'staged Shell GUI digest mismatch' >&2
  exit 1
}

install -d -o root -g root -m 0700 "$state"
cp -p "$binary" "$state/empiieros-shell-gui.before"
tar -C "$assets" -cf "$state/shell-assets.before.tar" .
if [ -f /home/eliteew-ts/.config/labwc/rc.xml ]; then
  cp -p /home/eliteew-ts/.config/labwc/rc.xml "$state/rc.xml.before"
fi

install -o root -g root -m 0755 "$stage/empiieros-shell-gui" "$binary"
replace_assets "$stage/shell-assets.tar"
[ "$(tree_hash "$assets")" = "$expected_assets_hash" ] || {
  printf '%s\n' 'installed Shell asset digest mismatch' >&2
  exit 1
}

if [ -f /home/eliteew-ts/.config/labwc/rc.xml ]; then
  sed -i '/<action name="Maximize"\/>/d' /home/eliteew-ts/.config/labwc/rc.xml
  lock_labwc_kiosk_close
  reconfigure_labwc
fi

revive_gui
applied=1

{
  printf 'transaction=%s\n' "$transaction"
  printf 'gui_sha256=%s\n' "$expected_binary_hash"
  printf 'assets_sha256=%s\n' "$expected_assets_hash"
} >"$state/applied.env"
chmod 0600 "$state/applied.env"
trap - EXIT HUP INT TERM
printf 'shell transaction applied: %s\n' "$transaction"
REMOTE
}

rollback_shell() {
  local transaction="$1"
  [[ "$transaction" =~ ^[0-9]{8}T[0-9]{6}Z-shell-[0-9]+$ ]] \
    || fail "invalid Shell transaction id"
  ssh_exec "sudo -n sh -s -- '$transaction'" <<'REMOTE'
set -eu
transaction="$1"
state="/var/lib/empiieros-live-transactions/$transaction"
binary="/usr/local/bin/empiieros-shell-gui"
assets="/opt/empiieros/shell"
[ -f "$state/empiieros-shell-gui.before" ]
[ -f "$state/shell-assets.before.tar" ]
install -o root -g root -m 0755 "$state/empiieros-shell-gui.before" "$binary"
rm -rf -- "$assets"
install -d -o root -g root -m 0755 "$assets"
tar -C "$assets" -xf "$state/shell-assets.before.tar"
find "$assets" -type d -exec chmod 0755 {} +
find "$assets" -type f -exec chmod 0644 {} +
chown -R root:root "$assets"
if [ -f "$state/rc.xml.before" ]; then
  install -o eliteew-ts -g eliteew-ts -m 0644 \
    "$state/rc.xml.before" /home/eliteew-ts/.config/labwc/rc.xml || true
fi
if [ -f /home/eliteew-ts/.config/labwc/rc.xml ]; then
  sed -i \
    's#command="pkill -f /usr/local/bin/empiieros-shell-gui"#command="true"#g' \
    /home/eliteew-ts/.config/labwc/rc.xml
  runtime_dir="$(getent passwd eliteew-ts | awk -F: '{print "/run/user/" $3}')"
  if [ -d "$runtime_dir" ]; then
    sudo -n -u eliteew-ts env \
      "XDG_RUNTIME_DIR=$runtime_dir" \
      WAYLAND_DISPLAY=wayland-0 \
      labwc --reconfigure >/dev/null 2>&1 || true
  fi
fi
old_pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
attempt=0
while [ "$attempt" -lt 20 ]; do
  pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
  if [ -n "$pid" ] && [ "$pid" != "$old_pid" ]; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 1
done
[ "$attempt" -lt 20 ]
printf 'shell transaction rolled back: %s\n' "$transaction"
REMOTE
}

webkit_gst_rel_allowed() {
  local rel="$1"
  case "$rel" in
    usr/lib/libwebkit*|usr/lib/libjavascriptcore*|usr/lib/libgst*| \
    usr/lib/gstreamer-1.0/*|usr/lib/libnice*|usr/lib/libopus*| \
    usr/lib/libvpx*|usr/lib/libsrtp*|usr/lib/libav*|usr/lib/libsw*| \
    usr/lib/libpostproc*|usr/lib/libgudev*|usr/lib/liborc*| \
    usr/lib/libXv*|usr/lib/libseccomp*|usr/lib/webkit2gtk-4.1/*| \
    usr/libexec/webkit2gtk-4.1/*|usr/libexec/gstreamer-1.0/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

deploy_webkit_gst() {
  local tree="$1"
  local transaction stage tree_hash webkit_hash rel
  local webkit_so0 webkit_real
  [[ -d "$tree" && ! -L "$tree" ]] || fail "WebKit/GST tree must be a regular directory"
  webkit_so0="${tree}/usr/lib/libwebkit2gtk-4.1.so.0"
  [[ -L "$webkit_so0" ]] || fail "WebKit/GST tree must contain libwebkit2gtk-4.1.so.0 symlink"
  webkit_real="$(readlink -f "$webkit_so0")"
  [[ -n "$webkit_real" && -f "$webkit_real" && ! -L "$webkit_real" ]] \
    || fail "WebKit/GST so.0 must resolve to a regular library"
  case "$webkit_real" in
    "${tree}/usr/lib/libwebkit2gtk-4.1.so.0."*) ;;
    *) fail "WebKit/GST so.0 must resolve inside tree usr/lib" ;;
  esac
  [[ "$(basename "$webkit_real")" =~ ^libwebkit2gtk-4\.1\.so\.0\.[0-9]+\.[0-9]+$ ]] \
    || fail "WebKit/GST so.0 resolved to an unexpected name"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstwebrtc.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstwebrtc.so" ]] \
    || fail "WebKit/GST tree must contain libgstwebrtc.so"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstdtls.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstdtls.so" ]] \
    || fail "WebKit/GST tree must contain libgstdtls.so"
  grep -a -F 'bound on-peer-certificate after create_connection' \
    "${tree}/usr/lib/gstreamer-1.0/libgstdtls.so" >/dev/null \
    || fail "libgstdtls.so missing dtlsdec peer-cert rebind"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstrtpmanager.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstrtpmanager.so" ]] \
    || fail "WebKit/GST tree must contain libgstrtpmanager.so (rtpbin)"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstrtp.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstrtp.so" ]] \
    || fail "WebKit/GST tree must contain libgstrtp.so (rtpfunnel)"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstvideorate.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstvideorate.so" ]] \
    || fail "WebKit/GST tree must contain libgstvideorate.so"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstvideotestsrc.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstvideotestsrc.so" ]] \
    || fail "WebKit/GST tree must contain libgstvideotestsrc.so"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstaudiotestsrc.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstaudiotestsrc.so" ]] \
    || fail "WebKit/GST tree must contain libgstaudiotestsrc.so"
  [[ -f "${tree}/usr/lib/gstreamer-1.0/libgstinterleave.so" && ! -L "${tree}/usr/lib/gstreamer-1.0/libgstinterleave.so" ]] \
    || fail "WebKit/GST tree must contain libgstinterleave.so (deinterleave)"
  [[ -f "${tree}/usr/libexec/webkit2gtk-4.1/WebKitWebProcess" && ! -L "${tree}/usr/libexec/webkit2gtk-4.1/WebKitWebProcess" ]] \
    || fail "WebKit/GST tree must contain WebKitWebProcess"
  while IFS= read -r -d '' rel; do
    rel="${rel#./}"
    webkit_gst_rel_allowed "$rel" \
      || fail "WebKit/GST tree has a path outside the allowlist: ${rel}"
  done < <(find "$tree" \( -type f -o -type l \) -printf '%P\0')
  if find "$tree" -type l -lname '/*' -print -quit | rg -q .; then
    fail "WebKit/GST tree must not contain absolute symbolic links"
  fi
  transaction="$(date -u +%Y%m%dT%H%M%SZ)-webkit-gst-$RANDOM"
  stage="/home/${USER_NAME}/.cache/empiieros-live/${transaction}"
  webkit_hash="$(sha256sum "$webkit_real" | awk '{print $1}')"
  tree_hash="$(
    (
      cd "$tree"
      LC_ALL=C find . \( -type f -o -type l \) -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
    ) | sha256sum | awk '{print $1}'
  )"

  ssh_exec "install -d -m 0700 '$stage'"
  local_tar="${RUNTIME_DIR}/empiieros-${transaction}.tar"
  printf 'metal-live: packing WebKit/GST tree\n'
  tar -C "$tree" -cf "$local_tar" .
  printf 'metal-live: copying WebKit/GST tree to the seat\n'
  scp "${ssh_options[@]}" -- "$local_tar" "${USER_NAME}@${HOST}:${stage}/webkit-gst.tar"
  unlink "$local_tar"
  printf 'metal-live: activating WebKit/GST on the seat\n'

  ssh_exec \
    "sudo -n sh -s -- '$transaction' '$stage' '$webkit_hash' '$tree_hash'" <<'REMOTE'
set -eu
transaction="$1"
stage="$2"
expected_webkit_hash="$3"
expected_tree_hash="$4"
state="/var/lib/empiieros-live-transactions/$transaction"
applied=0
so0="/usr/lib/libwebkit2gtk-4.1.so.0"
webkit="$(readlink -f "$so0" 2>/dev/null || true)"

tree_hash() {
  (
    cd "$1"
    LC_ALL=C find . \( -type f -o -type l \) -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
  ) | sha256sum | awk '{print $1}'
}

wait_for_gui() {
  local started="$1"
  local attempt=0
  while [ "$attempt" -lt 40 ]; do
    pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
    if [ -n "$pid" ] && [ "$pid" != "$started" ]; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  return 1
}

revive_gui() {
  old_pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
  pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
  attempt=0
  while [ "$attempt" -lt 20 ]; do
    pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
    if [ -z "$pid" ] || [ "$pid" = "$old_pid" ]; then
      :
    else
      pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
    fi
    if [ -z "$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)" ]; then
      break
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  uid="$(id -u eliteew-ts)"
  runtime_dir="/tmp/runtime-${uid}"
  if [ -S "/run/user/${uid}/wayland-0" ]; then
    runtime_dir="/run/user/${uid}"
  fi
  # fs.protected_regular blocks root from truncating another
  # user's file in sticky /tmp. Recreate as the seat user.
  rm -f /tmp/empiieros-meet-gst.log
  sudo -n -u eliteew-ts touch /tmp/empiieros-meet-gst.log
  chmod 0644 /tmp/empiieros-meet-gst.log
  chown eliteew-ts:eliteew-ts /tmp/empiieros-meet-gst.log
  sudo -n -u eliteew-ts env \
    HOME=/home/eliteew-ts \
    USER=eliteew-ts \
    XDG_CONFIG_HOME=/home/eliteew-ts/.config \
    XDG_RUNTIME_DIR="$runtime_dir" \
    WAYLAND_DISPLAY=wayland-0 \
    GDK_BACKEND=wayland,x11 \
    GST_DEBUG_NO_COLOR=1 \
    GST_DEBUG=2,webrtcbin:5,webkitwebrtc*:6,GST_ELEMENT_FACTORY:4,dtls*:5 \
    GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log \
    /usr/local/bin/empiieros-shell-gui >/dev/null 2>&1 &
  wait_for_gui ""
}

restore() {
  trap - EXIT HUP INT TERM
  if [ -f "$state/new.list" ]; then
    while IFS= read -r rel; do
      [ -n "$rel" ] || continue
      if [ -L "/$rel" ] || [ -f "/$rel" ]; then
        rm -f -- "/$rel"
      fi
    done <"$state/new.list"
  fi
  if [ -s "$state/before.tar" ]; then
    tar -C / -xf "$state/before.tar"
  fi
  if command -v ldconfig >/dev/null 2>&1; then
    ldconfig
  fi
  revive_gui
}

on_exit() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ "$applied" -ne 1 ]; then
    if ! restore; then
      printf '%s\n' 'CRITICAL: WebKit/GST rollback did not recover the GUI process' >&2
      exit 70
    fi
  fi
  exit "$status"
}
trap on_exit EXIT HUP INT TERM

install -d -o root -g root -m 0700 "$state"
: >"$state/replaced.list"
: >"$state/new.list"
tar -tf "$stage/webkit-gst.tar" | while IFS= read -r entry; do
  rel="${entry#./}"
  case "$rel" in
    ""|*/.) continue ;;
  esac
  case "$rel" in
    */) continue ;;
  esac
  if [ -e "/$rel" ] || [ -L "/$rel" ]; then
    printf '%s\n' "$rel"
  fi
done | LC_ALL=C sort -u >"$state/replaced.list"
tar -tf "$stage/webkit-gst.tar" | while IFS= read -r entry; do
  rel="${entry#./}"
  case "$rel" in
    ""|*/.) continue ;;
  esac
  case "$rel" in
    */) continue ;;
  esac
  if [ ! -e "/$rel" ] && [ ! -L "/$rel" ]; then
    printf '%s\n' "$rel"
  fi
done | LC_ALL=C sort -u >"$state/new.list"
if [ -s "$state/replaced.list" ]; then
  tar -C / -cf "$state/before.tar" -T "$state/replaced.list"
else
  : >"$state/before.tar"
fi

tar -C / --no-same-owner -xf "$stage/webkit-gst.tar"
so0="/usr/lib/libwebkit2gtk-4.1.so.0"
webkit="$(readlink -f "$so0" 2>/dev/null || true)"
chown root:root "$webkit" \
  /usr/lib/gstreamer-1.0/libgstwebrtc.so \
  /usr/lib/gstreamer-1.0/libgstdtls.so \
  /usr/lib/gstreamer-1.0/libgstrtpmanager.so \
  /usr/lib/gstreamer-1.0/libgstrtp.so \
  /usr/lib/gstreamer-1.0/libgstvideorate.so \
  /usr/lib/gstreamer-1.0/libgstvideotestsrc.so \
  /usr/lib/gstreamer-1.0/libgstaudiotestsrc.so \
  /usr/lib/gstreamer-1.0/libgstinterleave.so \
  /usr/lib/libseccomp.so.2.5.5 2>/dev/null || true
if command -v ldconfig >/dev/null 2>&1; then
  ldconfig
fi
[ -n "$webkit" ] && [ -f "$webkit" ] || {
  printf '%s\n' 'installed WebKit library missing' >&2
  exit 1
}
actual_webkit_hash="$(sha256sum "$webkit" | awk '{print $1}')"
[ "$actual_webkit_hash" = "$expected_webkit_hash" ] || {
  printf '%s\n' 'installed WebKit digest mismatch' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstwebrtc.so ] || {
  printf '%s\n' 'libgstwebrtc.so missing after extract' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstdtls.so ] || {
  printf '%s\n' 'libgstdtls.so missing after extract' >&2
  exit 1
}
grep -a -F 'bound on-peer-certificate after create_connection' \
  /usr/lib/gstreamer-1.0/libgstdtls.so >/dev/null || {
  printf '%s\n' 'dtlsdec peer-cert rebind missing from libgstdtls.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstrtpmanager.so ] || {
  printf '%s\n' 'libgstrtpmanager.so missing after extract' >&2
  exit 1
}
grep -a -F 'rtpbin' /usr/lib/gstreamer-1.0/libgstrtpmanager.so >/dev/null || {
  printf '%s\n' 'rtpbin string missing from libgstrtpmanager.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstrtp.so ] || {
  printf '%s\n' 'libgstrtp.so missing after extract' >&2
  exit 1
}
grep -a -F 'rtph264pay' /usr/lib/gstreamer-1.0/libgstrtp.so >/dev/null || {
  printf '%s\n' 'rtph264pay string missing from libgstrtp.so' >&2
  exit 1
}
grep -a -F 'rtpfunnel' /usr/lib/gstreamer-1.0/libgstrtpmanager.so >/dev/null || {
  printf '%s\n' 'rtpfunnel string missing from libgstrtpmanager.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstvideorate.so ] || {
  printf '%s\n' 'libgstvideorate.so missing after extract' >&2
  exit 1
}
grep -a -F 'videorate' /usr/lib/gstreamer-1.0/libgstvideorate.so >/dev/null || {
  printf '%s\n' 'videorate string missing from libgstvideorate.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstvideotestsrc.so ] || {
  printf '%s\n' 'libgstvideotestsrc.so missing after extract' >&2
  exit 1
}
grep -a -F 'videotestsrc' /usr/lib/gstreamer-1.0/libgstvideotestsrc.so >/dev/null || {
  printf '%s\n' 'videotestsrc string missing from libgstvideotestsrc.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstaudiotestsrc.so ] || {
  printf '%s\n' 'libgstaudiotestsrc.so missing after extract' >&2
  exit 1
}
grep -a -F 'audiotestsrc' /usr/lib/gstreamer-1.0/libgstaudiotestsrc.so >/dev/null || {
  printf '%s\n' 'audiotestsrc string missing from libgstaudiotestsrc.so' >&2
  exit 1
}
[ -f /usr/lib/gstreamer-1.0/libgstinterleave.so ] || {
  printf '%s\n' 'libgstinterleave.so missing after extract' >&2
  exit 1
}
grep -a -F 'deinterleave' /usr/lib/gstreamer-1.0/libgstinterleave.so >/dev/null || {
  printf '%s\n' 'deinterleave string missing from libgstinterleave.so' >&2
  exit 1
}
[ -e /usr/lib/libnice.so.10 ] || {
  printf '%s\n' 'libnice.so.10 missing after extract' >&2
  exit 1
}
grep -a -F 'Enable RTCPeerConnection' "$webkit" >/dev/null || {
  printf '%s\n' 'RTCPeerConnection string missing from WebKit' >&2
  exit 1
}
grep -a -F 'ensureDtlsCertificatePem' "$webkit" >/dev/null || {
  printf '%s\n' 'DTLS PEM lock missing from WebKit' >&2
  exit 1
}
grep -a -F 'Applied DTLS PEM' "$webkit" >/dev/null || {
  printf '%s\n' 'DTLS PEM apply string missing from WebKit' >&2
  exit 1
}
find /home/eliteew-ts/.cache/gstreamer-1.0 /root/.cache/gstreamer-1.0 \
  -type f -name 'registry*' -delete 2>/dev/null || true
loader=/lib64/ld-linux-x86-64.so.2
if [ -x "$loader" ]; then
  "$loader" --list "$webkit" >"$state/ld.webkit" 2>&1 || true
  if grep -q 'not found' "$state/ld.webkit"; then
    cat "$state/ld.webkit" >&2
    printf '%s\n' 'WebKit loader is missing a shared library' >&2
    exit 1
  fi
fi

if ! revive_gui; then
  uid="$(id -u eliteew-ts)"
  runtime_dir="/tmp/runtime-${uid}"
  if [ -S "/run/user/${uid}/wayland-0" ]; then
    runtime_dir="/run/user/${uid}"
  fi
  sudo -n -u eliteew-ts env \
    HOME=/home/eliteew-ts \
    USER=eliteew-ts \
    XDG_CONFIG_HOME=/home/eliteew-ts/.config \
    XDG_RUNTIME_DIR="$runtime_dir" \
    WAYLAND_DISPLAY=wayland-0 \
    GDK_BACKEND=wayland,x11 \
    /usr/local/bin/empiieros-shell-gui >"$state/gui.err" 2>&1 &
  sleep 3
  if [ -s "$state/gui.err" ]; then
    cat "$state/gui.err" >&2
  fi
  printf '%s\n' 'GUI did not stay up after WebKit/GST extract' >&2
  exit 1
fi
applied=1

{
  printf 'transaction=%s\n' "$transaction"
  printf 'webkit_sha256=%s\n' "$expected_webkit_hash"
  printf 'tree_sha256=%s\n' "$expected_tree_hash"
} >"$state/applied.env"
chmod 0600 "$state/applied.env"
trap - EXIT HUP INT TERM
printf 'webkit-gst transaction applied: %s\n' "$transaction"
REMOTE
}

rollback_webkit_gst() {
  local transaction="$1"
  [[ "$transaction" =~ ^[0-9]{8}T[0-9]{6}Z-webkit-gst-[0-9]+$ ]] \
    || fail "invalid WebKit/GST transaction id"
  ssh_exec "sudo -n sh -s -- '$transaction'" <<'REMOTE'
set -eu
transaction="$1"
state="/var/lib/empiieros-live-transactions/$transaction"
[ -d "$state" ] || {
  printf '%s\n' 'webkit-gst transaction is missing' >&2
  exit 1
}

wait_for_gui() {
  local started="$1"
  local attempt=0
  while [ "$attempt" -lt 40 ]; do
    pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
    if [ -n "$pid" ] && [ "$pid" != "$started" ]; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  return 1
}

revive_gui() {
  old_pid="$(pgrep -f '^/usr/local/bin/empiieros-shell-gui' || true)"
  pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
  if wait_for_gui "$old_pid"; then
    return 0
  fi
  uid="$(id -u eliteew-ts)"
  runtime_dir="/tmp/runtime-${uid}"
  if [ -S "/run/user/${uid}/wayland-0" ]; then
    runtime_dir="/run/user/${uid}"
  fi
  sudo -n -u eliteew-ts env \
    HOME=/home/eliteew-ts \
    USER=eliteew-ts \
    XDG_CONFIG_HOME=/home/eliteew-ts/.config \
    XDG_RUNTIME_DIR="$runtime_dir" \
    WAYLAND_DISPLAY=wayland-0 \
    GDK_BACKEND=wayland,x11 \
    /usr/local/bin/empiieros-shell-gui >/dev/null 2>&1 &
  wait_for_gui ""
}

if [ -f "$state/new.list" ]; then
  while IFS= read -r rel; do
    [ -n "$rel" ] || continue
    if [ -L "/$rel" ] || [ -f "/$rel" ]; then
      rm -f -- "/$rel"
    fi
  done <"$state/new.list"
fi
if [ -s "$state/before.tar" ]; then
  tar -C / -xf "$state/before.tar"
fi
if command -v ldconfig >/dev/null 2>&1; then
  ldconfig
fi
revive_gui
printf 'webkit-gst transaction rolled back: %s\n' "$transaction"
REMOTE
}

rollback_runtime() {
  local transaction="$1"
  [[ "$transaction" =~ ^[0-9]{8}T[0-9]{6}Z-runtime-[0-9]+$ ]] \
    || fail "invalid Runtime transaction id"
  ssh_exec "sudo -n sh -s -- '$transaction'" <<'REMOTE'
set -eu
transaction="$1"
state="/var/lib/empiieros-live-transactions/$transaction"
[ -f "$state/empiieros-runtime.before" ]
[ -f "$state/empiieros-runtime.service.before" ]
install -o root -g root -m 0755 \
  "$state/empiieros-runtime.before" /usr/local/bin/empiieros-runtime
install -o root -g root -m 0644 \
  "$state/empiieros-runtime.service.before" /etc/systemd/system/empiieros-runtime.service
systemctl daemon-reload
systemctl stop empiieros-runtime.service || true
systemctl reset-failed empiieros-runtime.service
restored_epoch="$(date +%s)"
systemctl start empiieros-runtime.service
systemctl start empiieros-shell-bridge.socket
nudged=0
started_gui=0
attempt=0
while [ "$attempt" -lt 90 ]; do
  if systemctl is-active --quiet empiieros-runtime.service \
    && systemctl is-active --quiet empiieros-shell-bridge.socket \
    && [ -S /run/empiieros-shell/runtime.sock ] \
    && journalctl -u empiieros-runtime.service --since "@$restored_epoch" --no-pager \
      | awk '/id=shell-.* ok=true status=observed/ { found=1 } END { exit !found }'; then
    break
  fi
  if [ "$nudged" -eq 0 ] && [ "$attempt" -ge 12 ]; then
    pkill -f '^/usr/local/bin/empiieros-shell-gui' || true
    nudged=1
  fi
  if [ "$started_gui" -eq 0 ] && [ "$attempt" -ge 16 ] \
    && ! pgrep -f '^/usr/local/bin/empiieros-shell-gui' >/dev/null; then
    uid="$(id -u eliteew-ts)"
    runtime_dir="/tmp/runtime-${uid}"
    if [ -S "/run/user/${uid}/wayland-0" ]; then
      runtime_dir="/run/user/${uid}"
    fi
    sudo -n -u eliteew-ts env \
      HOME=/home/eliteew-ts \
      USER=eliteew-ts \
      XDG_CONFIG_HOME=/home/eliteew-ts/.config \
      XDG_RUNTIME_DIR="$runtime_dir" \
      WAYLAND_DISPLAY=wayland-0 \
      GDK_BACKEND=wayland,x11 \
      /usr/local/bin/empiieros-shell-gui >/dev/null 2>&1 &
    started_gui=1
  fi
  attempt=$((attempt + 1))
  sleep 1
done
[ "$attempt" -lt 90 ]
printf 'runtime transaction rolled back: %s\n' "$transaction"
REMOTE
}

main() {
  if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    return 0
  fi
  validate_configuration
  case "${1:-}" in
    connect)
      [[ $# -eq 1 ]] || fail "connect takes no arguments"
      ensure_connection
      ;;
    check)
      [[ $# -eq 1 ]] || fail "check takes no arguments"
      ensure_connection
      if [[ "${EMPIIEROS_METAL_NO_CONTROLMASTER:-0}" == 1 ]]; then
        ssh "${ssh_options[@]}" "${USER_NAME}@${HOST}" true
      else
        ssh "${ssh_options[@]}" -O check "${USER_NAME}@${HOST}"
      fi
      ;;
    probe)
      [[ $# -eq 1 ]] || fail "probe takes no arguments"
      ensure_connection
      probe
      ;;
    deploy-runtime)
      [[ $# -eq 3 ]] || fail "deploy-runtime requires BINARY and UNIT"
      ensure_connection
      deploy_runtime "$2" "$3"
      ;;
    deploy-helper)
      [[ $# -eq 3 ]] || fail "deploy-helper requires BINARY and UNIT"
      ensure_connection
      deploy_helper "$2" "$3"
      ;;
    deploy-seat-shield)
      [[ $# -eq 4 ]] || fail "deploy-seat-shield requires FW NFT and UNIT"
      ensure_connection
      deploy_seat_shield "$2" "$3" "$4"
      ;;
    deploy-seat-shield-watch)
      [[ $# -eq 3 ]] || fail "deploy-seat-shield-watch requires BIN and UNIT"
      ensure_connection
      deploy_seat_shield_watch "$2" "$3"
      ;;
    deploy-portal)
      [[ $# -eq 3 ]] || fail "deploy-portal requires BINARY and UNIT"
      ensure_connection
      deploy_portal "$2" "$3"
      ;;
    deploy-spa)
      [[ $# -eq 3 ]] || fail "deploy-spa requires BINARY and UNIT"
      ensure_connection
      deploy_spa "$2" "$3"
      ;;
    deploy-wg)
      [[ $# -eq 3 ]] || fail "deploy-wg requires WG_BIN and MODULE_DIR"
      ensure_connection
      deploy_wg "$2" "$3"
      ;;
    rollback-runtime)
      [[ $# -eq 2 ]] || fail "rollback-runtime requires TRANSACTION"
      ensure_connection
      rollback_runtime "$2"
      ;;
    deploy-shell)
      [[ $# -eq 3 ]] || fail "deploy-shell requires GUI and ASSETS"
      ensure_connection
      deploy_shell "$2" "$3"
      ;;
    rollback-shell)
      [[ $# -eq 2 ]] || fail "rollback-shell requires TRANSACTION"
      ensure_connection
      rollback_shell "$2"
      ;;
    deploy-webkit-gst)
      [[ $# -eq 2 ]] || fail "deploy-webkit-gst requires TREE"
      ensure_connection
      deploy_webkit_gst "$2"
      ;;
    rollback-webkit-gst)
      [[ $# -eq 2 ]] || fail "rollback-webkit-gst requires TRANSACTION"
      ensure_connection
      rollback_webkit_gst "$2"
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
}

main "$@"
