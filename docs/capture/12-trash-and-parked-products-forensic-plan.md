# Trash + parked products forensic plan

**Capture date:** 2026-09-11
**Status:** F1.8 + F1.9 **executed** 2026-09-13 (T0–T3 read-only). Soft-retire batch on this host = **0-file skeletons**; DEWA/EWPower deep claims limited to layout+canon. Kinship trees not restored/copied.
**Purpose:** (1) analyze the culture and content of `trash/` in the monorepo; (2) study soft-retired / parked large-scale projects (**DEWA**, **EWPower**, **EWGen**, **EWC**, **EWCoderOrchestrator**, **SSFEW**, …) as **working visual tools** set aside to avoid interference — not "deleted," but **sources of insights** for the EmpiierOS foundation.

**Hard rule:** harvest capabilities / patterns / architecture lessons — **never** restore whole trees as second products into EmpiierOS or monorepo root without an explicit operator restore task.

Companions: [`09`](09-genew-harvest.md) · [`10`](10-monorepo-agent-forensic-plan.md) · [`11`](11-git-history-forensic-plan.md) · monorepo skill `eliteew-capability-harvest`.

---

## Hard constraints

| Allowed | Forbidden |
|---------|-----------|
| Read under monorepo `trash/`, Documents trash, named parked trees | `git restore` / copy whole DEWA/EWPower/… to living roots |
| Inventory, architecture notes, capability maps → EmpiierOS only | Recreate second SaaS / second Keycloak / second cloud |
| Quote paths, README, manifests, small excerpts | Blind `cp -a` of product trees into EmpiierOS |
| Use monorepo harvest skill **as methodology reference** (read) | Port product ops (deploy lines, domain hosts) wholesale |

Operator intent (this session): trees are **parked so they don’t interfere**; they remain **good working exemplars** → study → lay foundation. EmpiierOS must **not** become DEWA-2 / EWPower-2.

---

## Two different “trash” realities (do not conflate)

| Kind | What it is | Study goal |
|------|------------|------------|
| **A. Operational trash** | Monorepo `../powered_by_emperyworld/trash/YYYYMMDD-HHMMSS/…` — agent/human **trash-move** buckets (~138 on this host; mostly docs/shared scraps; some large e.g. Substrate `out/`) | How trash-not-rm works in practice; what agents throw away; failure modes; ignore/firewall; retention |
| **B. Soft-retired product parks** | Named trees **DEWA, EWPower, EWGen, EWC, EWCoderOrchestrator, SSFEW, nexus-root, …** moved off workspace root so they don’t drown EliteEW | Working architectures & UX/tooling to **learn from**; capability harvest mindset; anti-second-universe law |

Canon already states this split: `.cursorrules` retired list + `eliteew-capability-harvest` + editions §10.

---

## Locate map (this host — verify in T0)

Preferred soft-retire batch (canon): `trash/20260718-015129/{DEWA,EWPower,EWGen,EWC,EWCoderOrchestrator,SSFEW,…}`

| Location | Status (2026-09-11 smoke) | Notes |
|----------|---------------------------|-------|
| `powered_by_emperyworld/trash/20260718-015129/` | **Not present** under monorepo trash | Monorepo trash is mostly timestamped operational moves |
| `~/Документы/trash/20260718-015129/` | **Present** (DEWA, EWPower, EWGen, EWC, EWCO, SSFEW, nexus-root, …) | Sizes look **sparse/skeleton** — may be incomplete; confirm before deep conclusions |
| `~/Документы/EWGen` | **Large (~8.5G)** living-looking EWGen tree | Strong candidate for deep read if soft-retire copy is thin |
| `../genEW` | External Empery prototype | Already harvested for agent plane in `09`; product/factory still relevant for T2 |
| `../EliteEW` | Host copy resembling Empery/genEW layout | Locate-only; don’t confuse with monorepo |
| Archives (`Документы/ewgold.tar`, `EliteEW.tar`, zips) | Present | Open **only** if operator asks; note as offline sources |
| Monorepo `trash/README.md` | Points older archives → `documentation/archive/` | Separate legacy archive track |

**T0 must:** build authoritative locate table (path, size, README, git?, runnable?, completeness) before deep study.

---

## Parked product roster (study targets)

From `.cursorrules` / harvest source-map (not deleted — soft-retired):

| Tree | Former role (canon) | EmpiierOS study angle |
|------|---------------------|------------------------|
| **DEWA** | Operator / energy apps | App composition, operator UX, what became “second SaaS” pain |
| **EWPower** | Power / energy plane | Domain tooling that worked; isolation vs EliteEW |
| **EWGen** | Generation / control-center | Factory / control-plane patterns (kinship to genEW) |
| **EWC** | Coder plane | Agentic coder surfaces |
| **EWCoderOrchestrator** | Orchestrator | Multi-agent / job orchestration lessons |
| **SSFEW** | Hub / agent patterns | Shell/hub patterns (not second hub product) |
| **nexus-root** | (retired) | Boundary / why it was parked |
| **access-gate-logwatch**, **tg_bot_ip_verif** | Ops side tools | Optional; lower priority for OS foundation |

Also study **operational trash patterns** (Kind A) as management methodology.

---

## Study phases

### T0 — Locate & classify inventory

- [x] List monorepo `trash/` top-level: count, size histogram, named vs timestamp-only
- [x] Confirm soft-retire batch path(s) on this host; completeness (file counts, README, package.json / compose presence)
- [x] Map external full trees (Documents/EWGen, genEW, …)
- [x] Tag each: `ops-trash` | `soft-retire-park` | `external-prototype` | `archive-tar` | `skeleton-incomplete`
- [x] Write locate table into §Findings below

**Exit T0:** know *where the real working exemplars are* before analyzing empty shells. **Done 2026-09-13.**

### T1 — Operational trash forensic (Kind A)

Questions:

1. What categories get trash-moved (docs, shared, .cursor, Substrate outs, failed experiments)?
2. Repeat offenders / agent mistake shapes (wrong path, Dual SoT debris, probe modules)?
3. Does trash-not-rm + retention actually protect archaeology?
4. Ignore/indexing: should EmpiierOS F3 ignore `trash/` always but allow **explicit** `@path` harvest reads?
5. Relation to `eliteew-capability-harvest` and `trash-and-deletion-policy`

**Exit T1:** lessons for F3 ignore + F4 trash-not-rm + “capability harvest from trash” skill stub (OS-flavoured).

### T2 — Parked products deep read (Kind B) — **priority exemplars**

For each priority tree (**DEWA**, **EWPower**, then **EWGen** / EWC / EWCO / SSFEW as time allows):

| Lens | Ask |
|------|-----|
| Product shape | What user journeys worked? What was the “desktop/tool” face? |
| Architecture | Monolith vs services? Auth? Deploy? Why N universes hurt? |
| Agent/meta | Did it have `.cursor` / rules? What scaled or drowned? |
| Quality signals | Tests, honesty, stubs, docs lag |
| Harvestable for OS | Shell UX patterns? Runtime ideas? Ops tools? Pack/module boundaries? |
| Explicit DROP | Second Keycloak, full cloud clone, revive as root product |

Method aligned with monorepo `/eliteew-capability-harvest`:

locate → inventory capabilities → map to OS planes (shell/runtime/session/meta)
  → KEEP mechanism / HARVEST rewrite / DROP product zoo
  → NEVER restore whole tree

**Exit T2:** per-tree one-pager in EmpiierOS + cross-cutting anti-DEWA / anti-second-cloud foundation rules strengthened with evidence.

### T3 — Cross-synthesis with 09 / 10 / 11

| Source | Contribution |
|--------|----------------|
| genEW `09` | Kernel-first agent plane; compose-not-invent |
| monorepo agent `10` | Cause triad, Commit Package, trash policy machine |
| git `11` | Scars that *caused* soft-retire / fix storms |
| This file | Working parked tools as positive exemplars + zoo failure evidence |

**Produce:** “Parked products → EmpiierOS foundation” section for F1.6 AGENT-LESSONS:

- Why separate OS repo (avoid DEWA-style drown)
- Soft-retire > delete (archaeology + harvest)
- Capability harvest protocol for OS seed later (F6)
- What **not** to rebuild (N auth/deploy surfaces)

### T4 — Optional runnability / demo (operator-gated)

Only if operator says so: attempt **read-only** understanding of how to run DEWA/EWPower locally (docs only first). No porting into EmpiierOS. No network publish.

---

## Findings (append-only)

_Format: `DATE | KIND | PATH | completeness | one-line lesson | foundation control`_

- 2026-09-11 | locate-smoke | monorepo `trash/` ~138 buckets | ops-trash dominant; soft-retire batch **not** under monorepo path
- 2026-09-11 | locate-smoke | `Документы/trash/20260718-015129/{DEWA,EWPower,…}` | present but **sparse sizes** — verify before deep claims
- 2026-09-11 | locate-smoke | `Документы/EWGen` ~8.5G | candidate full exemplar
- 2026-09-11 | locate-smoke | `../genEW` | external prototype (see `09`)

### T0 locate table (2026-09-13 — host-truthful)

| Path | Size / count | Tag | Completeness |
|------|--------------|-----|--------------|
| `powered_by_emperyworld/trash/` | **~140** buckets · **4.0G** | ops-trash | Real content; 136×&lt;10M, 3×10–99M, 1×≥100M (`20260907-082046` ~3.7G trusted-substrate) |
| Soft-retire `Документы/trash/20260718-015129/` | **548K** · **0 files** entire tree | soft-retire-park + **skeleton-incomplete** | Dir layout only (DEWA/EWPower/EWGen/EWC/EWCO/SSFEW/nexus/…) — **not** deep-readable source |
| `Документы/EWGen` | **8.5G** · ~563k files · git · pkg · compose · `.cursor` | external-prototype | **Fullest EWGen** on host |
| `../genEW` | **1.3G** · git · `.cursor` | external-prototype | Agent/factory kinship (`09`) |
| `../EliteEW` | 4K stub | locate-only | Not a harvest tree |
| `Документы/ewgold.tar` / `EliteEW.tar` | 4.0G / 208M | archive-tar | Operator-gated open only |

**Ops-trash categories (sample):** docs-scrap ~60 · `.cursor`-meta ~42 · shared-fe ~18 · substrate-out ~11 · other/empty ~9.

### T1 — Ops-trash lessons (F1.8)

| # | Lesson | Control |
|---|--------|---------|
| 1 | Trash-moved: dual-SoT docs, `.cursor` state, shared API/FE scrap, Substrate outs | EmpiierOS `trash/**` ignore + `trash-move.sh` (already) |
| 2 | Mistake shapes: doc Dual SoT waves; hook/state churn; shared experiments; build outs left in-tree | Cause-first: fix generator (ignore outs / single SoT), not only trash more |
| 3 | trash-not-rm protects archaeology **when files survive**; empty soft-retire ≠ harvestable park | KEEP R5; honesty about skeleton parks |
| 4 | Always ignore bulk `trash/**`; harvest only via explicit `@path` | F3 already; F6 harvest protocol |
| 5 | Kind A (ops) ≠ Kind B (parked products) — do not conflate | Capture `12` law; later capability-harvest skill |
| 6 | Machine policy: trash-move + retention gates work in monorepo | EmpiierOS thin trash-not-rm hook (F4) — no gate zoo |

### T2 — Parked one-pagers (F1.9)

#### DEWA
- **Path:** soft-retire skeleton only (0 files). No fuller DEWA tree on host this pass.
- **Shape (canon + dirs):** operator shell — Bridge/missions/session/stack0 — own BE+FE+systemd = **second SaaS face**.
- **Harvest:** shell/session chrome + bridge notions → rewrite to OS planes later.
- **DROP / NEVER:** revive DEWA root; second Keycloak/`dewa.*` domains; whole-tree restore.

#### EWPower
- **Path:** soft-retire skeleton only (0 files).
- **Shape (canon + dirs):** ops/perimeter console (stacks/diagnostics/repairs/edge). Same product envelope as DEWA.
- **Harvest:** diagnostics/repair *workflow patterns* → runtime ops ideas (mechanisms).
- **DROP / NEVER:** second perimeter cloud; revive as root.

#### EWGen
- **Path:** soft-retire useless; **substance** = `Документы/EWGen` (8.5G) + kinship `genEW` (`09`).
- **Shape:** AI module generator + FastAPI + local AI services; heavy FE zoo.
- **Harvest:** factory/generator discipline via **genEW meta** — not second generator SPA in EmpiierOS.
- **DROP / NEVER:** revive EWGen as OS product; second AI compose stack as SoT.

#### EWC / EWCoderOrchestrator / SSFEW
- Soft-retire 0 files; layout hints only (coder backend; Deck/API+ui; agent+hub).
- **Harvest:** multi-agent/job/hub *mechanisms* → session/meta later.
- **DROP:** second coder/hub products as roots.

### T3 — Cross-synthesis

**Why separate OS repo:** editions/glossary + soft-retire of nine product roots + continuous ops-trash Dual SoT churn prove multi-product drown. EmpiierOS one-root + KINSHIP.

**Soft-retire > delete:** UTC trash still holds recoverable content; empty-shell soft-retire loses harvest — prefer real trash-move or keep external full trees.

**Harvest protocol (F6 later):**
```text
locate fullest tree → inventory capabilities → map to shell|runtime|session|meta
  → KEEP / HARVEST rewrite / DROP zoo → report + Approval → NEVER restore whole tree
```

**What not to rebuild:** N auth/deploy surfaces; DEWA/EWPower/EWGen/EWC/EWCO/SSFEW as roots; second cloud/Keycloak.

-

---

## Exit criteria (F1.8 + F1.9 done)

- [x] T0 locate table complete (host-truthful)
- [x] T1 ops-trash lessons (≥5) → F3/F4
- [x] T2 one-pagers for **DEWA** + **EWPower** (layout+canon honesty); EWGen via external full tree; EWC/EWCO/SSFEW shape notes
- [x] Explicit KEEP/HARVEST/DROP for parked products (no whole-tree restore)
- [x] Anti-second-universe rules reinforced with concrete evidence
- [x] Folded into AGENT-LESSONS
- [x] Zero restores / zero new product roots created

---

## Relation to foundation waves

| Wave | Feed |
|------|------|
| **F1.8** | Execute T0–T1 (trash forensic) |
| **F1.9** | Execute T2–T3 (parked DEWA/EWPower/… study) |
| F3 | Always ignore bulk `trash/` noise; allow explicit harvest paths |
| F4 | trash-not-rm; capability-harvest skill (OS); anti-DEWA rule |
| F5 | Preflight: lessons include parked-product anti-patterns |
| F6 | Seed allowlist informed by harvest maps — still no wholesale revive |

See [`04-foundation-waves.md`](04-foundation-waves.md).
