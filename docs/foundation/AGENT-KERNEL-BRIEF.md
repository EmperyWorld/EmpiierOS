# EmpiierOS agent-kernel brief

**Status:** Living secure-reset handoff, 2026-09-18.
**Verdict:** The kernel is a workflow aid, not an acceptance authority.

## Axis

OS Shell (React) + EliteEW Runtime (Rust) + Trusted Substrate (Linux).
Tauri remains living kinship, not the EmpiierOS plan axis.

## Current inventory

- Nine always-applied rules define scope, honesty, change control, and safety.
- Five hook bindings: one fail-closed policy guard in three modes and two
  advisory hooks.
- Thirteen reusable skills and five role-specific agents.
- One glob rule (`comfyui-media`) for local ComfyUI UI media. AlwaysApply stays nine.
- Explicit semantic evidence is run by `scripts/preflight-foundation.sh`.
- Language bridge is disabled. Do not call Gemma intake/outtake.

Counts are descriptive. New assets require a concrete failure mode, bounded
runtime, and selftest; they do not require gaming a numerical quota.

## Hook contract

- `policy_guard.py shell`: blocks hard deletes, destructive Git, and unapproved
  commit/push/init/restore.
- `policy_guard.py tool`: blocks Delete tools and secret-like writes, including
  nested payloads.
- `policy_guard.py read`: blocks secret-like reads while allowing examples and
  public keys.
- Malformed policy input denies.
- Advisory completion and session-root hooks fail open.
- Stop hooks never execute tests, preflight, network calls, or other acceptance.

## Working loop

```text
intake → scope → cause → implement → same-phase docs
       → tests + negative cases → image/QEMU/metal when applicable
       → honest verdict → optional Commit Package
```

## Current scope

Unscoped product expansion is frozen. The locked FoFline V2 wave may continue.
The operator-approved secure reset may change the existing Shell and Runtime
here plus `powered_by_emperyworld/pack-organization/trusted-substrate/`.
Other kinship trees remain out of scope. Never write `connection-agent`.

## Commands

```bash
bash scripts/hooks-selftest.sh
npm ci --prefix shell-ui
bash scripts/preflight-foundation.sh
```
