# Foundation waves (not product waves)

**Capture date:** 2026-09-11
**Principle:** Multiple waves for creating the foundation — consciously code the product without rushing.
A single W0 is not enough. This is not task spreading, but the load-bearing structure.

**Unified plan SoT (2026-09-11):** [`../foundation/FOUNDATION-MASTER-PLAN.md`](../foundation/FOUNDATION-MASTER-PLAN.md) (+ companion specs in `docs/foundation/`). This file (`04`) remains detailed F1-track notes; the master plan owns order/exits/checklist.

## Meta rules for these waves

- Wave Exit = a **verifiable** artifact (preflight / checklist / files exist), not "write markdown and forget."
- Product UI / seed React / Substrate payload profile — **after** the agreed foundation.
- Spend as much time as necessary.
- Lower waves are **drafts**; we will split and supplement them in `07-open-capture.md`.

---

## F0 — Capture & intent lock (this folder)

**Goal:** All critical thinking is recorded; it will not be lost during chat transitions.

**Exit (draft → synced 2026-09-13):**

- [x] `capture` directory created
- [x] Operator: intent lock 2026-09-11 (axis / two failures / method / repo / foundation; isolation min vs max) — see `00`
- [x] Open questions are tracked in `07` (naming/host residual — parked, not blocker)
- [x] Origin arc "how a separate repo was born" is recorded in `00`
- [x] Maturity criterion + twin risks in `01`
- [x] **WHERE OS LIVES** → `docs/WHERE-OS-LIVES.md` (F7 thin, 2026-09-13)
- [x] Local `git init` + baseline commits (Task 13 + 2026-09-13) — **no remote** yet

---

## F1 — Lesson forensic (monorepo + genEW → OS kernel)

**Goal:** Extract experience from rules / hooks / skills / git history: **development management methodologies with the Cursor AI agent** (not product law).

**Sources (read-only):**

- **`../powered_by_emperyworld/.cursor/`** — living EliteEW agent OS (rules/hooks/skills/agents). Study plan: [`10-monorepo-agent-forensic-plan.md`](10-monorepo-agent-forensic-plan.md).
- **`../genEW`** — Cursor control plane that scaled quality: [`09-genew-harvest.md`](09-genew-harvest.md).
- **Git history (read-only)** — monorepo + genEW: typical development/product errors → foundation. Plan: [`11-git-history-forensic-plan.md`](11-git-history-forensic-plan.md).
- **Trash + parked products (read-only)** — ops `trash/` + soft-retired **DEWA / EWPower / EWGen / …** as working exemplars. Plan: [`12-trash-and-parked-products-forensic-plan.md`](12-trash-and-parked-products-forensic-plan.md).
- **Security + Tauri launcher + open finds (read-only)** — living security system, launcher/host exemplar, opportunistic useful findings. Plan: [`13-security-tauri-launcher-and-open-harvest-plan.md`](13-security-tauri-launcher-and-open-harvest-plan.md). Stance: study ≠ plan-through-Tauri ([`02`](02-tauri-stance.md)).

### F1 sub-stages (do not skip)

| Sub | Name | Status |
|-----|------|--------|
| F1.0 | genEW harvest | [x] `09` + notes in `05` (2026-09-11) |
| F1.1 | Monorepo inventory + study plan | [x] `10` (2026-09-11) — counts/clusters/phases |
| F1.2 | Monorepo management-core deep read (Phase B) | [x] 2026-09-13 — notes in `10` §E; cause triad / HA / VPP / trash / living docs / steward / inquisitor / doors→planes |
| F1.3 | Monorepo hooks deep read (Phase C) | [x] 2026-09-13 — thin hooks shortlist in `10` §E; 61→≤8; trash-not-rm = next gap |
| F1.4 | Skills workflow deep read (Phase D) | [x] 2026-09-13 — pipeline map in `10` §E; no eliteew-implement; harvest plan-canon → future `plan` |
| F1.5 | Compare genEW ↔ EliteEW mechanisms (Phase E) | [x] 2026-09-13 — union in `10` §E; genEW shape + EliteEW VPP/trash/cause; caps ≤9/≤8/≤12 |
| F1.6 | AGENT-LESSONS + KEEP/HARVEST/DROP/REWRITE (Phase F) | [x] 2026-09-13 — lessons + A1–A15 + `AGENT-KERNEL-BRIEF.md`; `11`–`13` pointer-only then |
| F1.7 | Git history forensic (read-only) | [x] 2026-09-13 — G1–G4 in `11`; Dual SoT `29d6c5def`; mega `f357fe295`; genEW `d0e1bd8`/`110360d` |
| F1.8 | Monorepo `trash/` forensic (ops trash-move) | [x] 2026-09-13 — T0–T1 in `12`; ~140 buckets / 4G; Kind A lessons |
| F1.9 | Parked products study (DEWA, EWPower, …) | [x] 2026-09-13 — T2–T3 in `12`; soft-retire=0-file skeletons; EWGen via Documents/genEW |
| F1.10 | Security system study | [x] 2026-09-13 — Track S in `13`; Runtime ACL / policy SoT / F8 inputs |
| F1.11 | Tauri launcher / living host study | [x] 2026-09-13 — Track L in `13`; journey→Shell/Runtime map; do-not-break list |
| F1.12 | Open useful-findings mandate | [x] 2026-09-13 — Track O open log (≥10 FOUND); lessons #38–42 |

**F1.7 detail (git forensic):**

- **Goal:** typical development + product errors and gaps from `git log`/`show` → anti-pattern catalog + positive habits → controls for F3–F5.
- **Scope:** monorepo + genEW. Write findings **only** into EmpiierOS.
- **Taxonomy:** D* (process), P* (product/architecture), Q* (productivity) — see `11`.
- **Sub-phases:** G0 plan ✓ → G1–G4 **done 2026-09-13** (findings in `11`).
- **Forbidden:** commit/push/rebase/reset/amend/config; any "cleanup" of living trees.

**F1.8–F1.9 detail (trash + parked products):**

- **SoT plan:** [`12-trash-and-parked-products-forensic-plan.md`](12-trash-and-parked-products-forensic-plan.md).
- **F1.8 / Kind A:** monorepo `trash/YYYYMMDD-…` — how `trash-not-rm` works in practice; what agents delete; lessons for F3 ignore + F4.
- **F1.9 / Kind B:** soft-retired **DEWA, EWPower, EWGen, EWC, EWCoderOrchestrator, SSFEW, …** — not crossed out, but **parked working tools**; study them as visual working systems → conclusions into the foundation.
- **Law:** capability harvest / learn — **never** restore whole trees as second products (anti-DEWA / anti-second-cloud).
- **Locate (T0 done):** soft-retire batch under `~/Документы/trash/20260718-015129/` = **0-file skeletons**; ops trash = monorepo `trash/` (~140 / 4G); full EWGen = `Документы/EWGen` + genEW.
- **Forbidden:** `cp -a` revive to monorepo/EmpiierOS roots; second Keycloak/cloud; mutating parked trees.

**F1.10–F1.12 detail (security + launcher + open finds):**

- **SoT plan:** [`13-security-tauri-launcher-and-open-harvest-plan.md`](13-security-tauri-launcher-and-open-harvest-plan.md).
- **F1.10 / Track S:** security system (boundaries, capabilities, enroll, FoFline, Substrate, EBrowser policy) → OS Runtime/session defaults; later F8.
- **F1.11 / Track L:** Tauri **launcher** + living `connection-agent` host — a strong exemplar; rewrite map → OS Shell session + Runtime; **do not break**; Tauri is **not** the OS of future plans (`02`).
- **F1.12 / Track O:** open mandate for implementers — log everything useful for the development foundation (`FOUND | …`); triage into AGENT-LESSONS or product-later; without product coding.
- **Forbidden:** in-place Tauri "preparation for OS"; design new OS caps as Tauri commands/plugins.

**Exit (draft):**

- [x] Deep reads F1.2–F1.6 (monorepo agent forensic B–F)
- [x] Execution F1.7–F1.12
- [x] Trash + parked-product notes — F1.8–F1.9 (`12`)
- [x] Security one-pager + launcher rewrite map — F1.10–F1.12 (`13`)

**F1 constraints:**

- Goal = **management methodologies** + history/trash/parked/security/launcher lessons + opportunistic finds — not a clone of 85 rules / whole Tauri host / DEWA trees.
- Anti-patterns must include: fake green, symptom-only, autonomous git, `rm`, doc lag, gate fatigue, cross-door, invent-without-extract, map-only done, Dual SoT, shotgun, in-place living rewrite, second-universe revive, **framework-as-plan-axis (Tauri)**.
- Cap for later F4: alwaysApply ≤ ~8–12; hooks ≤ ~8–12; management skills ≤ ~12.

---

## F2 — Identity & repo skeleton (empty product)

**Goal:** Physical OS line contour without product code.

**Isolation (from operator lock 2026-09-11):**

- F2 **can start** with: existing EmpiierOS path + later `git init` under current user when operator says go.
- **Separate Linux user** = optional stretch in F2 / follow-on — desirable max, **not** required to green F0–F1 or to begin ignore/skeleton work once operator opens F2.

**Exit (historical — done on disk; living SoT = master plan):**

- [x] Repo path this repo + local git (Task 13); remote deferred
- [x] Git identity local `EmpiierOS <empiieros@local>`
- [x] Skeleton dirs: `shell-ui/`, `runtime/`, `session/`, `docs/`, `.cursor/`
- [x] Under `.cursor/`: agents/skills/rules/hooks/orchestration + blueprint/lifecycle stubs
- [x] README + KINSHIP
- [x] Capture under `docs/capture/`

---

## F3 — Ignore & context firewall

**Goal:** Cursor/git ignore as the first line of defense against clutter.

**Exit (historical — done):**

- [x] `.gitignore` / `.cursorignore` / `.cursorindexingignore`
- [x] Preflight fails if ignores are missing or weak
- [x] Rule: one agent session ↔ one workspace root (R1 + KINSHIP)
- [x] Ignore ideas harvested (secrets/trash/audit noise out; meta in)

---

## F4 — Agent kernel (mini `.cursor`, not full EliteEW/genEW clone)

**Goal:** A small, rigid agent OS for EliteEW OS.

**Shape target (from genEW harvest — adapt, don’t clone):**

| Piece | EmpiierOS lean |
|-------|----------------|
| Rules always-on | ≤5–7: OS north-star planes · honesty/no-fake-green · cause-first / fix-generator-not-symptom · verified proposal · trash-not-rm · foundation-before-product · Tauri not plan axis |
| Agents | 4: architect / implementer / verifier / refactorer (ownership matrix); no generator-root-cause until a factory exists |
| Skills | plan · implement · verify · evolve · independent-work-audit; skills = Task wrappers + checklists |
| Hooks | ≤5 failClosed (destructive shell, secrets, optional remote policy, trash-not-rm); **style advisory only** |
| Artifacts | `orchestration/designs|reports|audit` + blueprint/lifecycle stubs |

**Exit (F4 complete 2026-09-13 — lifecycle skills on disk):**

- [x] Curated rules R1–R9
- [x] 3 failClosed hooks day-1 (shortlist ≤8 in `10` §E; `trash-not-rm` still next)
- [x] Explicit: Tauri not normative planning axis (R3)
- [x] Explicit: no product coding before foundation preflight green (R2)
- [x] DROP list in AGENT-LESSONS / SPEC
- [x] Verifier stance: evidence > LLM PASSED (agent stub + HA-light)
- [x] Thin F4 remaining: `trash-not-rm` hook, `plan` skill, session-root + completion advisory — **done 2026-09-13**
- [x] Lifecycle skills: `implement` / `verify` / `evolve` / `independent-work-audit` — **done 2026-09-13** (11/12; `capability-harvest` deferred)

---

## F5 — Durable foundation preflight

**Goal:** A single command that proves the foundation "holds itself up."

**Exit (done):**

- [x] `scripts/preflight-foundation.sh`
- [x] Checks: skeleton, ignores, lessons, kinship, agent kernel, seed policy, no product flood
- [x] Fails closed on missing pieces / anti-gaming
- [x] Optional: `/independent-work-audit` before product waves — skill **on disk 2026-09-13**
- [x] Operator accepts: "it is possible to live in this repo for weeks without product code"

---

## F6 — Seed policy (still not full UI product)

**Goal:** Copy-paste-edit rules until the first seed.

**Exit (done — policy only):**

- [x] ORIGIN.md template
- [x] Allowlist / denylist in `docs/seed/SEED-POLICY.md`
- [x] Conscious duplication until OS Shell contract is stable
- [x] No actual copy yet — preflight blocks product flood

---

## F7+ — later foundation (parked slots)

Split as needed, for example:

- F7 — Documentation map & writing rules for OS line
  - [x] WHERE OS LIVES + NORTH-STAR (2026-09-13)
  - [x] OS Shell v0 SoT one-pager → `docs/OS-SHELL-V0-SOT.md` (2026-09-13)
  - [ ] Fuller docs map / writing rules — optional later
- F8 — Security/privacy defaults for OS repo — [x] **2026-09-13** → [`../foundation/SECURITY-PRIVACY-DEFAULTS.md`](../foundation/SECURITY-PRIVACY-DEFAULTS.md)
- F9 — Release/signing identity stubs (no secrets) — [x] **2026-09-13** → [`../foundation/RELEASE-SIGNING-STUBS.md`](../foundation/RELEASE-SIGNING-STUBS.md)
- F10 — Substrate kinship contract — [x] **2026-09-13** → [`../foundation/SUBSTRATE-KINSHIP.md`](../foundation/SUBSTRATE-KINSHIP.md) (dual-flag; no kiosk break; no BR import)

### Product waves (AFTER foundation acceptance — sketch only, not started)

Origin dialogue phases — **not** foundation F0–F5; reminder of the order after preflight green:

| Phase | Do | Don't |
|-------|----|-------|
| Seed copy | Narrow UI copy into OS tree | Refactoring "for the future" while living |
| Edit only copy | OS Shell mode, no browser assumptions | Hot imports from living FE as SoT |
| Thin runtime | Own bridge; start read-only status | Port seat/WG/Capsule from Tauri wholesale |
| Substrate profile | Second image profile + dual-flag | Flip default kiosk off Tauri |
| Later | Soft-retire old host on journey parity | Big-bang cutover |

Product waves (shell seed, runtime stub, substrate profile) — **separate plan after** foundation acceptance.

---

## Reminder

Until the foundation preflight is green — **you cannot** "start a little bit of UI, then we'll finish the rules later."
