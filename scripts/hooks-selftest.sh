#!/usr/bin/env bash
# Semantic selftest for the small EmpiierOS hook kernel.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FAILURES=0

pass() {
  printf 'PASS  %s\n' "$*"
}

fail() {
  printf 'FAIL  %s\n' "$*"
  FAILURES=$((FAILURES + 1))
}

permission() {
  python3 -c 'import json,sys; print(json.load(sys.stdin).get("permission", ""))'
}

guard_result() {
  local mode="$1"
  local payload="$2"
  printf '%s' "$payload" | python3 .cursor/hooks/policy_guard.py "$mode" | permission
}

expect_permission() {
  local label="$1"
  local mode="$2"
  local payload="$3"
  local expected="$4"
  local observed
  observed="$(guard_result "$mode" "$payload")"
  if [[ "$observed" == "$expected" ]]; then
    pass "$label -> $expected"
  else
    fail "$label -> expected $expected, observed ${observed:-<empty>}"
  fi
}

if python3 - <<'PY'
import json
from pathlib import Path

config = json.loads(Path(".cursor/hooks.json").read_text(encoding="utf-8"))
hooks = config["hooks"]
bindings = [binding for values in hooks.values() for binding in values]
assert len(bindings) == 5, len(bindings)

required = {
    "python3 .cursor/hooks/policy_guard.py shell",
    "python3 .cursor/hooks/policy_guard.py tool",
    "python3 .cursor/hooks/policy_guard.py read",
    "python3 .cursor/hooks/session-root-hygiene.py",
    "python3 .cursor/hooks/foundation-completion-hint.py",
}
assert {binding["command"] for binding in bindings} == required

for binding in bindings:
    command = binding["command"]
    if "policy_guard.py" in command:
        assert binding.get("failClosed") is True
        assert binding.get("timeout", 999) <= 5
PY
then
  pass "hooks.json has five bounded bindings"
else
  fail "hooks.json structure or policy is invalid"
fi

for path in \
  .cursor/hooks/policy_guard.py \
  .cursor/hooks/session-root-hygiene.py \
  .cursor/hooks/foundation-completion-hint.py
do
  if [[ -s "$path" ]]; then
    pass "$path present"
  else
    fail "$path missing or empty"
  fi
done

expect_permission \
  "read ordinary file" \
  read \
  '{"path":"docs/NORTH-STAR.md"}' \
  allow
expect_permission \
  "read public-key example" \
  read \
  '{"arguments":{"path":"fixtures/keys/device.pub"}}' \
  allow
expect_permission \
  "deny nested private key read" \
  read \
  '{"arguments":{"path":"fixtures/keys/device.key"}}' \
  deny
expect_permission \
  "deny environment write" \
  tool \
  '{"tool_name":"Write","arguments":{"path":".env.production"}}' \
  deny
expect_permission \
  "allow environment example write" \
  tool \
  '{"tool_name":"Write","arguments":{"path":".env.example"}}' \
  allow
expect_permission \
  "deny Delete tool" \
  tool \
  '{"tool_name":"Delete","arguments":{"path":"docs/NORTH-STAR.md"}}' \
  deny
expect_permission \
  "allow read-only git" \
  shell \
  '{"command":"git status --short"}' \
  allow
expect_permission \
  "deny chained hard delete" \
  shell \
  '{"command":"printf ok; /usr/bin/rm -r build"}' \
  deny
expect_permission \
  "deny hard reset" \
  shell \
  '{"command":"git reset --hard HEAD"}' \
  deny
expect_permission \
  "deny unapproved commit" \
  shell \
  '{"command":"git commit -m audit"}' \
  deny
expect_permission \
  "deny approval marker outside prefix" \
  shell \
  '{"command":"printf EMPIIEROS_OPERATOR_APPROVED=1; git commit -m audit"}' \
  deny
expect_permission \
  "allow explicitly approved commit" \
  shell \
  '{"command":"EMPIIEROS_OPERATOR_APPROVED=1 git commit -m audit"}' \
  allow
expect_permission \
  "deny malformed JSON" \
  shell \
  '{' \
  deny
expect_permission \
  "deny missing command" \
  shell \
  '{}' \
  deny

if [[ "$FAILURES" -ne 0 ]]; then
  printf 'RESULT: FAIL (failures=%s)\n' "$FAILURES"
  exit 1
fi

printf 'RESULT: PASS\n'
