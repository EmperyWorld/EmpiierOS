#!/usr/bin/env bash
# Runtime Unix-socket framing, authorization, and liveness smoke.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TMP="$(mktemp -d -t empiieros-runtime-smoke.XXXXXX)"
PID=""
PROXY_PID=""

cleanup() {
  if [[ -n "$PROXY_PID" ]] && kill -0 "$PROXY_PID" 2>/dev/null; then
    kill "$PROXY_PID" 2>/dev/null || true
    wait "$PROXY_PID" 2>/dev/null || true
  fi
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  chmod -R u+w "$TMP" 2>/dev/null || true
  /usr/bin/rm -rf -- "$TMP"
}
trap cleanup EXIT INT TERM

mkdir -p "$TMP/strict-run" "$TMP/proxy-run" "$TMP/dev-run"
chmod 700 "$TMP" "$TMP/strict-run" "$TMP/proxy-run" "$TMP/dev-run"

cargo build \
  --locked \
  --manifest-path runtime/Cargo.toml \
  --no-default-features \
  --features dev-peer-bypass

BINARY="$ROOT/runtime/target/debug/empiieros-runtime"

wait_for_socket() {
  local socket="$1"
  local attempt
  for ((attempt = 0; attempt < 100; attempt++)); do
    if [[ -S "$socket" ]]; then
      return 0
    fi
    if ! kill -0 "$PID" 2>/dev/null; then
      return 1
    fi
    sleep 0.05
  done
  return 1
}

stop_runtime() {
  kill "$PID"
  wait "$PID" 2>/dev/null || true
  PID=""
}

STRICT_SOCKET="$TMP/strict-run/runtime.sock"
EMPIIEROS_RUNTIME_SOCKET="$STRICT_SOCKET" \
EMPIIEROS_RUNTIME_STATE_DIR="$TMP/strict-state" \
EMPIIEROS_ALLOWED_BRIDGE_UID="$(id -u)" \
EMPIIEROS_ALLOWED_BRIDGE_EXE="$BINARY" \
  "$BINARY" >"$TMP/strict.log" 2>&1 &
PID="$!"
wait_for_socket "$STRICT_SOCKET"

python3 - "$STRICT_SOCKET" <<'PY'
import json
import socket
import sys

with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
    client.settimeout(3)
    client.connect(sys.argv[1])
    client.sendall(b'{"id":"deny-1","op":"runtime.health","params":{}}\n')
    response = json.loads(client.makefile("rb").readline())

assert response["ok"] is False, response
assert response["status"] == "denied", response
assert response["error"]["code"] == "peer_executable_mismatch", response
PY
stop_runtime

SYSTEMD_SOCKET_ACTIVATE="$(command -v systemd-socket-activate)"
SYSTEMD_SOCKET_PROXY="/usr/lib/systemd/systemd-socket-proxyd"
[[ -x "$SYSTEMD_SOCKET_ACTIVATE" && -x "$SYSTEMD_SOCKET_PROXY" ]]
cp "$SYSTEMD_SOCKET_PROXY" "$TMP/systemd-socket-proxyd"
chmod 0755 "$TMP/systemd-socket-proxyd"

PROXY_BACKEND="$TMP/proxy-run/backend.sock"
PROXY_FRONTEND="$TMP/proxy-run/frontend.sock"
EMPIIEROS_RUNTIME_SOCKET="$PROXY_BACKEND" \
EMPIIEROS_RUNTIME_STATE_DIR="$TMP/proxy-state" \
EMPIIEROS_ALLOWED_BRIDGE_UID="$(id -u)" \
EMPIIEROS_ALLOWED_BRIDGE_EXE="$TMP/systemd-socket-proxyd" \
  "$BINARY" >"$TMP/proxy-runtime.log" 2>&1 &
PID="$!"
wait_for_socket "$PROXY_BACKEND"

"$SYSTEMD_SOCKET_ACTIVATE" \
  --listen="$PROXY_FRONTEND" \
  "$TMP/systemd-socket-proxyd" "$PROXY_BACKEND" \
  >"$TMP/proxy.log" 2>&1 &
PROXY_PID="$!"
wait_for_socket "$PROXY_FRONTEND"

python3 - "$PROXY_FRONTEND" <<'PY'
import json
import socket
import sys

with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
    client.settimeout(5)
    client.connect(sys.argv[1])
    client.sendall(b'{"id":"proxy-1","op":"runtime.health","params":{}}\n')
    response = json.loads(client.makefile("rb").readline())

assert response["ok"] is True, response
assert response["status"] == "ok", response
assert response["data"]["transport"] == "unix-peercred-v1", response
PY
kill "$PROXY_PID"
wait "$PROXY_PID" 2>/dev/null || true
PROXY_PID=""
stop_runtime

DEV_SOCKET="$TMP/dev-run/runtime.sock"
EMPIIEROS_RUNTIME_SOCKET="$DEV_SOCKET" \
EMPIIEROS_RUNTIME_STATE_DIR="$TMP/dev-state" \
EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER=1 \
  "$BINARY" >"$TMP/dev.log" 2>&1 &
PID="$!"
wait_for_socket "$DEV_SOCKET"

if EMPIIEROS_RUNTIME_SOCKET="$DEV_SOCKET" \
  EMPIIEROS_RUNTIME_STATE_DIR="$TMP/second-state" \
  EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER=1 \
  "$BINARY" >"$TMP/second.log" 2>&1
then
  printf 'FAIL  second daemon replaced active socket\n' >&2
  exit 1
fi

python3 - "$DEV_SOCKET" <<'PY'
import json
import socket
import sys

SOCKET_PATH = sys.argv[1]


def request(payload: bytes, *, shutdown: bool = False) -> dict[str, object]:
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.settimeout(5)
        client.connect(SOCKET_PATH)
        client.sendall(payload)
        if shutdown:
            client.shutdown(socket.SHUT_WR)
        line = client.makefile("rb").readline(300_000)
    assert line.endswith(b"\n"), line[-80:]
    return json.loads(line)


health = request(b'{"id":"health-1","op":"runtime.health","params":{}}\n')
assert health["ok"] is True, health
assert health["status"] == "ok", health
assert health["data"]["transport"] == "unix-peercred-v1", health

snapshot = request(b'{"id":"snapshot-1","op":"system.snapshot","params":{}}\n')
assert snapshot["ok"] is True, snapshot
assert snapshot["status"] == "observed", snapshot
assert snapshot["data"]["security"]["status"] in {"verified", "degraded"}, snapshot
assert snapshot["data"]["sequence"] >= 1, snapshot
assert snapshot["data"]["provenance"]["collector"] == "runtime-system-probe", snapshot
events = request(b'{"id":"events-1","op":"system.events","params":{"after_sequence":0}}\n')
assert events["ok"] is True, events
assert events["data"]["latest_sequence"] >= snapshot["data"]["sequence"], events
assert isinstance(events["data"]["events"], list), events

unknown = request(b'{"id":"unknown-1","op":"not.registered","params":{}}\n')
assert unknown["ok"] is False, unknown
assert unknown["status"] == "not_found", unknown

disabled = request(
    b'{"id":"disabled-1","op":"security.integrity.run","params":{}}\n'
)
assert disabled["ok"] is False, disabled
assert disabled["status"] == "not_available", disabled

framing = request(
    b'{"id":"frame-1","op":"runtime.health","params":{}}',
    shutdown=True,
)
assert framing["ok"] is False, framing
assert framing["error"]["code"] == "newline_terminated_request_required", framing

oversized = request(b"x" * (64 * 1024 + 1) + b"\n")
assert oversized["ok"] is False, oversized
assert oversized["status"] == "request_too_large", oversized

held = []
try:
    for _ in range(16):
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.settimeout(3)
        client.connect(SOCKET_PATH)
        held.append(client)
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as overflow:
        overflow.settimeout(3)
        overflow.connect(SOCKET_PATH)
        overflow.sendall(b'{"id":"overflow","op":"runtime.health","params":{}}\n')
        busy = json.loads(overflow.makefile("rb").readline())
    assert busy["ok"] is False, busy
    assert busy["status"] == "busy", busy
    assert busy["error"]["code"] == "connection_limit", busy
finally:
    for client in held:
        client.close()
PY

printf 'PASS  runtime-health-smoke (peer deny, socket proxy, active-socket, framing, connection bound)\n'
