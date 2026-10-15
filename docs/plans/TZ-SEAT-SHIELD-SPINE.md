# TZ — Seat Shield spine (journal · connections · heuristics)

**Status:** Binding work order. **P1 + P2 + P3 local Measure green**. **Not** MATCH. **Not** acceptance. Official bake / metal mux not claimed.
**Date:** 2026-09-21
**Living contract:** [`../../session/SEAT-SHIELD.md`](../../session/SEAT-SHIELD.md)
**Companion:** [`TZ-SEAT-SHIELD-POST-S4.md`](TZ-SEAT-SHIELD-POST-S4.md) · [`TZ-SEAT-SHIELD-HARDENING.md`](TZ-SEAT-SHIELD-HARDENING.md) · [`TZ-SEAT-SHIELD-H6-PRIVATETMP.md`](TZ-SEAT-SHIELD-H6-PRIVATETMP.md)

## Law

- `events.json` = decision/status SoT (ring 200).
- `journal.jsonl` = durable browse copy (cap 1000 lines / 1 MiB).
- Heuristics run inside Runtime observe cycle only. Kill-switch: `heuristics_enabled` (no `ENABLE_ACTS`).
- Chains / burst are **observe-only**. Auto BlockPeer / auto-hold gates unchanged.
- No Python TUI, AF_PACKET, eBPF in this TZ. No MATCH claims.
- inet_diag probe (P3) ≠ AF_PACKET.

## Phases

| Phase | Outcome |
|-------|---------|
| **P0** | This file + plans index; preflight green before P1 claim |
| **P1** | Full-width tabs; Соединения; Журнал + `seat.shield.journal`; observe `connections[]`; `heuristics_enabled` stub — **local Measure 2026-09-21** |
| **P2** | peer_window + risk_milli + `chain_*` behind kill-switch; Shell Journal `group_by=chain` — **local Measure 2026-09-21** |
| **P3** | `seat-shield-probe` inet_diag dump + TTL fail→`/proc` + RTT — **local Measure 2026-09-21** |

## P3 Measure (local)

- Crate: `parse_probe_snap` + service fake-probe RTT backend + timeout→`proc_net_tcp`
- `cargo build --manifest-path runtime/seat_shield_probe/Cargo.toml`
- `scripts/preflight-foundation.sh` → exit 0
- Packager wires probe into overlay; metal MATCH still requires Substrate bake + plant evidence

## Operator phrases

P0–P2:

> Unlock EmpiierOS write for Seat Shield spine P1–P2 per `docs/plans/TZ-SEAT-SHIELD-SPINE.md`.

P3:

> Unlock EmpiierOS write for Seat Shield probe helper (ewonika inet_diag dump). No eBPF. No AF_PACKET.
