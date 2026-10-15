# Intent and shared understanding

**Capture date:** 2026-09-11
**Operator lock:** 2026-09-11 — understanding confirmed (see §Lock below).

## Lock (operator-confirmed 2026-09-11)

1. **Product Axis:** OS Shell (React) + Runtime (first-party) + Linux floor. Tauri is a living host in the monorepo, **not** the language of future plans.
2. **Two failures to avoid:** "A pretty skin without bridge/ops" and "Our own GNOME."
3. **Method:** Do not break the living; new = copy → paste → edit; soft-retire later.
4. **Separate repo (EmpiierOS)** — isolation of attention/machine for this axis; not just "an idea = a new folder."
5. **Foundation** is multi-wave and machine-checkable; no rushing to code the UI.

### Isolation clarification (operator)

| Level | Status |
|-------|--------|
| **Now enough** | Separate repo + separate Cursor window + do not mix with monorepo in a single agent session without a kinship-task |
| **Desirable max (not a start blocker)** | Separate Linux user / permissions — to be enabled later during the identity wave (F2) when we reach it |
| **Kinship** | monorepo = living Tauri / Substrate / API **read-only**; work home = this repo |

## Operator intent (as understood)

1. Session focused on the OS (Trusted Substrate kinship), **not** about refining the kiosk as the topic of the day.
2. Ambition for an admin-OS is deferred: first, a deeper **web + Linux** hybrid.
3. Do not remove Tauri now; but **do not plan the future through Tauri**.
4. React from the EliteEW frontend must become a **full-fledged OS UI** — acting like KDE/GNOME in role, but specifically only for EliteEW OS.
5. Do **not break the current state**: it will eventually go into history; for now, it remains living.
6. New = **separate**: copy file → paste file → edit file (a parallel line).
7. Separate git / possibly different user and permissions — to prevent a mess.
8. The foundation is multi-wave; no rushing to code the product.
9. A single "branching canon" is **insufficient** without a machine foundation — it will be forgotten within hours/days.

## Agent understanding (confirmed with operator)

| Phrase | Meaning |
|--------|---------|
| Separate | A new project/tree/repo + its own rules/docs/plans; not a feature inside `connection-agent` |
| Do not break | Zero required edits to living Tauri/FE/kiosk to start the new line |
| Copy-paste-edit | Seeded from files, subsequent drift is OK; not an in-place rewrite; **deliberate duplication** at the start |
| Hybrid | React = OS UI (shell plane); Linux + first-party bridge = ops plane |
| Tauri today | Living host / history-in-progress; not the normative axis for new plans |
| Foundation | Repo, identity, ignores, agent kernel, lessons, kinship — prior to product code |
| Mature OS capability | Planned as **shell surface + bridge op + substrate policy** — not as a `tauri command`/plugin |

## How the separate-repo idea was born (arc)

The chain of reasoning (origin dialogue → EmpiierOS), not "wanting a new git immediately":

1. **SoT without Tauri in the plan** — L-session / L-bridge / L-shell / L-apps; React = OS Shell mode of existing EliteEW DNA, not a new UI from scratch.
2. **Risks** — "A pretty skin" (UI without a canonical bridge) **or** "Our own GNOME" (loss of precision/focus).
3. **Do not break the living** → parallel line: copy → paste → edit (initially thought of as a pack/tree nearby).
4. **Drowning in the monorepo** → folders are not enough: one `.cursor`/dirty tree/agent context → requires a **separate git** + its own rules/docs/plans.
5. **Stronger isolation** → possibly a different user/permissions/identity + ignore-first.
6. **A canon without a machine will be forgotten** → multi-wave **foundation** (lessons, hooks, preflight), without rushing to code the product.
7. **Capture home** → first a notebook, then this repo.

A separate repo = a continuation of copy-paste-edit **at the project level**, plus protection of agent/human attention — not "just another EliteEW door."

## Working name (parked)

- **EliteEW OS** — product line
- **EliteEW OS Shell** — React as session UI
- **EliteEW Runtime** — first-party bridge / ops execution (not a "Tauri plan")
- **Trusted Substrate** — Linux floor (already in the monorepo); later a separate payload profile for the OS Shell

Directory/repo names are open (see `07-open-capture.md`).
