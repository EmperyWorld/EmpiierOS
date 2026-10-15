# Seat Shield H3 audit report

**Date:** 2026-09-20
**Status:** Report. Not `READY`. Not a GPL-fail of WebKit/GTK/nft.
**Tool:** `cargo-audit` 0.22.2 on this workstation.

## Measure

```text
PATH="$HOME/.cargo/bin:$PATH" bash scripts/runtime-cargo-audit.sh
```

Exit **0**. `--deny unsound --deny yanked` on:

- `runtime/Cargo.lock` (161 crate dependencies)
- `runtime/seat_shield_core/Cargo.lock` (35 crate dependencies)

No ignore list. Advisory DB 1251 entries (`~/.cargo/advisory-db`).

Shield tree (`cargo tree`): `serde` / `serde_json` / `sha2` / `thiserror` / `inotify` (+ `libc` via inotify-sys). MIT/Apache. Floor GTK/nft stay copyleft.

## Hosts without the tool

`scripts/runtime-cargo-audit.sh` still **SKIP**s when `cargo-audit` is absent. That is not a gate fail. Preflight only checks the script and `--deny unsound` / lockfile flags.

## Out of scope

`cargo geiger`, clippy pedantic deny-all, `cargo vendor`, pqcrypto, deleting a living crate to fake green.
