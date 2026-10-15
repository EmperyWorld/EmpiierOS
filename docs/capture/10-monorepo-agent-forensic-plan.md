# Monorepo agent forensic plan — EliteEW `.cursor` study

**Capture date:** 2026-09-11
**Source (read-only):** `../powered_by_emperyworld/.cursor/`
**Status:** Inventory + Phase B–**F** done (§E + AGENT-LESSONS + AGENT-KERNEL-BRIEF, 2026-09-13). F1.7–F1.12 tracks still open outside this file’s B–F.
**Purpose:** To identify **the best development management methodologies using the Cursor AI agent** — not to copy the product rules monorepo.

Companion harvest already completed for genEW: [`09-genew-harvest.md`](09-genew-harvest.md).
This file serves as the monorepo counterpart + phased study checklist.

---

## Why this stage exists

The monorepo contains a **mature, product-coupled** agent OS:

| Asset | Scale (2026-09-11 snapshot) |
|-------|----------------------------:|
| Rules (`rules/*.mdc`) | ~85 (~23 alwaysApply) |
| Skills | ~40 |
| Slash commands | ~40 (1:1 wrappers) |
| Hook scripts | ~40+ |
| `hooks.json` bindings | ~61 across 10 events (~22 failClosed) |
| Project agents | 2 (`inquisitor`, `tauri-security-reviewer`) |

Without a dedicated forensic wave, we either (a) drown EmpiierOS in the zoo, or (b) forget the real management gold (Commit Package, cause-first triad, trash, hard acceptance, living docs).

**North star of the study:** methodologies of **managing** Cursor agents
`intake → cause → readiness → proof → docs same-phase → Commit Package → acceptance`
— not EliteEW product law, not Tauri-as-axis, not deploy/Gemma zoo.

---

## A) Scale & map pointers

| Path | Role |
|------|------|
| `.cursor/README.md` | Workspace canon; open monorepo root; steward skill |
| `.cursor/hooks.json` | Lifecycle SoT |
| `.cursor/hooks/README.md` | Gate philosophy + failClosed table |
| `.cursor/domain_matrix.json` | Domains + pack_layout |
| `.cursor/universe_map.md` | Session atlas (hosts, doors) |
| `.cursor/rules/` | Agent law |
| `.cursor/skills/` | Workflow skills |
| `.cursor/commands/` | `/slash` → skills |
| `.cursor/agents/` | Project subagents |
| `.cursor/orchestration/` | recipes/reports (lighter than genEW) |

Ignore layers (also study for F3): root `.cursorignore`, `.cursorindexingignore`, nested FE ignore.

---

## B) Management clusters (filename map — for deep read)

### KEEP/HARVEST candidates (management core)

| Theme | Rules (examples) | Skills |
|-------|------------------|--------|
| Cause / anti-crutch / readiness | `cause-first-intake`, `eliteew-anti-crutch`, `eliteew-solution-readiness`, `eliteew-strict-roast`, `engineering-excellence` | `cause-first`, `eliteew-anti-crutch`, `eliteew-solution-readiness` |
| Acceptance / verify | `hard-acceptance-gate`, `task-intake-and-risk-gate` | `eliteew-hard-acceptance`, `verify-solution` |
| Git / Commit Package | `verified-proposal-protocol`, `git-agent-restrictions`, `eliteew-shotgun-budget` | `rework-commits` |
| Trash | `trash-and-deletion-policy` | (hooks: `trash-retention-gate`) |
| Living docs | `living-documentation-contract`, `documentation-maintenance`, `eliteew-fixed-errors` | `eliteew-doc-sync`, `eliteew-fixed-errors` |
| Session / steward | `agent-hooks`, `agent-skills`, `agent-configuration`, `agent-context-indexing` | `eliteew-session-start`, `eliteew-cursor-steward`, `plan-canon-locks` |
| Scope doors (pattern) | `domain-boundary-gate`, `eliteew-pack-layout` | `project-discovery-audit*` |
| Adversarial review | — | agent `inquisitor`; skills `review-bugbot`, product audits (posture only) |
| Capability harvest | — | `eliteew-capability-harvest` (trash harvest method) |

### DROP / defer for OS day-1 (product sediment)

| Theme | Examples |
|-------|----------|
| Mobile / EWHome / Site Studio / UGS / EWA | `android-mobile-*`, `mobile-*`, `ewhome-*`, `ewa-*`, `ugs-*`, `site-studio-*` |
| Keycloak encyclopedia | `keycloak-eliteew` rule + skill |
| Full domain_matrix / universe_map hosts | `.111`/`.136`/`.141` atlas — harvest *idea* of matrix only |
| Gemma mandatory auditor | `gemma-senior-auditor-gate` + `gemma-gate.py` + validate/deploy-gemma skills |
| Deploy / prod-line | `project-and-deploy`, `deploy-line-gate`, `eliteew-prod-line` |
| Tauri-as-axis | `tauri-security-*` rule/skill/agent/hooks stop-loop — keep *independent reviewer* pattern only |
| Desktop product shell cluster | `desktop-*.mdc`, premium UI product rules |
| Buildroot Dual SoT | `embedded-buildroot-sot` + BR gates |
| FE type zoo gates | `as-unknown-as`, `fe-ts-hygiene`, openapi-ssot — until OS has that stack |
| Master-card / passport / canvas phase zoo | optional later living-docs variants |

**Gate zoo risk: REAL.** Harvest **few fail-closed primitives + one completion loop**, not ~61 bindings.

---

## C) Study phases (F1 execution checklist)

Waves are **not** implemented yet — this is the plan to follow when the operator starts F1 (or F1-mono).

### Phase A — Inventory

- [x] Counts, top-level SoTs, cluster map, hooks events (this file, 2026-09-11)
- [ ] Optional: tag all 85 rules KEEP/HARVEST/DROP in a table (filename-only OK) — **deferred forever OK**; not a closeout blocker after F1.6

### Phase B — Management-core deep read

Read full text (rule + paired skill + cited platform docs if any):

1. `rules/cause-first-intake.mdc` + `skills/cause-first/SKILL.md`
2. `eliteew-anti-crutch.mdc` + `skills/eliteew-anti-crutch/`
3. `eliteew-solution-readiness.mdc` + skill
4. `hard-acceptance-gate.mdc` + `skills/eliteew-hard-acceptance/` (+ `verify-solution`)
5. `verified-proposal-protocol.mdc` + `git-agent-restrictions.mdc`
6. `trash-and-deletion-policy.mdc`
7. `living-documentation-contract.mdc` + `documentation-maintenance.mdc` + `eliteew-doc-sync`
8. `task-intake-and-risk-gate.mdc` + `eliteew-user-workflow-preferences.mdc`
9. `domain-boundary-gate.mdc` + `eliteew-pack-layout.mdc` (**pattern** only)
10. `skills/eliteew-session-start/`, `eliteew-cursor-steward/`, `plan-canon-locks/`
11. `agents/inquisitor.md`
12. Meta: `agent-hooks.mdc`, `agent-skills.mdc`, `agent-configuration.mdc`, `.cursor/README.md`

**Exit A/B:** notes appended under §E below + bullets in [`05-lessons-to-harvest.md`](05-lessons-to-harvest.md).

**Status:** [x] Phase B done 2026-09-13 — see §E.

### Phase C — Hooks machine-enforcement deep read

1. `hooks.json` — which events EmpiierOS needs day-1
2. `hooks/README.md`
3. Core scripts only: `session-bootstrap.py`, `phase_gate_state.py`, `trash-retention-gate.py`, `validate-agent-policy.py`, `anti-crutch-gate.py`, `solution-readiness-gate.py`, `hard-acceptance-gate.py`, `completion-gate.py`, `doc-phase-gate.py`, `git-phase-gate.py`, `prompt-secrets-gate.py`, `secrets-read-gate.py`, structure of `hooks-selftest.sh`
4. **Skip/port-later:** `gemma-gate.py`, `tauri-security-gate.py`, `deploy-line-gate.py`, Buildroot gates, FE hygiene/OpenAPI gates

**Exit C:** “thin hooks shortlist” for F4 (≤ ~8–12 bindings).

**Status:** [x] Phase C done 2026-09-13 — see §E F1.3.

### Phase D — Skills workflow deep read

Practiced pipeline:

```text
session-start → cause-first / plan-canon-locks
  → solution-readiness + anti-crutch
  → implement
  → doc-sync
  → hard-acceptance / verify-solution
  → Commit Package (verified-proposal)
  → optional inquisitor / bugbot / rework-commits
```

Also: `project-discovery-audit*`, `eliteew-fixed-errors`, `eliteew-capability-harvest`.

**Exit D:** corrected pipeline map + KEEP/HARVEST/DROP for skills → `05` / §E; target ≤12 management skills for F4.

**Status:** [x] Phase D done 2026-09-13 — see §E F1.4.

### Phase E — Compare vs genEW ([`09`](09-genew-harvest.md))

| Strength | genEW | EliteEW monorepo |
|----------|-------|------------------|
| Kernel-first + design→verify artifacts | strong | lighter orchestration |
| Factory compose-not-invent | strong | different (doors/packs) |
| Independent stage audit | `independent-work-audit` | product/interface audits + inquisitor |
| Commit Package / trash / cause triad | softer | **strong** |
| Hard acceptance stop-loop | verify reports | **strong** (gate zoo risk) |
| Living docs same-phase | lighter | **strong** |

**Produce:** union of **mechanisms**, not union of product gates.

**Exit E:** comparison table filled + union mechanism list in §E; caps for F4 confirmed against both kinships.

**Status:** [x] Phase E done 2026-09-13 — see §E F1.5.

### Phase F — Deliverables (F1 exit)

- [x] `AGENT-LESSONS.md` — durable mechanism laws + anti-pattern catalog (F1.6)
- [x] KEEP / HARVEST / DROP / **REWRITE** tables
- [x] Cap proposals: alwaysApply ≤9; hooks ≤8; skills ≤12; agents ≤4
- [x] Explicit anti-pattern table (A1–A15)
- [x] OS agent kernel brief for F4: `docs/foundation/AGENT-KERNEL-BRIEF.md`
  - Fold: `09` + this file §E (F1.2–F1.5) + pointers to plans `11`–`13` (**execution still open** — honesty; not silent skip)

**Status:** [x] Phase F done 2026-09-13 — monorepo agent forensic B–F closed. F1.7–F1.12 remain separate tracks.

---

## D) Hooks philosophy (one-liner for plans)

Rules state law; hooks **machine-enforce**. Prefer few `failClosed` (secrets, git policy, trash, anti-crutch/readiness/acceptance) + fail-open phase reminders. Full monorepo mesh = gate fatigue — do not port wholesale.

Events present: `sessionStart`, `sessionEnd`, `beforeSubmitPrompt`, `beforeReadFile`, `beforeShellExecution`, `subagentStart`, `preToolUse`, `postToolUse`, `afterFileEdit`, `stop`.

---

## E) Deep-read notes (append-only during F1)

### F1.2 / Phase B — Management-core deep read (2026-09-13)

**Source:** `../powered_by_emperyworld/.cursor/` (read-only).
**Checklist 1–12:** all listed files present and read. Product zoo (Gemma, Tauri stop-loop, mobile, deploy, BR) skipped.

#### Theme notes

| Theme | Mechanism (compressed) | EmpiierOS action |
|-------|------------------------|------------------|
| Cause-first | Cause → Lock → Measure; intake template; causal chain; mandatory closeout (root cause / paths / evidence / HA / Commit Package) | **KEEP/STRENGTHEN** R6 + skill — add closeout/intake template; DROP audit score caps |
| Anti-crutch | Crutch Test → `NOT_CRUTCH`; euphemism ban; prod writes need READY + machine proof | **HARVEST light** into R6 (already); **DROP** write-deny `anti-crutch-gate` day-1 |
| Solution readiness | Exhaust outcomes/errors before implement; `status: READY` + matrix | **REWRITE** thin intake/risk — **DROP** READY markdown + write-deny gate |
| Hard acceptance | 15-axis session audit; VERIFIED/BLOCKED; stop-loop; Gemma→Tauri→HA order | **KEEP HA-light**; **DROP** 15-axis stop-loop / Gemma / Tauri mesh |
| VPP / git | Commit Package → operator approval; VPP **wins** over autonomous-commit language; trash incident freezes git | **KEEP** R4 + hook; strengthen package fields; soft shotgun → **plane budget** only |
| Trash | No agent deletes; only trash-move; accidental delete = hard stop | **KEEP** R5; **STRENGTHEN** incident freeze + wire SPEC `trash-not-rm` hook later |
| Living docs | Read canon before code; same-phase rewrite; soft-retire false docs | **KEEP** R7; strengthen read-first + doc-impact line; **DROP** master-card/passport zoo |
| Intake / DoD | Free-form → goal/boundaries/risk/proof; edge matrix Offline/dual-flag/permission/regression | **HARVEST** into session-start / future `plan`; **DROP** Ruff/Biome/Gemma/Tauri DoD |
| Session-start | Preflight + priority stack + cause briefing | **KEEP thin**; already aligned |
| Steward | Audit `.cursor` vs caps + fresh Cursor docs; trash-not-rm; alwaysApply diet | **KEEP**; fix local cap drift (skill said ≤8, SPEC = **9**) |
| Plan-canon-locks | Intake → facts → plan → **operator scope lock** → **SoT wins over plan** → HA → Commit Package → implement | **HARVEST** into future `plan` skill (high leverage) |
| Doors / pack | One door per session; cross-domain needs impact analysis | **REWRITE** as R1 planes / one workspace root; **DROP** pack dirs + domain_matrix |
| Inquisitor | Readonly adversarial; default REJECT; Edge→KISS→Dependency; no praise/implement | **DEFER** (SPEC after F1.6); do not port EliteEW weapon list; `verifier` ≠ inquisitor |
| Meta | Rules=invariants; skills=workflows; hooks=deterministic deny; repeated error→hook | **HARVEST** into steward/README; **DROP** universe injection / MCP zoo |

#### Durable mechanism laws (Phase B)

1. Close on **Cause → Lock → Measure**; Measure/Lock alone ≠ Done.
2. **No crutches** / euphemism rebrands of workarounds.
3. Exhaust **in-scope outcomes** before implement (readiness — light form).
4. Acceptance needs **machine evidence**; edits invalidate verdicts.
5. **Commit Package → human approval**; never surprise commit/push.
6. **Soft-delete only**; delete incident = hard stop until operator closes.
7. **Read living SoT before code**; rewrite docs **same phase** as contracts.
8. Free-form intake → agent-owned **goal / boundaries / risk / proof**.
9. **One plane/workspace per session**; cross-boundary needs impact + approval.
10. **SoT wins over process plan**; lock scope before implement waves.
11. Rules instruct; **hooks enforce** repeated/security failures (`failClosed` when deny).
12. Adversarial review assumes the plan is wrong until attacks fail.

#### Explicit DROP (confirmed by deep read)

Gemma-as-HA-prereq · HA 15-axis stop-loop · Tauri F-01… mesh · readiness/anti-crutch **write-deny** gates · fixed-errors product registry · pack-cloud/org editions DoD · master-card/passport/canvas doc zoo · universe_map session injection · EliteEW dir shotgun list · deploy/Keycloak/BR Dual SoT hooks.

#### Gaps vs EmpiierOS (fill thin — not circus)

| Gap | Fill with | Not with |
|-----|-----------|----------|
| Cause closeout template | Strengthen `cause-first` skill | Product-audit caps |
| Edge/risk at intake | `session-start` / future `plan` | READY files + write gates |
| Stronger Commit Package fields | `verified-proposal` skill | Git completion-gate zoo |
| Trash incident freeze + hook | R5 + later SPEC hook | Full trash-retention mesh |
| Doc read-first + impact line | R7 prose / thin note | `eliteew-doc-sync` zoo |
| Plan→SoT→locks pipeline | Future `plan` skill | Premium/scaffold peers |
| Independent adversarial pass | Later inquisitor / verifier | Gemma auditor |
| Steward cap text drift | Fix skill vs SPEC R1–R9 | Port monorepo alwaysApply diet |

#### OS completion loop (from Phase B)

```text
intake (goal/planes/risk/edge)
  → cause-first (+ light crutch)
  → same-phase foundation docs
  → evidence / preflight
  → HA-light
  → Commit Package
  → operator approve
```

#### Surprises / high leverage

- Cause triad in monorepo is **three coupled machines** (rule + readiness gate + anti-crutch gate); OS correctly merged Cause+Crutch into R6 — do not re-split into write-deny circus.
- **«SoT wins over plan»** (plan-canon-locks) is the cleanest harvest for foundation docs vs capture essays.
- Delete incident **freezes git entirely** — stronger than EmpiierOS trash prose today.
- Meta law: repeated error → rule; if ignored → hook + rule + selftest — justifies few failClosed without copying ~61 bindings.
- Steward skill claimed ≤8 alwaysApply while SPEC/disk are **R1–R9** — local invariant bug (fixed in same F1.2 writeback).

**Phase B status:** done 2026-09-13.

### F1.3 / Phase C — Hooks machine-enforcement deep read (2026-09-13)

**Source:** monorepo `.cursor/hooks.json` + `hooks/README.md` + core scripts (read-only).
**Scale confirmed:** 10 events · **61** bindings · **22** failClosed · 39 fail-open. EmpiierOS today: **3** failClosed.

#### Philosophy (rewrite for OS)

Rules instruct invariants; skills teach workflows; hooks **machine-enforce** only repeated or security failures. Prefer a small failClosed set + fail-open reminders. Full monorepo mesh = gate fatigue — do not port.

#### Management-core map (compressed)

| Event | Management gates | Mode |
|-------|------------------|------|
| `sessionStart` | session-bootstrap (+ phase state reset) | advisory |
| `beforeSubmitPrompt` | prompt-secrets | failClosed |
| `beforeReadFile` | secrets-read; doc-phase track | FC / advisory |
| `beforeShellExecution` | shell-exec (trash/git/…) + product zoo | failClosed — zoo DROP |
| `preToolUse` | trash-retention; validate-agent-policy; readiness+anti-crutch write-deny; zoo | FC — write-deny DROP |
| `postToolUse` / `afterFileEdit` | phase trackers; HA invalidate | mostly advisory |
| `stop` | hard-acceptance loop; completion reminders; gemma/tauri stop | HA/gemma/tauri DROP |

#### Core script leverage

| Script | EmpiierOS |
|--------|-----------|
| trash-retention-gate | **REWRITE** → `trash-not-rm.py` (Delete+shell); skip Write-on-existing if too harsh |
| validate-agent-policy | **DROP** wholesale — overlap with stricter `git-autonomy-guard` |
| anti-crutch / solution-readiness gates | **DROP** as hooks (keep rule/skill) |
| hard-acceptance-gate | **DROP** stop-loop; HA-light + preflight stay |
| completion-gate | **REWRITE** → thin `foundation-completion-hint.py` |
| session-bootstrap | **REWRITE** thin (preflight/planes/VPP) — no universe map |
| phase_gate_state | **HARVEST** helper only if stop/doc reminders need state |
| doc-phase / git-phase | **HARVEST** later advisory / fold into stop hint |
| prompt-secrets / secrets-read | **HARVEST** later (write-only secrets today) |
| hooks-selftest.sh (~1325 LOC product mesh) | **REWRITE** tiny OS selftest later — not port |

#### Thin hooks shortlist for F4 (≤8; keep existing 3)

| # | Event | Script | failClosed | Priority |
|---|-------|--------|------------|----------|
| 1 | `beforeShellExecution` | `security-destructive.py` | yes | **now** (have) |
| 2 | `beforeShellExecution` | `git-autonomy-guard.py` | yes | **now** (have) |
| 3 | `preToolUse` Write\|Delete\|… | `protect-secrets.py` | yes | **now** (have) |
| 4 | `preToolUse` Shell\|Delete | `trash-not-rm.py` | yes | **next** (SPEC; biggest gap) |
| 5 | `sessionStart` or `beforeSubmitPrompt` | `session-root-hygiene.py` | no | later |
| 6 | `stop` | `foundation-completion-hint.py` | no | later |
| 7 | `beforeSubmitPrompt` | `prompt-secrets.py` | yes | later |
| 8 | `beforeReadFile` | `secrets-read.py` | yes | later |

Optional if budget allows: advisory `style-modularity-hint` / `audit-log` — defer.
**Budget path:** 3+trash = 4 FC; +2 advisory = 6; +secrets pair = 8. Do **not** add readiness/anti-crutch/HA stop.

#### Explicit DROP (hooks)

Gemma (+stop) · Tauri security mesh (+stop) · deploy-line / UV host · Buildroot Dual-SoT · FE hygiene/OpenAPI · scaffold/user-product/validate-doc path zoo · canvas/passport/master-card/fixed-errors hooks · domain_matrix/universe injection · readiness+anti-crutch **write-deny** · full HA `loop_limit: null` · wholesale 61-binding `hooks.json`.

#### Gap vs EmpiierOS (not circus)

1. **`trash-not-rm`** — Delete / plain `rm` / `git rm` not fully covered by `security-destructive` alone.
2. **`session-root-hygiene`** — wrong-root kinship warn.
3. **`foundation-completion-hint`** on stop — preflight / anti-false-green.
4. Optional: prompt + read secrets.

#### Selftest

Yes later: tiny `scripts/hooks-selftest.sh` or `.cursor/hooks/hooks-selftest.sh` — validate JSON + ≤cap bindings + one deny/allow per failClosed script. Not monorepo 1325-line mesh.

**Phase C status:** done 2026-09-13. Implementation of shortlist #4+ = F4 task (not auto).

### F1.4 / Phase D — Skills workflow deep read (2026-09-13)

**Source:** monorepo `.cursor/skills/` (~40) + slash `commands/` (1:1 wrappers). EmpiierOS skills compared (6 on disk).

#### Correction

Monorepo has **no** `eliteew-implement` / `/implement` skill. “Implement” = freeform coding after gates, or **P7** inside `plan-canon-locks`. genEW `/plan`→`/implement`→`/verify`→`/evolve` is a **different** control-plane shape (EmpiierOS SPEC target). EmpiierOS should prefer **genEW skill names** + **EliteEW mechanisms**.

#### Actual monorepo pipeline

```text
/eliteew-session-start
  → /cause-first  AND/OR  /plan-canon-locks   (P0–P6; SoT wins; operator scope lock)
  → /eliteew-solution-readiness  +  /eliteew-anti-crutch
  → [IMPLEMENT — unnamed]  (plan-canon P7 or direct after READY)
  → /eliteew-doc-sync
  → /eliteew-hard-acceptance  ≡ /verify-solution
  → Commit Package via RULE verified-proposal-protocol.mdc  (no VPP skill in monorepo)
  → optional: inquisitor | /review-bugbot | /rework-commits
```

#### Stage → EmpiierOS

| Stage | Action |
|-------|--------|
| session-start | **KEEP** thin (have) |
| cause-first | **KEEP** (have + R6) |
| plan-canon-locks | **HARVEST** → future thin `plan` (highest leverage gap) |
| solution-readiness / anti-crutch | **REWRITE** light into intake/R6; **DROP** write-deny + READY files |
| implement (unnamed) | **REWRITE** later as genEW-shaped `implement` — not EliteEW product skill |
| doc-sync | **HARVEST** mechanism → R7; **DROP** master-card/passport/canvas |
| hard-acceptance / verify-solution | **KEEP** HA-light; **DROP** alias as separate + 15-axis/Gemma |
| Commit Package | **KEEP** OS skill `verified-proposal` + R4 (monorepo = rule-only) |

#### Side skills

| Skill | OS |
|-------|-----|
| capability-harvest | **DEFER** stub after F1.8/F1.9; never restore |
| fixed-errors | **DROP** product registry |
| project-discovery-audit* | **DROP** day-1 |
| rework-commits / review-bugbot | **DROP** day-1 |
| inquisitor | **DEFER** after F1.6; ≠ `verifier` |

#### Target skills map (≤12, F4)

| Skill | Disk | Action |
|-------|------|--------|
| session-start, cause-first, verified-proposal, hard-acceptance-light, preflight-foundation, cursor-steward | yes | KEEP |
| `plan` | no | **ADD later** — harvest plan-canon-locks (SoT wins, scope lock) |
| `implement`, `verify`, `evolve` | no | **ADD later** — genEW shape + orchestration artifacts |
| `independent-work-audit` | no | **ADD later** — stage READY/DEBT |
| `capability-harvest` | no | stub later |

**NEVER:** premium/scaffold/factory/prod-line/Keycloak/deploy/gemma/tauri-security/product audits/doc phase zoo/readiness write-gate/fixed-errors.

#### Durable workflow laws (Phase D)

1. Skills = workflows; rules = invariants; hooks = enforce — don’t triplicate.
2. Session entry: preflight + goal/planes/risk/edge before substantive writes.
3. Medium+: Cause → Lock → Measure.
4. Process plan ≠ SoT; **SoT wins**; operator **scope lock** before implement.
5. Exhaust outcomes / Crutch Test before code — light form only.
6. EliteEW implement has no magic skill name; OS names it only if adopting genEW lifecycle.
7. Living docs same-phase as contract change.
8. Acceptance = evidence on session scope; narrative ≠ green.
9. Commit Package → human approval; skills cite VPP, don’t auto-commit.
10. Slash commands are thin wrappers; shape = skill names.
11. Inquisitor/bugbot/rework = side exits, not mandatory DoD.
12. Cap ≤12 management skills; product-door skills never enter OS kernel.

#### Explicit DROP (skills)

Gemma HA prereq · HA 15-axis/stop-loop · Tauri security as merge bar · readiness/anti-crutch write-deny · fixed-errors registry · master-card/passport/canvas · premium/scaffold/factory/prod-line/Keycloak/deploy · verify-solution as separate product skill · rework-commits/bugbot as day-1 DoD · session-start product zoo (editions/Biome/pack doors).

#### Gaps (thin, not circus)

1. Thin **`plan`** skill (SoT vs plan, scope lock) — highest leverage.
2. Named **`implement` / `verify` / `evolve`** (genEW shape) when lifecycle tasked.
3. **`independent-work-audit`**.
4. Doc impact one-liner in closeouts (not full doc-sync skill).
5. Optional capability-harvest stub post kinship/trash.

#### Link F1.2 / F1.3

Phase D confirms F1.2 completion loop and maps monorepo’s longer skill chain onto it. Readiness/doc-sync/HA are **mechanism sources**, not hooks to revive — F1.3 DROP of write-deny / HA stop-loop stands.

**Phase D status:** done 2026-09-13. Do **not** auto-add `plan` skill (F4 / operator task).

### F1.5 / Phase E — Compare genEW ↔ EliteEW monorepo (2026-09-13)

**Sources:** EmpiierOS [`09-genew-harvest.md`](09-genew-harvest.md) + F1.2–F1.4 notes above. Spot-check genEW: skills `plan/implement/verify/evolve/independent-work-audit` present; agents 4+generator-root-cause; hooks **8 bindings / ~6 failClosed** (vs monorepo **61 / 22**).

#### Strengths matrix (filled)

| Strength | genEW | EliteEW monorepo | EmpiierOS take |
|----------|-------|------------------|----------------|
| Kernel-first + design→verify artifacts | **strong** (`d0e1bd8`, designs/reports) | lighter orchestration | **HARVEST genEW shape** — stubs already; real cycle when product waves |
| Factory compose-not-invent | **strong** (recipes) | doors/packs / seed ORIGIN | **REWRITE** — seed policy + planes; no recipe zoo day-1 |
| Independent stage audit | **strong** (`independent-work-audit`) | product audits + inquisitor | **HARVEST genEW skill later** + defer inquisitor |
| Commit Package / trash / cause triad | softer | **strong** | **KEEP EliteEW** (R4/R5/R6 already) |
| Hard acceptance | verify reports / evidence | **strong** stop-loop (**gate zoo risk**) | **REWRITE** HA-light + preflight — **DROP** 15-axis/Gemma |
| Living docs same-phase | lighter | **strong** | **KEEP EliteEW** R7 |
| Named skill lifecycle | `/plan→/implement→/verify→/evolve` | unnamed implement + plan-canon-locks | **genEW names** + **EliteEW SoT-wins / scope lock** |
| Hook budget | few failClosed (~6 of 8) | mesh 61 / 22 FC | **genEW scale** + EliteEW trash/VPP mechanisms in shortlist |
| Fix generator not symptom | **strong** (capsule rule) | cause-first / anti-crutch | **UNION** — R6 already; no capsule-protect until generator exists |
| Honesty / no fake green | production-path | HA + honesty rules | **UNION** — R8 + preflight |
| Adversarial review | stage audit | inquisitor default REJECT | **HARVEST posture** later; ≠ cheerleader verifier |

#### Union of mechanisms (EmpiierOS SoT for F4)

**From genEW (shape):**
1. Four-layer control plane: agents / skills / hooks / rules / orchestration artifacts
2. Lifecycle: Intent → Plan → Implement → Verify → Evolve; no ready if verify failed/skipped
3. Design artifacts + verify reports; evidence > prose; deterministic FAIL beats LLM PASSED
4. Role ownership matrix (architect ≠ ready; implementer ≠ ready; verifier = gate)
5. Skills as Task wrappers (`disable-model-invocation`)
6. Few failClosed + advisory style
7. Independent stage audit (READY / READY_WITH_DEBT / NOT_READY)
8. Kernel-first bootstrap before product flood
9. Context firewall (ignore noise; keep meta indexed)
10. Compose-not-invent (idea only until packs exist)

**From EliteEW monorepo (management gold):**
1. Cause → Lock → Measure + light Crutch Test
2. Verified Proposal / Commit Package (VPP wins)
3. Trash-not-rm + delete incident freeze
4. Living docs same-phase + read-first
5. Plan-canon: **SoT wins over plan** + operator scope lock
6. Session intake: goal / planes / risk / edge
7. Soft shotgun → plane budget (advisory)
8. Door/pack pattern → OS planes / one workspace root
9. Inquisitor posture (defer)
10. HA evidence mindset without stop-loop circus

**Explicit non-union (DROP both zoos):**
- genEW: U1–U9/D tax, Hands/EIC, recipe-atom catalog, absolute remote-block, capsule-protect day-1, assembly T/C/S
- monorepo: Gemma HA, 15-axis stop-loop, readiness/anti-crutch write-deny, 61-hook mesh, product doors, Tauri-as-axis, master-card zoo

#### Caps confirmed (F4)

| Cap | Target | Rationale |
|-----|--------|-----------|
| alwaysApply rules | ≤9 (R1–R9 now) | EliteEW diet; don’t grow toward 23 |
| Hook bindings | ≤8 (3 now; shortlist to 6–8) | genEW-scale failClosed count |
| Management skills | ≤12 (6 now) | genEW lifecycle 5 + OS preflight/steward/VPP/HA-light + later audit/harvest |
| Agents | ≤4 day-1 (+ optional inquisitor later) | genEW role matrix |

#### EmpiierOS completion loop (union, locked)

```text
intake (goal/planes/risk/edge)
  → plan (SoT wins; operator scope lock)     [later skill; genEW name + EliteEW locks]
  → cause-first (+ light crutch)
  → implement (from design; no ready claim) [later]
  → same-phase docs
  → verify / HA-light + preflight evidence
  → optional independent-work-audit (stage)
  → Commit Package → operator approve
```

#### Gaps after Phase E (still not circus)

| Gap | Source | When |
|-----|--------|------|
| Thin `plan` skill | EliteEW plan-canon + genEW `/plan` | F4 |
| `implement` / `verify` / `evolve` | genEW | F4 when lifecycle tasked |
| `independent-work-audit` | genEW | F4/F5 stage |
| `trash-not-rm` hook | EliteEW | F4 (F1.3 #4) |
| Real design→verify golden cycle | genEW | first product/foundation meta task — not empty flood |
| inquisitor | EliteEW | after F1.6 |

**Phase E status:** done 2026-09-13.

### F1.6 / Phase F — Deliverables fold (2026-09-13)

**Exit artifacts:**

| Deliverable | Path |
|-------------|------|
| Mechanism lessons + anti-patterns A1–A15 + REWRITE | `docs/AGENT-LESSONS.md` |
| F4 handoff brief | `docs/foundation/AGENT-KERNEL-BRIEF.md` |
| Caps locked | rules ≤9 · hooks ≤8 · skills ≤12 · agents ≤4 |
| Fold of `09` + §E B–E | brief + lessons |
| Fold of `11`–`13` | **pointers only** — deep execution not done; brief §Deferred |

**Honesty:** Phase F closes the **monorepo agent forensic** (B–F). It does **not** claim F1.7 git scars, F1.8–F1.9 trash/parked, or F1.10–F1.12 security/launcher are executed.

**Phase F status:** done 2026-09-13. Monorepo `.cursor` study track complete for foundation. Next operator choice: F1.7+ forensics **or** thin F4 (`trash-not-rm` / `plan` skill).

-

---

## F) Relation to foundation waves

| Wave | Use of this file |
|------|------------------|
| **F1** | Execute Phases B–F here + git forensic `11` + trash/parked `12` (still pending) |
| **F3** | Ignore ideas from monorepo ignore layers |
| **F4** | Mini kernel from KEEP/HARVEST only |
| **F5** | Preflight proves thin kernel — not 85 rules |

See [`04-foundation-waves.md`](04-foundation-waves.md) F1 section.
