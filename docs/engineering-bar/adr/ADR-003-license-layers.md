# ADR-003: License layers stay separate

- Status: superseded by [ADR-004](ADR-004-mit-public-source.md) on 2026-10-16
- Date: 2026-09-23

## Context

The seat runs a GPL kernel and GPL userland beside a proprietary shell and runtime. Collapsing those into one license either opens the UI or hides copyleft.

## Decision

- Follow `LICENSE` and `NOTICE`. Proprietary covers shell-ui, the main runtime, and shell-gui. Seat crates that declare MIT stay MIT.
- Do not vendor kernel or other GPL sources here to "link more easily".
- Do not claim registry inclusion in code or docs.

## Consequences

An unknown-license dependency is blocked at review. It is not merged as a follow-up.
