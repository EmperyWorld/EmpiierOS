# seat_shield_core

EmpiierOS Seat Shield logic crate (vendored under `runtime/`).

- Contract: `session/SEAT-SHIELD.md`
- Floor helper protocol: `session/SEAT-SHIELD-FW.md`

Forbidden: `unsafe`, shell-string nft, Flask dual control plane, silent fail-open acts.

`seat.shield.status` calls `maybe_auto_observe` when `cycle_interval_secs` has elapsed. Runtime opens a new service per IPC line; the interval is stored in `cycle_state.json`. Product defaults are armed (`act_enabled=true`, `auto_act_protected=true`, `auto_quarantine_hash=true`); sparse legacy policy JSON migrates once on read. Protected auto-act applies `BlockPeer` only. Behavioral / hash / tmp-exec / `kernel_taint` never auto-act. S5A on-demand scan/hold is tmp/shm only; auto-hold only after `ioc_hash_match` + `ENABLE_QUARANTINE` (cap 1). Not on-access AV.

```bash
cargo test --manifest-path runtime/seat_shield_core/Cargo.toml
cargo test --manifest-path runtime/Cargo.toml --lib seat_shield
```
