# EmpiierOS — File size & modularity

**Status:** Spec for humans + agent advisory hook.  
**Date:** 2026-09-11  
**Goal:** Anti-god-file; clear homes for docs vs plans vs specs; modular planes.

---

## 1. Soft LOC limits (split triggers)

| Kind | Soft max | Hard warn | Action when exceeded |
|------|----------|-----------|----------------------|
| Rule `.mdc` | 120 lines | 200 | Split theme or extract reference.md |
| Skill `SKILL.md` | 150 lines | 250 | Move detail to `references/` |
| Agent `.md` | 120 lines | 180 | Trim ownership matrix; link blueprint |
| Spec / plan `.md` | 250 lines | 400 | Split by section into sibling files; index from parent. Exception: `docs/plans/SECURE-RESET-LIVE-DIARY.md` is an append-only lab log. |
| Capture thematic (`00`–`08` style) | 200 lines | 350 | Append-only OK; spin new dated file if topic explodes |
| Hook `.py` | 150 lines | 250 | Shared `hooks/lib/` |
| Plane `README.md` | 80 lines | 120 | Keep stub; detail → docs/ |
| Product source file (later) | 300 LOC | 500 | Split module; no god-components |

These are **discipline limits**, not CI blockers day-1. F5 may **warn**; later can failClosed on hard warn for `.cursor/` only.

---

## 2. One job per file

| Symptom | Fix |
|---------|-----|
| File mixes product law + git policy + UI tokens | Split by concern |
| Rule restates entire monorepo | Link kinship; keep OS-only |
| Plan duplicates capture verbatim | Point to `docs/capture/`; keep deltas in foundation specs |
| Hook copies another hook’s logic | `hooks/lib/` |

---

## 3. Where things live

| Content | Home | Not |
|---------|------|-----|
| Locked product/intent thinking (historical) | `docs/capture/` | Root long-term |
| Executable foundation plan | `docs/foundation/` | Chat-only |
| Active short plans | `docs/plans/` or `.cursor/plans/` | Product code dirs |
| Agent law | `.cursor/rules/` | `docs/` only as mirrors |
| Agent workflows | `.cursor/skills/` | Inline in rules |
| Human north-star short | `docs/NORTH-STAR.md` | 50-page essay |
| Lessons | `docs/AGENT-LESSONS.md` | Scattered chat |
| Seed policy | `docs/seed/` | `shell-ui/` without ORIGIN |
| Product UI | `shell-ui/` | `docs/`, `.cursor/` |
| Bridge | `runtime/` | FE |
| Session contracts | `session/` | Full Substrate tree |

---

## 4. Naming conventions

- Specs: `SCREAMING-KEBAB` or `Title-Case` for foundation pack — keep pack consistent (`FOUNDATION-MASTER-PLAN.md`).  
- Capture: keep `NN-topic.md` under `docs/capture/`.  
- Rules: `kebab-case.mdc`.  
- Skills: directory `kebab-case/SKILL.md`.  
- Dates in open capture: `YYYY-MM-DD` headings.

---

## 5. Modularity for planes (later product)

When product code arrives:

- `shell-ui/` — app modules by surface; shared tokens in `shell-ui/src/theme/` (example).  
- `runtime/` — crates/packages by capability; no UI imports.  
- Cross-plane contracts live in `docs/` or `session/contracts/` — not duplicated ad hoc.

No early monorepo shared package (conscious duplication) — see F6.

---

## 6. Anti-patterns

- Root accumulating 30+ markdown files of mixed purpose.  
- Single `RULES.md` that tries to be entire EliteEW.  
- Pasting SESSION-DIALOGUE into alwaysApply rules.  
- “Temporary” mega-files without split date.

---

## 7. Agent enforcement (advisory)

Hook `style-modularity-hint.py` (AGENT-KERNEL-SPEC): after large edits, if line count > hard warn for path kind → `agentMessage` reminder to split. Not failClosed day-1.

---

## 8. Preflight (optional checks)

- Warn if any `.cursor/rules/*.mdc` > 200 lines.  
- Fail if `docs/foundation/` (or `foundation/`) missing master plan.  
- Fail if `shell-ui/` contains `node_modules` committed.
