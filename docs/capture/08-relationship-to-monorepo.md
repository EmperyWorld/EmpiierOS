# Relationship to monorepo (`powered_by_emperyworld`)

**Capture date:** 2026-09-11

## Role of monorepo

Living home of:

- cloud / onprem / client **editions** contracts (shared API, SPA, packs);
- `pack-organization/connection-agent/` — Tauri living host;
- `pack-organization/trusted-substrate/` — Linux floor / kiosk payload today;
- agent rules/hooks/skills — **source of lessons**, not mandatory clone target.

## Role of EmpiierOS (this tree)

this repo

- **Home** of EliteEW OS foundation capture (moved 2026-09-11 from monorepo).
- Daily Cursor workspace for OS foundation + later shell/runtime.
- Monorepo keeps only a stub pointer — no dual full copy.

## Former monorepo capture path (retired)

Was: `powered_by_emperyworld/documentation/product/desktop/eliteew-os-foundation/`
Now: soft-retired / stub only in monorepo.

## Kinship rules (draft)

| Action | Monorepo | OS repo |
|--------|----------|---------|
| Fix living Tauri seat bug | yes | no (unless kinship task) |
| Design OS Shell | capture → then OS repo | yes |
| Change kiosk Substrate profile | yes (careful) | no break from OS line |
| Copy React tokens for seed | read origin | paste + ORIGIN.md |
| Invent second API cloud | no | no |

## Pack layout note

Monorepo pack layout remains SoT for EliteEW product doors.
OS line — **separate project kinship**, not a "fourth door inside the same agent drown" at the start.
Later, payload delivery (OS repo artifact → Substrate profile) can be described without merging working trees.

## Related: genEW as agent-practice kinship

`../genEW` — **not** a product seed for OS Shell, but a proven Cursor control plane (rules/hooks/skills/agents/orchestration) from which we take mechanisms for F1–F5.

- Harvest SoT in this repo: [`09-genew-harvest.md`](09-genew-harvest.md).
- Do not develop EmpiierOS foundation inside genEW.
- Do not open EmpiierOS + genEW + monorepo in one agent session without an explicit kinship task.

## Related: monorepo `.cursor` as management-methodology kinship

Living EliteEW agent OS lives at `powered_by_emperyworld/.cursor/` (rules / hooks / skills / agents / commands).

- **Study plan SoT:** [`10-monorepo-agent-forensic-plan.md`](10-monorepo-agent-forensic-plan.md) (**F1.2–F1.6 done 2026-09-13**; F1.7–F1.12 pending).
- F4 handoff: [`../foundation/AGENT-KERNEL-BRIEF.md`](../foundation/AGENT-KERNEL-BRIEF.md).
- Purpose of study: **AI agent management methodologies**, not product gate clones.
- Agent lessons source paths: `.cursor/rules/`, `.cursor/hooks/`, `.cursor/skills/`, `.cursor/agents/`, `.cursor/hooks.json`, `.cursor/README.md`.
- Read-only kinship; no edits to monorepo agent kernel from EmpiierOS sessions unless the operator opens an explicit monorepo task.

## Related: git history as scar-tissue kinship

History of monorepo + genEW is a **read-only** source of typical development and product mistakes (shotgun, Dual SoT, fake green, fix storms, gate self-fight, …).

- **Plan SoT:** [`11-git-history-forensic-plan.md`](11-git-history-forensic-plan.md) (F1.7).
- Goal: convert scars into EmpiierOS foundation controls — not to rewrite living history.
- Forbidden during forensic: any mutation of those git trees.

## Related: trash + soft-retired parks (DEWA / EWPower / …)

Monorepo `trash/` holds **ops trash-moves**; soft-retired product trees (**DEWA, EWPower, EWGen, EWC, EWCoderOrchestrator, SSFEW, …**) are **parked working tools** (not deleted) so they don’t interfere with living EliteEW.

- **Plan SoT:** [`12-trash-and-parked-products-forensic-plan.md`](12-trash-and-parked-products-forensic-plan.md) (F1.8–F1.9).
- Method kinship: monorepo skill `eliteew-capability-harvest` — read-only map; never restore whole trees without an explicit operator task.
- EmpiierOS studies them for foundation lessons; must not become a second DEWA/EWPower universe.

## Related: security + living Tauri launcher (study ≠ axis)

Living security + launcher/host in monorepo are **read-only kinship** for EmpiierOS foundation.

- **Plan SoT:** [`13-security-tauri-launcher-and-open-harvest-plan.md`](13-security-tauri-launcher-and-open-harvest-plan.md) (F1.10–F1.12).
- Key docs: `DESKTOP-CLIENT-SECURITY-BOUNDARIES.md`, `DESKTOP-CLIENT-LAUNCHER-JOURNEY.md`, `DESKTOP-TAURI-CONNECTED-CLIENT.md`, Trusted Substrate docs, `connection-agent/`.
- Stance: [`02-tauri-stance.md`](02-tauri-stance.md) — study allowed; Tauri is not the normative future axis; do not break the living host.
- F1.12: implementers may append any other useful foundation findings discovered while reading kinship trees.
