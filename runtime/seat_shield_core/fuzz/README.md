# H4 parser fuzz (optional nightly)

The OS measure is stable:

```text
CARGO_TARGET_DIR=runtime/target \
  cargo test --manifest-path runtime/seat_shield_core/Cargo.toml --test parser_fuzz
```

`cargo fuzz` / libFuzzer is optional host tooling. Do not require 24h in
preflight. Do not add etherparse. Targets would call
`seat_shield_core::parser_probe` only (`nft_text`, `nft_json`,
`proc_net_line`, `runtime_line` on `Unavailable`).
