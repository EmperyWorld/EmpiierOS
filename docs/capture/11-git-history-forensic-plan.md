# Git history forensic plan — development & product failure modes

**Capture date:** 2026-09-11
**Status:** F1.7 **executed** 2026-09-13 (G1–G4 read-only). Findings below. Kinship trees not modified.
**Purpose:** Identify typical development and product errors/gaps from git history → integrate into the EmpiierOS foundation (lessons, gates, habits) to work **more qualitatively and productively**.

Companion plans:

- Agent methodology (rules/hooks/skills): [`10-monorepo-agent-forensic-plan.md`](10-monorepo-agent-forensic-plan.md)
- genEW orchestration harvest: [`09-genew-harvest.md`](09-genew-harvest.md)
- Lessons sink: [`05-lessons-to-harvest.md`](05-lessons-to-harvest.md)

---

## Hard constraints (non-negotiable)

| Allowed | Forbidden |
|---------|-----------|
| `git log` / `git show` / `git diff` / `git blame` (read) | `git commit`, `git push`, amend, rebase, reset --hard |
| Append findings **only** into this repo capture | Edits under `powered_by_emperyworld/` or `genEW/` |
| Quote short commit subjects / hashes as evidence | Rewriting history, touching working trees “for convenience” |
| Classify patterns → KEEP habit / HARVEST gate / DROP anti-pattern | Using forensic as an excuse to “clean up” living repos |

One agent session ↔ one workspace root when digging; EmpiierOS is the write home for notes.

---

## Repos in scope

| Repo | Path | Why |
|------|------|-----|
| **EliteEW monorepo** | `../powered_by_emperyworld` | Primary living product; agent+human commit culture; multi-door shotgun risk; Substrate Dual SoT scars |
| **genEW** | `../genEW` | High-velocity AI factory arc; fake-PASSED / debt / junk / fix-generator lessons already hinted in messages |

Optional later (parked): other kinship trees only if operator names them.

---

## What we are hunting (failure taxonomy)

Not a vanity “git archaeology.” Each finding must map to a **foundation control** (rule / hook / skill / preflight / habit).

### Development / agent-process failures

| Code | Pattern | What to look for in history |
|------|---------|-----------------------------|
| D1 | Shotgun / mega-commit | Many paths / products in one commit; huge file counts |
| D2 | Dirty mixed packages | docs+code+mobile+cloud+OS-adjacent in one package without scope |
| D3 | Autonomous / surprise git | Agent-shaped commits without Commit Package culture (infer from message style + size) |
| D4 | Symptom fix churn | Rapid fix→fix→fix on same area without cause language |
| D5 | Revert / oscillation | revert, undo, rollback, “restore”, flip-flop |
| D6 | WIP / tmp / hack residue | wip, tmp, hack, TODO-as-commit, “for now” |
| D7 | Docs lag or docs-only green | `docs:` claiming closeout without code; or code without doc same-phase |
| D8 | Gate / agent self-fight | commits that only patch hooks/gates after agents lied or bypassed |
| D9 | Cross-door drive-by | unrelated doors touched “while you're at it” |
| D10 | Identity / branch confusion | author/committer anomalies; force-push scars if visible; branch soup |

### Product / architecture failures

| Code | Pattern | What to look for |
|------|---------|------------------|
| P1 | Dual SoT | dual SoT, overlay vs live, two paths of truth (esp. Trusted Substrate / BR) |
| P2 | In-place rewrite of living | “for the future” / big rewrite on living host without parallel line |
| P3 | Fake readiness | fake PASSED, DEMO, stub green, scorecard theater |
| P4 | Framework-as-axis lock-in | planning/shipping through Tauri/Electron as destiny |
| P5 | Hollow Done | map/health-only, UI stub, missing verify evidence |
| P6 | Generator vs symptom | hand-patch capsules / products instead of factory (genEW) |
| P7 | Honesty / trust labels | payload_trust, from-source claims, policy SoT fights |
| P8 | Release churn without root cause | OTA bumps + fix storms without cause registry trail |
| P9 | Second-product temptation | parallel SPA / tree restores / DEWA-style gravity |
| P10 | Security/policy regressions | deny-all flips, policy SoT moves SPA↔Rust, egress fixes |

### Productivity killers (quality × speed)

| Code | Pattern | Foundation implication |
|------|---------|------------------------|
| Q1 | Context drown | huge commits → agent and human lose reviewability |
| Q2 | Gate fatigue → bypass | hooks patched to “anti-lie” after gaming |
| Q3 | Redo loops | same bug class repeated (fixed-errors registry missing/unused) |
| Q4 | Late isolation | OS/cloud/mobile mixed until painful — argues separate repo early |
| Q5 | Missing stage audit | “P-wave done” in commits without independent evidence |

---

## Study phases (F1.7 execution)

### G0 — Plan lock (this file)

- [x] Taxonomy + constraints + repos (2026-09-11)
- [x] Smoke signals from recent history (hypotheses only — see §Smoke below)

### G1 — Monorepo quantitative pass (read-only)

Commands (examples — adjust dates/paths as needed; **never** add `-i` interactive):

```bash
cd ../powered_by_emperyworld
git log --oneline -100
git log --pretty=format:'%h %ad %s' --date=short -200
# large commits by file churn
git log --pretty=format:'COMMIT %h %s' --numstat -100 | # then rank by files/lines
git log --oneline --grep='revert\|wip\|tmp\|hack\|undo\|rollback\|dual.so\|fake\|hotfix' -i
git log --oneline --grep='trusted-substrate\|Dual SoT\|dual SoT' -i | head -40
git log --oneline -- '.cursor/hooks/' | head -40
git log --oneline --all --grep='agent-gates\|anti-lie\|Gemma' -i | head -30
# door mixing: commits touching 2+ pack roots
git log --name-only --pretty=format:'COMMIT %h %s' -50
```

Exit G1: table of top 15 “expensive” commits + tagged D*/P*/Q* codes → append §Findings.

### G2 — Monorepo qualitative deep dives

Pick 8–12 representative commits (hash + why):

- 1–2 mega / multi-door
- 2–3 Trusted Substrate Dual SoT / overlay arcs
- 2–3 fix storms (same subsystem)
- 1–2 agent-gates / anti-lie / Gemma
- 1–2 docs-sync vs code lag
- 1 OTA/release churn cluster

For each: `git show --stat HASH` (read); note **root failure mode** + **foundation control** that would have helped.

### G3 — genEW quantitative + qualitative

```bash
cd ../genEW
git log --oneline --reverse | head -40   # bootstrap order
git log --oneline --grep='fake\|PASSED\|debt\|junk\|hack\|strip\|capsule\|generator' -i
git log --oneline -- '.cursor/' | head -40
```

Focus: kernel-first vs later debt; fake-PASSED closeouts; junk teardown; fix-generator commits; scorecard theater vs evidence.

### G4 — Cross-repo synthesis → foundation

Produce in EmpiierOS only:

1. **Anti-pattern catalog** (code → example hashes → why it hurt → control)
2. **Product scar catalog** (Dual SoT, policy SoT fights, hollow Done, …)
3. **Positive patterns to KEEP** (small scoped commits, cause language, debt closeout honesty, verify PASSED with evidence)
4. Feed into F1.6 `AGENT-LESSONS.md` + F3/F4/F5 design (shotgun budget, Commit Package, trash, stage audit, dual-SoT ban, parallel-line rule)

### G5 — Optional timeboxed deeper

Only if requested by the operator: author stats, monthly fix/feature ratio, and path-hotspot heatmap (`git log --follow` on known scar files). Still read-only.

---

## Smoke signals (2026-09-11 — hypotheses, not verdicts)

Quick non-destructive glance — **must be confirmed in G1–G3**; do not treat as a locked Source of Truth (SoT).

### Monorepo (`powered_by_emperyworld`)

- Dense **feat/fix/release** clusters on desktop/EBrowser (high productivity; also presents a fix-storm risk **D4/P8**).
- Explicit history language regarding **Trusted Substrate Dual SoT kill** (`29d6c5def` subject sample) → prime **P1** deep dive.
- `fix(agent-gates): anti-lie Gemma…` → **D8/Q2** (gates fighting agent lies).
- Policy SoT moves (SPA snapshot dropped; Rust owns navigation policy) → **P7/P10**.
- Frequent `docs(sync)` in lockstep with passport/master-card → living-docs culture is present; still check **D7** lag cases.
- At least one extremely large commit signal in the recent window (high file churn) → validate **D1/Q1**.

### genEW

- Message arc already encodes lessons: fake-PASSED debt, tear down junk, cure Hands factory instead of capsules, require `verify PASSED`, strip junk modules → **P3/P5/P6** gold for foundation vocabulary.
- Kernel-first `d0e1bd8` remains a positive counter-example (**KEEP**).

---

## Findings (append-only during F1.7)

_Format: `DATE | REPO | HASH | codes | one-line failure | foundation control`_

### G1–G2 monorepo (2026-09-13)

| DATE | REPO | HASH | codes | one-line | control |
|------|------|------|-------|----------|---------|
| 2026-09-13 | mono | `29d6c5def` | P1 D5 D8 | Dual SoT kill — fuse-repack vs BR rootfs | Single artifact SoT + adversarial dual FAIL |
| 2026-09-13 | mono | `f364829a5` | P1 P10 D9 | EBrowser policy Dual SoT — SPA snapshot vs Rust API | Policy SoT on Runtime/bridge only |
| 2026-09-13 | mono | `8a1631864` | D8 P7 Q2 | Anti-lie Gemma — first-line ОДОБРЕНО vs later ЛОПАТА | Honesty labels; no Gemma mandate |
| 2026-09-13 | mono | `00b879247` | D8 Q2 P7 | Live Gemma gate bolted after narrative ship | HA-light + evidence; DROP Gemma zoo |
| 2026-09-13 | mono | `f357fe295` | D1 D2 D9 P4 Q1 | Air enroll mega multi-door (Tauri+TS+edge+API) | Plane isolation; Tauri not plan axis |
| 2026-09-13 | mono | `68878d507` | D4 Q3 P6 | Frontend TS hygiene W4 mega-clear | Cause-first at write time; fix generator |
| 2026-09-13 | mono | `515eca923` | D4 Q3 | TS clear W3 storm | Same |
| 2026-09-13 | mono | `33e0b73d7` | D4 Q3 | TS clear W2 storm | Same |
| 2026-09-13 | mono | `66bcf0fce` | D7 P3 Q5 | docs(sync) lag vs living HEAD | Doc-same-phase |
| 2026-09-13 | mono | `b631f9c78` | D1 D2 D10 Q4 | Late pack-layout physical shared/ move | Early plane dirs; no mid-life rewrite |
| 2026-09-13 | mono | `cfab6224f` | D4 P10 | OTA assumed pkexec/dnf on metal TS | Substrate capability matrix |
| 2026-09-13 | mono | `d532e73ab` | D1 D7 P5 | Huge docs export pack as “product” | Foundation-before-product; plane budget |
| 2026-09-13 | mono | `1d227faa8` | P6 D8 KEEP | Anti-crutch NOT_CRUTCH for READY | Keep thin Crutch Test (R6) |
| 2026-09-13 | mono | `324882131` | D10 KEEP | Retire mirror `.cursor` SoT | One workspace root |

### G3 genEW (2026-09-13)

| DATE | REPO | HASH | codes | one-line | control |
|------|------|------|-------|----------|---------|
| 2026-09-13 | genEW | `d0e1bd8` | KEEP | Kernel-first orchestration + first cycle | Kernel-before-product flood |
| 2026-09-13 | genEW | `110360d` | P6 KEEP | Cure Hands factory, not hand-patch capsules | Fix generator (R6); no capsule zoo |
| 2026-09-13 | genEW | `1f56608` | P3 P5 P7 | Close evolve fake-PASSED debt | Honesty; evidence > LLM PASSED |
| 2026-09-13 | genEW | `91e6e37` | P3 P7 | Stop anatomy force PASSED | Verifier auditor stance |
| 2026-09-13 | genEW | `7dd3d00` | KEEP | Desktop host from EliteEW kernel extract | Compose/extract not invent |
| 2026-09-13 | genEW | `0265d33` | KEEP | Strip junk modules | Context firewall / trash |

### Top expensive monorepo commits (G1 sample)

`d532e73ab` (173f) · `af4b67669` (132f) · `f357fe295` (88f) · `68878d507` (67f) · `515eca923` (54f) · `9a945f031` (52f) · `b631f9c78` (~160f pack-layout)

---

## Exit criteria (F1.7 done)

- [x] G1–G3 executed read-only; notes only in EmpiierOS
- [x] ≥10 tagged anti-patterns with real hashes
- [x] ≥5 product scars with foundation mapping
- [x] ≥5 positive habits to KEEP
- [x] Synthesis folded into [`05`](05-lessons-to-harvest.md) + `AGENT-LESSONS.md`
- [x] Explicit list: which controls land in F3 / F4 / F5 (see §G4 below)
- [x] Zero modifications to kinship git trees

### G4 synthesis — controls by wave

| Control | F3 | F4 | F5 |
|---------|----|----|-----|
| Plane isolation / no multi-door shotgun | ignore product noise | R1 planes; plane budget | anti-flood / no product without unlock |
| Single SoT (no Dual) | — | honesty + axis rules | preflight substance; no Dual artifact paths |
| Doc-same-phase | — | R7 | status/docs coherence |
| No fake PASSED / DEMO | — | R8 | preflight exit 0 only = green |
| Cause-first / anti-crutch | — | R6 + skills | — |
| Fix generator not symptom | — | R6; harvest `110360d` idea | — |
| No Gemma mandatory | — | explicit light HA | **no** Gemma in preflight |
| Tauri not plan axis | — | R3 | — |
| Anti-gaming gates | — | R9 | adversarial restore on gate edits |
| One workspace root | ignore kinship mirrors | session-start + hygiene hook | KINSHIP path checks |

### Product scars (≥5)

1. Trusted Substrate Dual SoT (`29d6c5def` + fix storm)
2. EBrowser policy Dual SoT (`f364829a5`)
3. Air enroll multi-door mega (`f357fe295`)
4. Frontend TS debt hygiene waves (W2–W4)
5. OTA retail assumptions on metal (`cfab6224f`)
6. Docs/Current HEAD lag (`66bcf0fce`)
7. Agent gate arms race (Gemma → anti-lie)
8. genEW evolve fake-PASSED (`1f56608`, `91e6e37`)
9. Late pack-layout isolation (`b631f9c78`)

### Positive KEEP (≥5)

| Habit | Hash |
|-------|------|
| Cause/Symptom/Verification in commit bodies | `29d6c5def`, `8a1631864` |
| Kill Dual SoT with selftest | `29d6c5def` |
| Generator-not-capsule | `110360d` |
| Anti-crutch NOT_CRUTCH | `1d227faa8` |
| Refuse fake PASSED | `1f56608`, `91e6e37` |
| Kernel-first / extract not invent | `d0e1bd8`, `7dd3d00` |

**Note:** The genEW working tree had pre-existing dirty files — these were not created by F1.7.

---

## Relation to foundation waves

| Wave | How git forensic feeds it |
|------|---------------------------|
| F1.6 | Anti-patterns + KEEP habits into AGENT-LESSONS |
| F2 | Separate repo/identity motivated by D1/D9/Q4 evidence |
| F3 | Ignore/firewall against drowning from huge contexts |
| F4 | Shotgun budget, VPP, trash, cause-first, dual-SoT ban, parallel-line, stage audit |
| F5 | Preflight refuses “foundation green” without lessons file + anti-pattern list |
| F6+ | Seed policy: no in-place living rewrite (P2); allowlist informed by parked harvest (`12`) — no wholesale revival |

See [`04-foundation-waves.md`](04-foundation-waves.md) F1.7.
