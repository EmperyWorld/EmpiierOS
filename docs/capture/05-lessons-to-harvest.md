# Lessons to harvest (draft list)

**Capture date:** 2026-09-11
**Source:** conversation + living EliteEW agent experience (rules/hooks/skills/git management).
**Use:** input to F1 forensic → curated OS agent kernel (KEEP/HARVEST/DROP/REWRITE).

## Management / process lessons (AI + Cursor)

1. **Monorepo drowning** — too many products in one agent context → scope errors, shotgunning, forgotten boundaries.
2. **Canon without machine** — beautiful markdown without hooks/preflight is forgotten within hours/days.
3. **In-place rewrite of living product** — "improving for the future" breaks what is already in combat. Requires a parallel line + soft-retire later.
4. **Framework-as-plan-axis** — planning "via Tauri/Electron" embeds a foreign framework into the DNA of decisions.
5. **Too many layers named, too few owned** — separation is good, but if every layer = new mental tax without a clear owner, the team drowns.
6. **Shotgun commits / dirty tree** — mixing OS/cloud/mobile/docs in a single commit package.
7. **Autonomous git commit/push** — an agent must not close a task with a commit without approval (Verified Proposal).
8. **Delete vs trash** — destructive deletions in the tree = an incident; requires a trash-move mindset.
9. **Doc lag** — code moves, canon lies; doc-same-phase is mandatory on contract change.
10. **Symptom fixes** — i18n/guard/scorecard without a cause-layer; anti-crutch / cause-first.
11. **Dual SoT** — two sources of truth (live USB vs overlay; docs vs code) always exacts revenge.
12. **Gate fatigue** — too many gates → desire to bypass; better to have few strict gates than fifty decorators.
13. **Cross-door edits "while I'm at it"** — a cloud session touches the org door and vice versa.
14. **Second product tree temptation** — DEWA/EWGen-style; harvest ≠ restore whole tree; the OS repo must not become a second cloud.
15. **Bootstrap honesty** — Electron/Tauri/engine labels must stay honest (`payload_trust`), no fake "fully from-source" claims.

## KEEP for OS agent kernel (candidates)

- Cause-first closeout
- Anti-crutch / solution readiness mindset (scoped, not a circus)
- Verified Proposal (Commit Package; no surprise commit/push)
- Trash-not-rm
- Doc-same-phase on contract change
- Hard acceptance light / evidence before "done"
- User product obligations (no sysadmin UX as a product)
- Engineering excellence: no invented topology; env not secrets in code

## HARVEST (rewrite for OS)

- Pack layout → **OS layout** (shell / runtime / session)
- Trusted Substrate kinship (payload profiles, dual-flag) without importing the full BR factory into day-1
- Desktop Kernel vs App thinking → OS Shell vs OS apps
- Security boundary thinking (capabilities, main vs product) without using Tauri vocabulary as the SoT

## DROP (do not copy into OS `.cursor` day-1)

- Full cloud deploy line (eeprodbaked / prod-core bake ritual zoo)
- Mobile door rules
- Keycloak encyclopedia as always-on
- Gemma/prod-core auditor as mandatory for empty foundation repo (revisit later if needed)
- Entire domain_matrix of monorepo
- "Open both doors" habits

## REWRITE

- Tauri ≠ future planning axis
- Foundation waves before product waves
- Separate identity / ignore-first
- Copy-paste-edit seed policy with ORIGIN.md

## Git history forensic (F1.7 — **done** 2026-09-13; SoT [`11`](11-git-history-forensic-plan.md))

**SoT plan:** [`11-git-history-forensic-plan.md`](11-git-history-forensic-plan.md)
**Rule:** read-only `git log`/`show` on monorepo + genEW; findings **only** appended here / in `11` §Findings. No kinship tree edits.

Hunt taxonomy (short):

- **D\*** process: shotgun, mixed packages, symptom churn, revert loops, WIP/hack, docs lag, gate self-fight, cross-door
- **P\*** product: Dual SoT, in-place living rewrite, fake readiness, framework-as-axis, hollow Done, generator-vs-symptom, honesty labels, release churn without cause
- **Q\*** productivity: context drown, gate fatigue→bypass, redo loops, late isolation, missing stage audit

Each finding → foundation control (F3 ignore / F4 rule-hook-skill / F5 preflight).

### Smoke hypotheses only (2026-09-11 — confirm in G1–G3)

- Monorepo: Dual SoT kill on Trusted Substrate; agent-gates anti-lie; EBrowser policy SoT SPA→Rust; dense fix/feat/OTA clusters; possible mega-commit churn.
- genEW: fake-PASSED debt closeouts; junk teardown; fix-generator-not-capsule; kernel-first positive counter-example.

### Forensic notes (append-only)

- (done 2026-09-13) F1.7 G1–G4 — findings in [`11`](11-git-history-forensic-plan.md). Key hashes: Dual SoT `29d6c5def`/`f364829a5`; mega `f357fe295`; Gemma anti-lie `8a1631864`; genEW kernel `d0e1bd8`; fix-generator `110360d`; fake-PASSED `1f56608`.
- **2026-09-11 — git history forensic plan:** [`11-git-history-forensic-plan.md`](11-git-history-forensic-plan.md). **Executed F1.7 2026-09-13.**
- **2026-09-11 — trash + parked products plan:** [`12-trash-and-parked-products-forensic-plan.md`](12-trash-and-parked-products-forensic-plan.md). F1.8 ops-trash + F1.9 DEWA/EWPower/… — locate smoke only; not deep-read yet.

---

## Trash + parked products (F1.8–F1.9 — **done** 2026-09-13; SoT [`12`](12-trash-and-parked-products-forensic-plan.md))

**Operator framing:** DEWA / EWPower / EWGen / … **not crossed out** — parked working tools so they don’t interfere with living EliteEW. Study them as exemplars; **harvest ≠ restore**.

**Host honesty (2026-09-13):** soft-retire batch `Документы/trash/20260718-015129/` = **0 files** (skeletons). Ops trash monorepo ~140 buckets / 4G. Full EWGen = `Документы/EWGen` + genEW.

Two kinds:

1. **Ops trash** — monorepo `trash/YYYYMMDD-…` (trash-move culture, agent debris).
2. **Soft-retire parks** — named product trees (DEWA, EWPower, …) per `.cursorrules` / `eliteew-capability-harvest`.

---

## Security + Tauri launcher + open finds (F1.10–F1.12 — **done** 2026-09-13; SoT [`13`](13-security-tauri-launcher-and-open-harvest-plan.md))

**SoT:** [`13-security-tauri-launcher-and-open-harvest-plan.md`](13-security-tauri-launcher-and-open-harvest-plan.md).

- **F1.10:** security one-pager — Runtime ACL, policy-in-Runtime, F8 inputs; Tauri skill = pattern only.
- **F1.11:** launcher journey → Shell/Runtime rewrite map; do-not-break list; chrome/tokens F6 only.
- **F1.12:** open log (≥10 FOUND); lessons #38–42.
- **Stance:** `02` held — Tauri living host ≠ plan axis. Zero living host edits.
- Execution not started.

---

## Monorepo `.cursor` inventory snapshot (2026-09-11)

**Root:** `../powered_by_emperyworld/.cursor/`
**Study plan SoT:** [`10`](10-monorepo-agent-forensic-plan.md) (Phases A–F).

| Asset | ~Count | Signal for EmpiierOS |
|-------|-------:|----------------------|
| Rules | 85 (~23 alwaysApply) | Mature law; **gate/rule zoo risk** — harvest management core only |
| Skills / commands | 40 / 40 | Strong management skills (cause-first, anti-crutch, readiness, hard-acceptance, VPP, session-start, steward) |
| Hook bindings | ~61 (10 events, ~22 failClosed) | Machine-enforce works; **do not port mesh** |
| Agents | 2 | `inquisitor` = adversarial review gold; Tauri reviewer = pattern only |

### Management KEEP/HARVEST shortlist (Phase B confirmed 2026-09-13)

See full notes: [`10`](10-monorepo-agent-forensic-plan.md) §E.

- Cause-first ↔ anti-crutch ↔ solution-readiness triad → OS: **merged** Cause+Crutch in R6; readiness = light intake only
- Hard acceptance + Commit Package → HA-light + VPP (KEEP); DROP Gemma/15-axis stop-loop
- Living docs same-phase + read-first (KEEP R7)
- Trash-not-rm + incident freeze (KEEP/STRENGTHEN)
- Session-start + cursor-steward + plan-canon-locks → plan skill later (HARVEST)
- Inquisitor posture deferred; doors → planes rewrite
- Capability-harvest-from-trash method (still F1.8+)

### Explicit DROP day-1 (confirmed F1.2 deep read)

- Mobile / EWHome / Site Studio / UGS / EWA / Keycloak encyclopedia
- Full `domain_matrix` + LAN `universe_map` hosts
- Gemma mandatory auditor + deploy/prod-line zoo + HA 15-axis stop-loop
- Readiness/anti-crutch **write-deny** gates; fixed-errors product registry
- Tauri-as-axis stop-loops (keep independent-reviewer idea)
- Desktop product `desktop-*.mdc` cluster / Buildroot Dual SoT / FE type zoo gates
- Master-card / passport / canvas doc phase zoo

### Compare later (F1.5)

EliteEW strengths vs genEW (`09`): Commit Package, trash, cause triad, hard-acceptance stop-loop, living docs — vs genEW kernel-first artifacts, compose-not-invent, independent-work-audit, Auditor verify. **Union of mechanisms.**

---

## genEW orchestration forensic (2026-09-11)

**Source:** `../genEW/.cursor/` + git `d0e1bd8` → harden → north-star → independent-work-audit → fake-PASSED → fix-generator.

### What scaled volume + quality

1. **Kernel-first commit** (`d0e1bd8`): agents + skills + hooks + rules + blueprint + lifecycle + designs/reports **shipped with** the first golden design→code→verify cycle — not after a product flood.
2. **Four-layer plane:** Intelligence (agents) / Automation (skills) / Control (few failClosed hooks) / Knowledge (always-on rules) + **orchestration artifacts**.
3. **Lifecycle gate:** Intent → Plan → Implement → Verify → Evolve; FAILED → fix → verify again; no “ready” while verify skipped.
4. **Auditor verify:** deterministic FAIL beats LLM `verify_status: PASSED`; evidence > prose; narrative non-authoritative.
5. **North star + Done means + FAIL list:** always-apply; map/health-only ≠ done; Fake*/demo ≠ production path.
6. **Planes never conflated** (runtime / manage UI / launch UI / meta) — rewrite for OS: session / runtime / shell / meta.
7. **Compose, do not invent** — recipes/extracts before scratch (OS: packs + ORIGIN seed later).
8. **Skills = Task wrappers** (`disable-model-invocation: true`) pointing at `.cursor/agents/*.md` + parent checklists.
9. **Few hard hooks, advisory style** — security/secrets/(optional) remote/(later) generator; style-contract hints only.
10. **Independent stage audit** separate from per-module verify — READY / READY_WITH_DEBT / NOT_READY.
11. **Fix generator, not capsule** — root cause over symptom polish; escape hatch explicit.
12. **Context firewall** — ignore dumps/audit JSONL/probes/`trash/`; keep rules/skills/seed designs indexed.

### Anti-patterns proven painful (add to F1 AGENT-LESSONS)

| Anti-pattern | Why it hurts |
|--------------|--------------|
| Fake PASSED / DEMO_PASSED / WARN-as-PASS | Trust collapses; volume of junk “green” |
| Invent without extract / pack | Parallel products; drown; rewrite platform |
| Map/health-only as Done | Hollow modules; UX debt forever |
| Capsule / symptom hand-patch | Factory stays broken; next evolve fails again |
| Dual demo/canary delivery path | Honesty rot; Verifier lies |
| Markdown canon without machine gates | Forgotten in hours/days |
| Clone entire gate zoo day-1 | Gate fatigue → bypass urge |

### KEEP / HARVEST / DROP (genEW → OS) — draft for F1

**KEEP (mechanism):** lifecycle + artifacts; design-before-code; Auditor verify; north star; planes; role matrix; skills-as-Task; few failClosed + advisory style; independent-work-audit; production-path honesty; ignore firewall; fix-generator mindset.

**HARVEST (rewrite):** SYSTEM_BLUEPRINT → OS meta blueprint; ECC/Desktop planes → Shell/Runtime/Session; recipes → future OS packs; production-path → no fake foundation green; Tauri ≠ plan axis (already in 02).

**DROP day-1:** U1–U9/D1–D3 tax; Hands/EIC/Reflex path; absolute remote-block; Podman-only hooks; capsule protect allowlists; recipe/atom zoo; expert-consortium; assembly T/C/S freeze; wholesale `.cursor` clone from genEW or monorepo.

**REWRITE into F4 mini kernel:** ≤5–7 always-on rules; 4 agents; plan/implement/verify/evolve + stage audit; ≤5 failClosed hooks; designs/reports dirs; **no** product coding until F5 preflight green.

### Additional management lessons (from genEW arc)

16. **Kernel-with-example beats kernel-only or flood-first** — one golden cycle teaches anatomy; empty meta or product-without-meta both fail.
17. **Harden gates after first real use** (`394a406`) — ship thin, then fail-closed where agents cheat.
18. **Stage audit ≠ module verify** — need both or “P-wave done” becomes self-grading.
19. **Scorecard/golden registry** — pack without PASSED golden ≠ assembly-active (idea for later OS packs).
20. **Ignore lab noise aggressively** — otherwise Cursor context drowns in probe designs and Hands dumps.
