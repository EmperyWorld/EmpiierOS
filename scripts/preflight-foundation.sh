#!/usr/bin/env bash
# EmpiierOS semantic preflight. Exit zero only when executable evidence is green.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# Kinship Trusted Substrate: env override, else sibling of this repo.
: "${ELITEEW_TS_ROOT:=$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}"
if [[ -d "$ELITEEW_TS_ROOT" ]]; then
  ELITEEW_TS_ROOT="$(cd "$ELITEEW_TS_ROOT" && pwd)"
fi
export ELITEEW_TS_ROOT

FAILURES=0

pass() {
  printf 'PASS  %s\n' "$*"
}

fail() {
  printf 'FAIL  %s\n' "$*"
  FAILURES=$((FAILURES + 1))
}

require_file() {
  local path="$1"
  if [[ -s "$path" ]]; then
    pass "$path present"
  else
    fail "$path missing or empty"
  fi
}

require_executable() {
  local path="$1"
  if [[ -x "$path" ]]; then
    pass "$path executable"
  else
    fail "$path is not executable"
  fi
}

run_check() {
  local label="$1"
  shift
  if "$@"; then
    pass "$label"
  else
    fail "$label"
  fi
}

printf 'EmpiierOS semantic preflight\n'

for path in \
  docs/NORTH-STAR.md \
  docs/WHERE-OS-LIVES.md \
  docs/FOUNDATION-STATUS.md \
  docs/plans/MATURITY-LEDGER.md \
  runtime/Cargo.toml \
  runtime/src/core.rs \
  runtime/src/daemon.rs \
  runtime/src/protocol.rs \
  shell-ui/package.json \
  shell-ui/package-lock.json \
  shell-ui/src/secure/SecureApp.tsx \
  .cursor/hooks.json \
  .cursor/hooks/policy_guard.py
do
  require_file "$path"
done

for path in \
  scripts/preflight-foundation.sh \
  scripts/hooks-selftest.sh \
  scripts/runtime-health-smoke.sh \
  scripts/bootstrap-phase-smoke.sh \
  scripts/package-shell-gui-rootfs.sh \
  scripts/metal-live.sh
do
  require_executable "$path"
done

run_check "living canon is secure-reset honest" python3 - <<'PY'
from pathlib import Path

north = Path("docs/NORTH-STAR.md").read_text(encoding="utf-8")
status = Path("docs/FOUNDATION-STATUS.md").read_text(encoding="utf-8")
plans = Path("docs/plans/README.md").read_text(encoding="utf-8")
ledger = Path("docs/plans/MATURITY-LEDGER.md").read_text(encoding="utf-8")

assert "NOT_READY" in north
assert "NOT_READY" in status
assert "historical" in plans.lower()
assert "Supersedes" in ledger
assert "completed and verified" not in status.lower()
PY

run_check "packager records dirty state and its own source" python3 - <<'PY'
from pathlib import Path

packager = Path("scripts/package-shell-gui-rootfs.sh").read_text(encoding="utf-8")
assert 'root / "scripts/package-shell-gui-rootfs.sh"' in packager
assert '"source_tree_dirty": bool(' in packager
assert "EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER" in packager
PY

run_check "metal harness parses cleanly" bash -n scripts/metal-live.sh
if command -v shellcheck >/dev/null 2>&1; then
  run_check "metal harness has no shellcheck findings" shellcheck scripts/metal-live.sh
else
  fail "shellcheck is required to verify scripts/metal-live.sh"
fi
run_check "metal harness is pinned and transactional" python3 - <<'PY'
from pathlib import Path
import os

harness = Path("scripts/metal-live.sh").read_text(encoding="utf-8")
assert "EMPIIEROS_METAL_HOSTKEY_SHA256 is required" in harness
assert "StrictHostKeyChecking=yes" in harness
assert "StrictHostKeyChecking=no" not in harness
assert "ServerAliveInterval=15" in harness
assert "authorized Shell snapshot was not observed; rolling back" in harness
assert "rollback-runtime" in harness
assert "deploy-shell" in harness
assert "deploy-seat-shield" in harness
assert "deploy-seat-shield-watch" in harness
assert "empiieros-seat-shield-fw" in harness
assert "deploy-webkit-gst" in harness
assert "rollback-webkit-gst" in harness
assert "libgstwebrtc.so" in harness
assert "libgstrtpmanager.so" in harness
assert "libgstrtp.so" in harness
assert "libgstdtls.so" in harness
assert "bound on-peer-certificate after create_connection" in harness
assert "libgstvideorate.so" in harness
assert "libgstvideotestsrc.so" in harness
assert "libgstaudiotestsrc.so" in harness
assert "libgstinterleave.so" in harness
assert "rtpfunnel string missing from libgstrtpmanager.so" in harness
assert "rtph264pay string missing from libgstrtp.so" in harness
assert "videorate string missing from libgstvideorate.so" in harness
assert "videotestsrc string missing from libgstvideotestsrc.so" in harness
assert "audiotestsrc string missing from libgstaudiotestsrc.so" in harness
assert "deinterleave string missing from libgstinterleave.so" in harness
assert "sudo -n -u eliteew-ts touch /tmp/empiieros-meet-gst.log" in harness
assert ": >/tmp/empiieros-meet-gst.log" not in harness
assert "Enable RTCPeerConnection" in harness
assert "ensureDtlsCertificatePem" in harness
assert "DTLS PEM lock missing from WebKit" in harness
assert "Applied DTLS PEM" in harness
assert "WebKit/GST so.0 must resolve to a regular library" in harness
assert "libwebkit2gtk-4.1.so.0.16.6" not in harness
assert "pgrep -f '^/usr/local/bin/empiieros-shell-gui'" in harness
kiosk = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
    "/payload-templates/usr-local-bin/eliteew-ts-kiosk"
).read_text(encoding="utf-8")
assert "GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log" in kiosk
assert "GST_DEBUG=2,webrtcbin:5,webkitwebrtc*:6,GST_ELEMENT_FACTORY:4,dtls*:5" in kiosk
kiosk_overlay = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
    "/config/buildroot/external/board/eliteew-ts/rootfs_overlay"
    "/usr/local/bin/eliteew-ts-kiosk"
).read_text(encoding="utf-8")
assert "GST_DEBUG_FILE=/tmp/empiieros-meet-gst.log" in kiosk_overlay
assert "GST_DEBUG=2,webrtcbin:5,webkitwebrtc*:6,GST_ELEMENT_FACTORY:4,dtls*:5" in kiosk_overlay
assert "pgrep -x empiieros-shell-gui" not in harness
ts = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
)
dtls = (
    ts
    / "config/buildroot/patches/package-dropins/webkitgtk"
    / "0002-webrtc-dtls-pem-before-sdp.patch"
).read_text(encoding="utf-8")
assert "Propagated DTLS PEM to webrtcbin" in dtls
assert "Applied DTLS PEM on" in dtls
assert "dtlsApplyPemAndAccept" in dtls
assert "dtlssrtpenc" in dtls
assert "dtlsAcceptPeerCertificate" in dtls
assert "X509_digest" in dtls
assert "sha-256" in dtls
assert "custom certificates not propagated to webrtcbin" in dtls
assert "2.54" not in dtls
assert "0002-webrtc-dtls-pem-before-sdp" in (
    ts / "config/buildroot/external/external.mk"
).read_text(encoding="utf-8")
gst_dtls = (
    ts
    / "config/buildroot/patches/package-dropins/gstreamer1/gst1-plugins-bad"
    / "0001-dtlsdec-rebind-peer-cert-after-pem.patch"
).read_text(encoding="utf-8")
assert "bound on-peer-certificate after create_connection" in gst_dtls
assert "on-peer-certificate" in gst_dtls
assert "2.54" not in gst_dtls
assert "0001-dtlsdec-rebind-peer-cert-after-pem" in (
    ts / "config/buildroot/external/external.mk"
).read_text(encoding="utf-8")
gst_dtls_adv = gst_dtls.replace(
    "bound on-peer-certificate after create_connection", ""
)
assert "bound on-peer-certificate after create_connection" not in gst_dtls_adv
dtls_adv = dtls.replace("Propagated DTLS PEM to webrtcbin", "")
assert "Propagated DTLS PEM to webrtcbin" not in dtls_adv
assert "lock_labwc_kiosk_close" in harness
assert (
    's#command="pkill -f /usr/local/bin/empiieros-shell-gui"#command="true"#g'
    in harness
)
assert "LC_ALL=C sort -z" in harness
assert "git commit" not in harness
assert "/dev/sd" not in harness
PY

run_check "hook kernel selftest" bash scripts/hooks-selftest.sh

run_check "no Runtime or Shell source outside the build graph" python3 - <<'PY'
import re
from pathlib import Path

rt = Path("runtime/src")
rt_roots = [rt / "core.rs", rt / "daemon.rs", *sorted((rt / "bin").glob("*.rs"))]
rt_seen: set[Path] = set()


def walk_rs(path: Path) -> None:
    if path in rt_seen:
        return
    rt_seen.add(path)
    text = path.read_text(encoding="utf-8")
    assert "#[path" not in text, path
    for match in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", text, re.M):
        name = match.group(1)
        for child in (path.parent / f"{name}.rs", path.parent / name / "mod.rs"):
            if child.is_file():
                walk_rs(child)
                break
        else:
            raise AssertionError(f"{path}: mod {name} has no file")


for root in rt_roots:
    walk_rs(root)
rt_dead = sorted(set(rt.rglob("*.rs")) - rt_seen)
assert not rt_dead, f"uncompiled Runtime sources: {rt_dead}"

sh = Path("shell-ui/src")
sh_roots = [sh / "main.tsx", *sorted(sh.rglob("*.check.ts"))]
sh_seen: set[Path] = set()
import_re = re.compile(r"""(?:from|import)\s*\(?\s*["'](\.[^"']+)["']""")


def resolve_ts(base: Path, spec: str) -> Path:
    target = (base.parent / spec).resolve().relative_to(Path.cwd())
    for cand in (target, *(target.with_name(target.name + ext) for ext in (".ts", ".tsx")),
                 target / "index.ts", target / "index.tsx"):
        if cand.is_file():
            return cand
    raise AssertionError(f"{base}: unresolved import {spec}")


def walk_ts(path: Path) -> None:
    if path in sh_seen:
        return
    sh_seen.add(path)
    if path.suffix == ".css":
        for match in re.finditer(r"""@import\s+["'](\.[^"']+)["']""", path.read_text(encoding="utf-8")):
            walk_ts(resolve_ts(path, match.group(1)))
        return
    for match in import_re.finditer(path.read_text(encoding="utf-8")):
        walk_ts(resolve_ts(path, match.group(1)))


for root in sh_roots:
    walk_ts(root)
sh_all = {
    p for p in sh.rglob("*")
    if p.suffix in {".ts", ".tsx", ".css"} and p.name != "vite-env.d.ts"
}
sh_dead = sorted(sh_all - sh_seen)
assert not sh_dead, f"Shell sources outside the import graph: {sh_dead}"
PY

run_check "production source needles" python3 scripts/preflight-needle-runner.py scripts/preflight-needles.py

run_check "shield overview does not repeat the retired lead" python3 - <<'PY'
from pathlib import Path
card = Path("shell-ui/src/secure/components/SeatShieldCard.tsx").read_text(encoding="utf-8")
if "seatShieldOverviewLead" in card:
    raise SystemExit("seatShieldOverviewLead is back on the shield card")
PY

run_check "time.sync capability follows the operator clock helper" python3 - <<'PY'
from pathlib import Path
gate = Path("runtime/src/capability_gate.rs").read_text(encoding="utf-8")
probe = Path("runtime/src/system_probe.rs").read_text(encoding="utf-8")
if "operator-wall-clock" not in gate:
    raise SystemExit("time.sync source is not operator-wall-clock")
banned = ('source: "systemd-timesyncd"', "timesyncd_not_observed")
if any(token in gate or token in probe for token in banned):
    raise SystemExit("time.sync still treats timesyncd as the clock gate")
PY


run_check \
  "runtime tests" \
  cargo test --locked --manifest-path runtime/Cargo.toml
run_check \
  "runtime check with production defaults" \
  cargo check --locked --all-targets --manifest-path runtime/Cargo.toml
run_check "runtime IPC smoke" bash scripts/runtime-health-smoke.sh

if [[ -d shell-ui/node_modules ]]; then
  run_check "Shell typecheck and production build" npm --prefix shell-ui run build
  run_check "kiosk CSP stays on disk; Vite rewrites only in memory" python3 - <<'PY'
from pathlib import Path

kiosk = "script-src empiier:"
source = Path("shell-ui/index.html").read_text(encoding="utf-8")
built = Path("shell-ui/dist/index.html").read_text(encoding="utf-8")
vite = Path("shell-ui/vite.config.ts").read_text(encoding="utf-8")
assert kiosk in source, "source index.html lost kiosk CSP"
assert kiosk in built, "production dist/index.html lost kiosk CSP"
assert "unsafe-eval" not in built
assert "unsafe-inline" not in built
assert "if (!ctx.server)" in vite
assert "LOCAL_DEV_CSP" in vite
assert "LOCAL_DEV_CSP" not in source
PY
else
  fail "shell-ui/node_modules absent; run npm ci --prefix shell-ui before preflight"
fi

run_check "Shell bootstrap/transport smoke" bash scripts/bootstrap-phase-smoke.sh

run_check "no generated dependency/build output is tracked" python3 - <<'PY'
import subprocess

tracked = subprocess.run(
    ["git", "ls-files", "-z"],
    check=True,
    capture_output=True,
).stdout.decode("utf-8").split("\0")
bad = [
    path
    for path in tracked
    if path
    and (
        "/node_modules/" in f"/{path}/"
        or path.startswith("shell-ui/dist/")
        or path.startswith("runtime/target/")
    )
]
assert not bad, bad
PY

run_check "tracked diff has no whitespace errors" git diff --check

if [[ "$FAILURES" -ne 0 ]]; then
  printf 'RESULT: FAIL (failures=%s)\n' "$FAILURES"
  exit 1
fi

printf 'RESULT: PASS\n'
