# EmpiierOS — Agent kernel spec (mini `.cursor`)

**Status:** Living secure-reset spec — **9 alwaysApply rules · 1 glob rule · 5 hook bindings · 13 skills · 5 agents**.
**Date:** 2026-09-22
**Law:** Keep only controls with a named failure, bounded runtime, and a semantic
selftest. Counts are an observation, not a quota to game. This remains a curated
kernel, not a clone of the EliteEW or genEW automation meshes.
**Sources:** `AGENT-LESSONS.md`, `09-genew-harvest.md`, `10-monorepo-agent-forensic-plan.md` §E, `AGENT-KERNEL-BRIEF.md`, `LANGUAGE-BRIDGE.md`.

**Current inventory:**

| Asset | Current | Change rule |
|-------|--------:|-------------|
| alwaysApply rules | 9 | Merge/remove only when semantics remain explicit |
| Glob rules | 1 | `comfyui-media.mdc` — UI media via local ComfyUI; not alwaysApply |
| Hook bindings in `hooks.json` | 5 | Add only with failure case + selftest |
| Skills | 13 | Add only for a reusable workflow |
| Project agents | 5 | Add only for a distinct responsibility |

The language bridge is **disabled**. Do not restore Gemma intake/outtake unless
the operator explicitly asks. Advisory hooks must stay fail-open.

**Where Cursor loads this (docs: rules, skills, hooks, subagents):**

| Cursor looks here | EmpiierOS |
|-------------------|-----------|
| Project rules | `.cursor/rules/*.mdc` (frontmatter `alwaysApply` / `globs` / `description`) |
| Plain agent brief | root `AGENTS.md` (map only; law stays in the `.mdc` files) |
| Skills | `.cursor/skills/<name>/SKILL.md` with `name` + `description` |
| Subagents | `.cursor/agents/<name>.md` with `name` + `description` |
| Hooks | `.cursor/hooks.json` + `.cursor/hooks/*.py` |
| Extra ignore | `.cursorignore` (on top of `.gitignore`) |

There is **no** `.cursorrules`. That filename is not the current project-rules
format. Customize → Rules lists the `.mdc` files. Customize → Skills lists
folders whose `SKILL.md` has `name` and `description`.

---

## 1. KEEP → rules (alwaysApply unless noted)

| ID | Proposed file | Purpose | Origin |
|----|---------------|---------|--------|
| R1 | `os-north-star.mdc` | Planes: session / runtime / shell / meta; axis Shell+Runtime+Linux; twin risks; maturity = surface+op+policy | `00`/`01`, genEW north-star shape |
| R2 | `foundation-before-product.mdc` | No product UI/Runtime seed until preflight green (or operator override) | Lock |
| R3 | `tauri-not-plan-axis.mdc` | Living host kinship OK; no new caps as tauri command/plugin SoT | `02` |
| R4 | `verified-proposal.mdc` | No autonomous commit/push; Commit Package → human approve | EliteEW VPP KEEP |
| R5 | `trash-not-rm.mdc` | Soft-delete via `scripts/trash-move.sh`; ban casual `rm -rf` / Delete tool for tracked trees | EliteEW trash KEEP |
| R6 | `cause-first-and-anti-crutch.mdc` | Cause\|Lock\|Measure; no symptom-only closeout; light Crutch Test | EliteEW triad KEEP (merged to avoid rule count) |
| R7 | `doc-same-phase.mdc` | Contract/code change → docs same phase; no docs-only fake green | EliteEW living docs KEEP |
| R8 | `honesty-and-acceptance-light.mdc` | No DEMO_PASSED / fake foundation green; HA light; **Language bridge** section (not auditor) | production-path + HA light + bridge |
| R9 | `anti-gaming-preflight.mdc` | Never delete/soften preflight checks to force PASS; fix artifacts; adversarial restore required on gate edits | curator false-green history |

**Optional globs (not alwaysApply):** `comfyui-media.mdc` (`shell-ui/**`,
`runtime/shell-gui/**`) — generate UI media on local ComfyUI `:31003`.
Still later: `file-size-modularity.mdc`, `seed-origin.mdc`.

---

## 2. DROP (do not port day-1)

| Drop | Why |
|------|-----|
| Cloud deploy / prod-line / eeprodbaked zoo | Wrong product |
| Mobile / EWHome / Site Studio / UGS / EWA door rules | Wrong doors |
| Keycloak encyclopedia always-on | Drown |
| Full `domain_matrix` / universe LAN atlas | Rewrite 2–4 OS domains later if needed |
| Gemma mandatory auditor + stop-loop | Empty repo false blockers |
| Tauri security stop-loop / F-01…F-13 mesh | Pattern only → independent reviewer later |
| Buildroot Dual SoT gates | Substrate stays in monorepo |
| FE `as-unknown-as` / OpenAPI SSOT gates | Until shell-ui stack exists |
| Hands / EIC / recipe / U1–U9 / D1–D3 tax | genEW factory, not OS kernel |
| Absolute block-remote-send | Policy toggle later if needed |
| Master-card / passport / canvas phase zoo | Optional F7+ |
| Full expert-consortium / product audit skills | Later |

---

## 3. HARVEST → rewrite (mechanisms, new names)

| Mechanism | OS rewrite |
|-----------|------------|
| Pack / door boundaries | OS planes in R1; one session ↔ one root |
| Desktop Kernel vs App | Shell vs OS apps (manifests) — docs later |
| Independent stage audit | Skill `independent-work-audit` |
| Design → implement → verify | Skills + agents (genEW shape) |
| Auditor verify (evidence > LLM PASSED) | Agent `verifier` + skill `verify` |
| Fix generator not symptom | Fold into R6 until a factory exists |
| Capability harvest from trash | Skill stub `capability-harvest` (read-only) — post F1.8/F1.9 |
| Inquisitor adversarial review | Optional 5th agent later |
| Session-start / steward | Skills `session-start`, `cursor-steward` (thin) |
| Gemma RU↔EN bridge | **Disabled** 2026-09-17 — hook/skill/command off-disk; see `LANGUAGE-BRIDGE.md` |

---

## 4. Skills

| Skill | Role | Notes |
|-------|------|-------|
| `plan` | Architect → design artifact | **on disk**; `disable-model-invocation: true` |
| `implement` | Implementer from design + **honesty-stub wave** | **on disk**; former `honesty-stub-wave` folded here |
| `verify` | Auditor gate; report PASSED/FAILED | **on disk**; evidence > prose |
| `evolve` | Delta loop plan→implement→verify | **on disk** |
| `independent-work-audit` | Stage READY / READY_WITH_DEBT / NOT_READY | **on disk** |
| `cause-first` | Cause workflow | **on disk** |
| `verified-proposal` | Commit Package helper | **on disk** |
| `session-start` | Preflight reminder + intake | **on disk** |
| `cursor-steward` | Audit local `.cursor` vs this SPEC | **on disk** |
| `hard-acceptance-light` | Evidence checklist before done | **on disk** |
| `capability-harvest` | Read-only map from kinship/trash | **on disk**; harvest ≠ restore |
| `preflight-foundation` | Run/explain `scripts/preflight-foundation.sh` | **on disk** |
| `comfyui-media` | Local ComfyUI icons/foley for Shell and native chrome | **on disk**; RTX 3090 on `:31003` |

**Now:** 13 skills. Former `language-bridge` skill is removed. Count changes
require a concrete reusable workflow, not a quota exception.

---

## 5. Agents (5)

| Agent | Owns | Must not |
|-------|------|----------|
| `architect` | designs under orchestration | Production code |
| `implementer` | code/docs per design | Declare foundation/product “ready” |
| `verifier` | reports; fail-closed | Upgrade FAILED→PASSED without evidence |
| `refactorer` | structure/debt | New features |
| `inquisitor` | readonly adversarial challenge reports | Write product code; greenwash FAIL |

**Now:** 5 agents (4 core + inquisitor). Do not grow further without raising this SPEC.

---

## 6. Hooks

One small guard owns fail-closed policy. Duplicate bindings and synchronous
acceptance work inside stop hooks are forbidden because they previously bricked
the editor while providing no security boundary.

| Event | Script | Mode | Purpose |
|-------|--------|------|---------|
| `beforeShellExecution` | `policy_guard.py shell` | failClosed, 5s | Hard deletes, destructive Git, unapproved Git mutation |
| `preToolUse` | `policy_guard.py tool` | failClosed, 5s | Delete tools, secret-like writes, nested Shell payloads |
| `beforeReadFile` | `policy_guard.py read` | failClosed, 5s | Secret-like reads with public/example exceptions |
| `beforeSubmitPrompt` | `session-root-hygiene.py` | fail-open | Root warn only |
| `stop` | `foundation-completion-hint.py` | fail-open | Static evidence reminder |

Malformed JSON is denied by `policy_guard.py`; advisory hooks fail open.
`foundation-completion-hint.py` never executes preflight itself.
**Explicitly not day-1:** gemma-gate senior auditor, language-bridge hook, tauri-security-gate, deploy-line-gate, FE hygiene mesh, new afterFileEdit binding.

---

## 7. Orchestration artifacts

| Path | Purpose |
|------|---------|
| `SYSTEM_BLUEPRINT.md` | Short: Intent→Plan→Implement→Verify→Evolve for **this** repo |
| `lifecycle.json` | Machine map skill→agent→outputs (v7; 13 skills) |
| `designs/` | Architect outputs |
| `reports/` | Verifier / audit stage reports |
| `audit/` | Hook JSONL (ignored) |
| `.cursor/commands/` | **Absent** — empty slash-command dir was removed 2026-09-18 |

---

## 8. Provisional vs hardened

| Phase | Kernel state |
|-------|----------------|
| Historical baseline | Rules, skills, agents, and hooks established |
| Secure reset | Single fail-closed policy guard + semantic selftest |
| Acceptance | Preflight executes tests/build/smokes outside hooks |
| Language bridge | **Disabled** — `LANGUAGE-BRIDGE.md` |

---

## 9. Anti-goals

- Do not copy monorepo `hooks.json` wholesale.  
- Do not enable Gemma/Tauri **auditor** stop loops.  
- Do not add product-door skills (Keycloak, mobile, deploy-nvme, …).  
- Do not optimize for an asset count instead of a concrete failure mode.
