#!/usr/bin/env bash
# H3: supply-chain report for Runtime + Seat Shield. Not a GPL-fail of the OS floor.
# First wave is a report. cargo-audit is optional until the tool is on the host.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

printf '=== seat_shield_core licenses ===\n'
cargo tree --manifest-path runtime/seat_shield_core/Cargo.toml --prefix none -e normal --locked || {
  echo "FAIL  cargo tree seat_shield_core" >&2
  exit 1
}

printf '=== runtime crate licenses (informational; floor GTK/nft are not MIT) ===\n'
cargo tree --manifest-path runtime/Cargo.toml --prefix none -e normal --locked | head -n 80 || true

if ! command -v cargo-audit >/dev/null 2>&1 && ! cargo audit -V >/dev/null 2>&1; then
  echo "SKIP  cargo-audit not installed (H3 first report; not a gate fail)"
  exit 0
fi

printf '=== cargo audit runtime lock ===\n'
# cargo-audit 0.22 audits a lockfile (`--file`), not a manifest path.
cargo audit --file runtime/Cargo.lock --deny unsound --deny yanked
printf '=== cargo audit seat_shield_core lock ===\n'
cargo audit --file runtime/seat_shield_core/Cargo.lock --deny unsound --deny yanked
