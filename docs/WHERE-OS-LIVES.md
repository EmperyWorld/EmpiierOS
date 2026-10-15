# WHERE OS LIVES

**Status:** Living one-pager (secure reset, updated 2026-09-21).
**Verdict:** `NOT_READY`.
**Home:** this repo — separate git, separate Cursor window.
**Glass map:** [`../session/SEAT-FACE.md`](../session/SEAT-FACE.md).

## Layout (day-1)

```text
EmpiierOS/
├── shell-ui/     # L-shell — secure React shell and native-bridge client
├── runtime/      # L-bridge — Rust daemon, authenticated Unix IPC, real probes/actions
├── session/      # L-session — seat/session contracts; not the Buildroot factory
├── docs/         # Living canon, maturity ledger, historical plans, capture archive
├── scripts/      # semantic preflight, packaging, smoke, and trash tooling
├── .cursor/      # Small agent kernel; guardrails are not acceptance authority
└── trash/        # Soft-delete landing (contents gitignored)
```

Normative tree detail: [`foundation/PROJECT-TREE.md`](foundation/PROJECT-TREE.md).

## Non-goals (here)

- Petty peripheral drift (receipt printers, barcode toy spoolers) — focus stays locked on the 2M-line EliteEW platform
- Product React / Runtime ops / FE seed without scoped wave + ORIGIN
- Second cloud / Keycloak / mobile / DEWA-style revive
- Wholesale clone of monorepo FE, `connection-agent`, or `.cursor` zoo
- Planning new capabilities as Tauri commands/plugins
- Breaking living Tauri / Substrate kiosk from this line

## Seed (policy only)

| | |
|--|--|
| **Allow** | Thin chrome/tokens; bridge stubs rewritten to Runtime; ORIGIN required |
| **Deny** | Wholesale SPA seed / `connection-agent`; cloud doors; BR factory; secrets; remote `product.open` / URL-from-JS |
| **SoT** | [`seed/SEED-POLICY.md`](seed/SEED-POLICY.md) — unlock + PW0 A scaffold; chrome seed still gated |

## Kinship (read-only)

| Tree | Role |
|------|------|
| `powered_by_emperyworld` | Living EliteEW and Trusted Substrate Buildroot SoT |
| `genEW` | Agent orchestration mechanisms (not product seed) |

Write/agent work = **this** repo by default. The active secure reset includes the
operator-approved Trusted Substrate scope under
`powered_by_emperyworld/pack-organization/trusted-substrate/`; other kinship
paths remain out of scope. Map: [`KINSHIP.md`](KINSHIP.md) · Floor contract:
[`foundation/SUBSTRATE-KINSHIP.md`](foundation/SUBSTRATE-KINSHIP.md).

## Isolation now vs later

| Now | Later (optional) |
|-----|------------------|
| Separate repo + Cursor window + no mixed session | Separate Linux user / remote / signing |

## Gate

```bash
npm ci --prefix shell-ui
bash scripts/preflight-foundation.sh
```

A local exit 0 is necessary but does not replace image, QEMU, or metal
acceptance. EmpiierOS preflight is not the Trusted Substrate selftest.
Product unlock remains scoped. Living source (not official `out/`):
seat launcher journey, always-on HUD, Seat Browser overlay, Seat
Shield, V2 install / LUKS2 state, Wi-Fi `AF_INET` plant, empty
Settings IPs, FoFline content-audio leftovers. Install:
[`../session/OS-INSTALL.md`](../session/OS-INSTALL.md). Authority:
[`FOUNDATION-STATUS.md`](FOUNDATION-STATUS.md) ·
[`plans/MATURITY-LEDGER.md`](plans/MATURITY-LEDGER.md) ·
[`../session/SEAT-FACE.md`](../session/SEAT-FACE.md).
