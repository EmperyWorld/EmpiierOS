# genEW harvest — agent orchestration that scaled quality

**Capture date:** 2026-09-11
**Source (read-only):** `../genEW/` — especially `.cursor/` + git from `d0e1bd8` onward
**Status:** Working harvest for EliteEW OS foundation. **F1.5 (2026-09-13):** folded into [`10`](10-monorepo-agent-forensic-plan.md) §E union vs monorepo. **Not** a mandate to clone genEW. **Not** product coding.

---

## Why this kinship

genEW (Empery World / EWorld) is where we already proved that a **Cursor-native control plane** (rules + hooks + skills + subagents + orchestration artifacts) can generate **large volume at high structural quality** when locked early — instead of improvising module-by-module in chat.

EmpiierOS harvests **mechanisms**, not the product zoo (ECC Hands, recipes catalog, Desktop mounts, Capsule factory).

---

## A) Origin story (compressed chronology)

| When | Commit / move | What mattered |
|------|---------------|---------------|
| 2026-07-20 | `d0e1bd8` *initialize empery orchestration layer and base architecture* | **Kernel first, with one reference cycle.** Same commit: agents + skills + hooks + rules + `SYSTEM_BLUEPRINT` + `lifecycle.json` + `designs/` + `reports/` **and** first anatomy (`core/` + `core_auth` + analytics design/verify). Orchestration was not bolted on after a product flood. |
| Same day | Trinity / ECC / EIC / Podman | Product fabric grew **on top of** the meta-plane, not instead of it. |
| 2026-07-23 | `394a406` / `f843024` *harden orchestration* | Hooks → Python + shared lib; secrets protect; **R/U gates fail-closed**; omitted `ui` ≡ required; verify **evidence-based**. |
| Later | North-star rule always-on | *Compose services… not invent parallel products.* Manage env (ECC) vs Launch env (Desktop) never conflated. |
| Later | `independent-work-audit` | Stage-gate skeptic **separate** from per-module `/verify`. |
| P4–P19 | Factory maturity | Golden packs, `forbid_scratch`, evolve delta gates, **fake-PASSED** closeout, then *fix the generator, not the capsule* + capsule-protect hook. |

**Bootstrap order that worked:**

```text
blueprint + role split + design/verify dirs + few always-on laws
  → one golden design→code→verify cycle
  → north star + factory DoD
  → deterministic gates before trusting LLM verify_status
  → THEN volume (capsules / packs)
```

---

## B) Four-layer control plane (keep the shape)

| Layer | Location (genEW) | Role |
|-------|------------------|------|
| **Intelligence** | `.cursor/agents/` | Architect, Implementer, Verifier, Refactorer (+ later generator-root-cause) |
| **Automation** | `.cursor/skills/` | `/plan` `/implement` `/verify` `/evolve` (+ create-service wrappers, audits) |
| **Control** | `hooks.json` + `.cursor/hooks/` | Few **failClosed** denies; style advisory; audit JSONL |
| **Knowledge** | `.cursor/rules/*.mdc` | Always-on planes, DoD, production-path honesty, north star |
| **Artifacts** | `.cursor/orchestration/` | Blueprint, designs, reports, audit, recipes (recipes = product factory; defer for OS) |

Companion machine map: `lifecycle.json` — skills declare agent, inputs, outputs, next gate.

### Lifecycle (normative shape for OS later)

```text
Intent → Plan → Implement → Verify → (Evolve/Refactor) → Ready
         ↑___________________________________|
                    FAILED → fix → verify again
```

No “ready” while Verify is FAILED or skipped.

---

## C) Highest-leverage patterns (ranked)

1. **Traceable lifecycle with artifacts** — design md/json under `designs/`; verify report under `reports/` with explicit `PASSED`/`FAILED`. Human can reconstruct intent → build → compliance.
2. **Design before code** — Architect owns scope; Implementer may not invent requirements.
3. **Verifier as Auditor, not cheerleader** — *Deterministic FAIL always wins over LLM `verify_status: PASSED`.* Narrative is non-authoritative. Evidence > prose.
4. **North star + Done means + explicit FAIL list** — always-apply product goal; map/health-only ≠ done; Fake*/demo path ≠ production-ready.
5. **Planes never conflated** — genEW: Runtime / Management UI / Launch UI / Meta. EmpiierOS rewrite: **Linux session / Runtime bridge / OS Shell / Meta (.cursor)**.
6. **Factory law: compose, do not invent** — recipes/extracts/templates before scratch; generic CRUD last resort. For OS: future packs + monorepo seed policy — not day-1 catalog.
7. **Role ownership matrix** — Architect=design only; Implementer=code ≠ “ready”; Verifier=gate; Refactorer=structure/debt, not features.
8. **Skills as Task wrappers** — `disable-model-invocation: true`; parent checklist; Task prompt points at `.cursor/agents/*.md`. Skills orchestrate; they don’t hide business logic.
9. **Few failClosed hooks + advisory style** — hard deny: destructive shell, secrets write, (optional) remote publish, (later) generator/capsule protect. Style-contract = hints after edit, not 50 blockers.
10. **Independent stage audit** — READY / READY_WITH_DEBT / NOT_READY; BLOCK/DEBT/DRIFT; plan todo green ≠ evidence.
11. **Fix the generator, not the symptom** — *treat the cold, do not wipe the patient’s nose.* Escape hatch only with explicit operator file + intent.
12. **Context firewall** — ignore Hands dumps, audit JSONL, probe designs/reports, `trash/`; **keep** rules/skills/agents/seed designs indexed.
13. **Honesty / production-path only** — never `DEMO_PASSED`, dual canary track as delivery story, Fake* in `src/`.
14. **Config-driven assembly (later idea)** — LLM emits Blueprint+Manifest; machine owns inject; gates authoritative — harvest *idea*, not genEW schemas wholesale.

### Law phrases worth EmpiierOS vocabulary

- Compose, do not invent.
- Map/health alone is not done.
- Evidence > prose.
- Treat the cold — do not wipe the patient’s nose.
- Unregistered / unproven does not exist.
- Never DEMO_PASSED / fake foundation green.
- Deterministic FAIL beats LLM PASSED.

---

## D) What NOT to copy day-1 into EmpiierOS

| genEW pattern | Why drop / soften |
|---------------|-------------------|
| Full U1–U9 + D1–D3 UI tax on every module | No product flood in F0–F5; drown empty foundation |
| Hands / EIC / Reflex Autonomous as default path | Product factory, not OS kernel |
| Absolute `block-remote-send` | Lab lock; OS may need remotes — policy toggle later |
| Podman-only / docker-deny | Runtime TBD for OS line |
| Capsule protect allowlists | Only after a generator exists |
| Recipe + EliteEW extract + atom_catalog zoo | Harvest compose *idea*; not the catalog |
| Expert-consortium product audit | Later; not F1–F4 |
| Assembly freeze Gates T/C/S | Overkill until config-driven assembly exists |
| Always-on `service-ui` / `module-contract` globs | Rewrite to OS planes when modules exist |

**Gate fatigue:** genEW accumulated many R/U/D gates and stayed useful because **hard fails stayed few** and style stayed advisory. EmpiierOS F4 = mini kernel, not full clone.

---

## E) EmpiierOS rewrite map

| genEW | EmpiierOS (target rewrite) |
|-------|----------------------------|
| ECC manage / Desktop launch | OS Shell UI / OS app surfaces |
| `services/*` anatomy | `runtime/` (and later session) modules — when they exist |
| recipes + EliteEW extract | post-F5 OS packs + monorepo ORIGIN seed |
| Hands capsule protect | N/A until a generator exists |
| `production-path.mdc` | honesty / no fake foundation green |
| `empery-north-star.mdc` | OS north star: Shell + Runtime + Linux; Tauri not plan axis |
| `independent-work-audit` | F-wave stage gates / foundation acceptance |
| `SYSTEM_BLUEPRINT` + `lifecycle.json` | OS meta blueprint (stub in F2/F4) |
| Plan→Implement→Verify→Evolve | Same shape for foundation *and* later product waves |

---

## F) Recommended capture actions (still brainstorm — no wave execution yet)

1. Fold this file into F1 exit → `AGENT-LESSONS.md` (mechanism lessons, not slogans).
2. F2/F4: empty `.cursor/{agents,skills,rules,hooks,orchestration/{designs,reports,audit}}` + blueprint stub **before** product tree.
3. F3: ignore firewall inspired by genEW `.cursorignore` ideas (dumps out; seed/meta in).
4. F4: ≤5–7 always-on rules; 4 agents; plan/implement/verify/evolve + independent-work-audit; ≤5 failClosed hooks; style advisory.
5. F5: preflight proves meta-plane exists; independent-work-audit READY before any shell-ui seed.
6. Keep collecting in `05` / `07`; do **not** implement `.cursor` product until operator starts a wave.

---

## G) Pointers into genEW (read when deepening)

| Topic | Path |
|-------|------|
| System blueprint | `.cursor/orchestration/SYSTEM_BLUEPRINT.md` |
| Lifecycle machine map | `.cursor/orchestration/lifecycle.json` |
| North star | `.cursor/rules/empery-north-star.mdc` |
| Planes / DoD | `.cursor/rules/architecture-core.mdc` |
| Honesty path | `.cursor/rules/production-path.mdc` |
| Fix generator | `.cursor/rules/fix-generator-not-capsule.mdc` |
| Stage audit | `.cursor/skills/independent-work-audit/SKILL.md` |
| Hooks config | `.cursor/hooks.json` |
| Context firewall | `.cursorignore` |
| First kernel commit | `git show d0e1bd8` |
| Harden commit | `git show 394a406` |
