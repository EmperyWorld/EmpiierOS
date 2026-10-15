# Non-goals and guards

**Capture date:** 2026-09-11

## Non-goals (foundation phase)

| Non-goal | Why |
|----------|-----|
| Break / refactor living Tauri "for the OS" | Parallel line first |
| Remove Tauri now | Living carrier; history later |
| Plan capabilities as Tauri plugins/commands | Wrong axis |
| Clone the entire `shared/web/frontend` | Seed is narrow; otherwise, it becomes a second SPA monorepo |
| Wholesale clone `connection-agent` | We would pull in host dependencies |
| Second stack0 / Keycloak / cloud topology | Kinship only |
| Admin-OS SKU in foundation | Deferred; after shell/runtime spine |
| Break Trusted Substrate kiosk profile | Profile coexistence later |
| Treat markdown-only branching as "foundation" | Requires ignores, kernel, and preflight |
| Rush product UI coding in F0–F5 | Foundation first |
| Extract shared packages "immediately to avoid duplication" | Conscious duplication until OS Shell contract is stable (`01`) |
| Port WG/license/Capsule to v0 OS host | Will become a second Tauri; stubs + Runtime later |
| Port the entire EliteEW `.cursor/` to the OS repo | Gate fatigue + drowning |
| Clone the entire genEW `.cursor/` / Hands / recipes / U–D gate zoo | Harvest mechanisms only (`09`); otherwise, a second Empery factory |
| Port the entire monorepo `.cursor/` (~85 rules / ~61 hooks) to OS | Gate fatigue; study via `10`, KEEP management core only |
| "Clean up" monorepo/genEW during git forensic | Forensic = read-only; edits only in EmpiierOS capture |
| Restore / `cp -a` entire DEWA, EWPower, EWGen, ... into EmpiierOS or monorepo root | Soft-retire parks = study/harvest only; second-universe ban (`12`) |
| Plan OS capabilities as Tauri plugins/commands "since we are studying the launcher" | Study ≠ plan axis (`02`, `13`); harvest → Runtime/Shell rewrite |

## Guards (behavioral)

1. One agent session — one workspace root (OS **or** monorepo **or** genEW), except for explicit kinship tasks. **Locked:** work home = this repo; monorepo = read-only kinship.
2. No product coding until foundation preflight is green.
3. No in-place "preparation" of living files for the future OS.
4. Any copy from the monorepo → `ORIGIN.md` (once we reach the seed stage).
5. Dual-flag: kiosk/Tauri payload vs OS Shell payload = fail-closed, once implemented.
6. This capture folder is not to be presented as a locked SoT without an operator lock. (**Partial lock 2026-09-11:** intent axis in `00` confirmed; full product SoT still later.)
7. genEW / monorepo agent patterns — **rewrite** into OS mini-kernel; no wholesale paste.
8. F1 monorepo study = **read-only**; do not edit living hooks/rules in the monorepo from the EmpiierOS chat.
9. F1.7 git forensic = **read-only** `git log`/`show` only; findings only in EmpiierOS (`11` / `05`); no commit/push/rebase/amend/reset in kinship repos.
10. F1.8–F1.9 trash/parked study = **read-only**; capability harvest mindset; **no** whole-tree restore of DEWA/EWPower/... without an explicit operator restore task.
11. F1.10–F1.11 security/launcher study = **read-only**; do not break living Tauri; do not make Tauri the normative OS plan axis.
12. F1.12 open findings = append to EmpiierOS capture; no product coding from opportunistic finds without a wave start.
13. Separate Linux user — **optional max** on F2; **not** a blocker for continuing foundation in EmpiierOS under the current user.

## Allowed now

- Append files to this repo (this capture)
- Recall lessons, break down foundation waves
- View (read-only) monorepo **and** `../genEW` rules/hooks/skills/agents/git for harvesting
- Discuss repo/user/dir names without creating them, until the operator says "start F2"
- **Do not** implement waves / seed React until the operator starts a wave or approves preflight
