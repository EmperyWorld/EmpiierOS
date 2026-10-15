# Session bootstrap phase (v0)

**Status:** Contract stub + Shell helpers 2026-09-13. **Not** full launcher Journey. **Not** Tauri.
**Plan:** historical wave in `trash/`. Living status: [`../../docs/plans/MATURITY-LEDGER.md`](../../docs/plans/MATURITY-LEDGER.md).
**Kinship HARVEST:** Identifying-until-resolved (`docs/capture/13`, lesson #40).

## Law

1. **Identifying until resolved** — do not show enroll / activation / privileged chrome while boot signals are incomplete.
2. Ping/health alone is **not** enough — wait for `seat.status` (and later entitlement) to finish.
3. Shell **displays** phase; Runtime owns privileged truth (`seat.status`, future bind).
4. Honesty: phase `unbound` ≠ Start OK; `boot_error` is explicit.

## v0 phases

| Phase | User sees | Exit |
|-------|-----------|------|
| `identifying` | Brand + busy honesty | `system` or `boot_error` |
| `system` | OS Shell chrome + Health/Seat panels | later: enroll when tasked |
| `boot_error` | Honest failure + retry hint | re-run IPC |

## Signals (v0)

| Signal | Source |
|--------|--------|
| `healthDone` / `healthOk` | Runtime `GET /v0/health` |
| `seatDone` / `seatOk` | Runtime `GET /v0/seat/status` |
| `bootFailed` | Both unreachable or hard client failure |

## Helpers

Pure TS: `shell-ui/src/session/bootstrapPhase.ts` (ORIGIN lists kinship `journeyPhase.ts`).

- `mayEnableStartCta(present, verified)` — Start only when **both** true (lab-verify may set `verified:true`; still not channel / not product Done).
- `mayShowWelcomeSurface(phase, showWelcome)` — welcome chrome only on `system` when policy `show_welcome`.
