# EmpiierOS — Historical Foundation Master Plan

**Status:** Historical baseline, superseded for acceptance by the 2026-09-16
secure reset. Current verdict: `NOT_READY`.
**Home:** `docs/foundation/`
**Date:** 2026-09-11
**Depends on capture:** `docs/capture/00`–`08`, `04` waves, `05` lessons, `09`–`13` forensics (plans), SESSION-SUMMARY.
**Accept:** [`docs/FOUNDATION-STATUS.md`](../FOUNDATION-STATUS.md) §Operator accept (P0/P10).

**Companion specs:**

| Spec | Role |
|------|------|
| [PROJECT-TREE.md](PROJECT-TREE.md) | Target directory tree + owners |
| [AGENT-KERNEL-SPEC.md](AGENT-KERNEL-SPEC.md) | Mini `.cursor` KEEP/HARVEST/DROP |
| [GIT-AND-IGNORE-SPEC.md](GIT-AND-IGNORE-SPEC.md) | git init / identity / ignores |
| [FILE-SIZE-AND-MODULARITY.md](FILE-SIZE-AND-MODULARITY.md) | LOC limits, split rules, doc homes |

Working capture and this plan remain records of earlier decisions. Current
execution and acceptance authority is `docs/FOUNDATION-STATUS.md`,
`docs/plans/MATURITY-LEDGER.md`, and the secure-reset audit.

---

## 1. Locked product & isolation (do not re-litigate)

1. **Axis:** OS Shell (React) + EliteEW Runtime (first-party) + Linux floor. Tauri = living host in monorepo, **not** plan language.
2. **Avoid:** (a) beautiful skin without bridge/ops; (b) “our GNOME”.
3. **Method:** do not break living; new = copy → paste → edit; soft-retire later.
4. **Repo:** EmpiierOS = attention/machine isolation under that axis — not “idea = new folder in monorepo”.
5. **Foundation:** multi-wave, machine-checkable; **no product UI** until preflight green.
6. **Isolation now:** separate repo + Cursor window + no mixed agent session without kinship-task.
7. **Linux user:** desirable max on F2 later — **not** a start blocker.
8. **Kinship:** monorepo = living Tauri / Substrate / API **read-only**; work home = this repo.
9. **Mature capability:** `shell surface + bridge op + substrate policy` — never `tauri command`/plugin as SoT.

---

## 2. What “foundation done” means

Foundation is **green** only when:

```text
scripts/preflight-foundation.sh  → exit 0
```

checks (see §6 F5) pass **and** operator accepts: *«we can live in this repo for weeks without product code»*.

Markdown alone ≠ foundation.

The 2026-09-11 foundation acceptance was invalidated by the 2026-09-16 audit:
its gate could pass while the image and product path were unusable. Product
unlock was not revoked, but expansion is on hold until secure-reset acceptance.

---

## 3. Non-goals (this plan)

| Non-goal | Why |
|----------|-----|
| React / Runtime / Substrate payload code | Product waves after acceptance |
| Actual seed copy from monorepo FE | F6 = policy only; copy = later task |
| `git init` / Linux user create | Operator must say "do it" (post–Task 12) |
| Clone EliteEW/genEW `.cursor` zoo | Gate fatigue; KEEP only |
| Cloud deploy / mobile / Keycloak / Gemma mandatory | DROP |
| Bidirectional shared packages day-1 | Conscious duplication until Shell contract stable |
| In-place living Tauri/FE/kiosk prep | Parallel line |

---

## 4. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Canon forgotten in days | Preflight + few failClosed hooks |
| Gate fatigue / bypass | Add controls only for a named failure with a bounded semantic selftest |
| False-green / softened gates | R9 anti-gaming + strict preflight anchors |
| Second DEWA / second cloud | Anti-second-universe + seed denylist |
| Skeleton becomes root dump | PROJECT-TREE + FILE-SIZE rules |
| F1 forever / analysis paralysis | Provisional kernel shipped; F1 deep reads refine later |
| Mixing monorepo in OS chat | Session hygiene + KINSHIP |
| Premature shared package | F6 conscious duplication |

---

## 5. Dependency graph (order)

```text
F0 Capture lock ─────────────────────────────────┐
                                                 │
F1 Forensic (plans 09–13; deep reads parallel) ──┼──► F4 Agent kernel (refined by F1.6)
       │                                         │
       ▼                                         │
F2 Skeleton + optional git init ─────────────────┤
       │                                         │
       ▼                                         │
F3 Ignore firewall ──────────────────────────────┤
       │                                         │
       ▼                                         │
F4 Mini agent kernel ◄───────────────────────────┘
       │
       ▼
F5 Preflight green ──► operator accept ──► (product waves later)
       │
       ▼
F6 Seed policy (text only; no copy yet)
       │
       ▼
F7+ docs map / F8 security defaults / F9 signing stubs / F10 substrate kinship
```

**Pragmatic path (historical):** F0 → F2/F3/F4 provisional → F5 harden → accept → F6 policy → later git init / product.

---

## 6. Waves — unified exits (machine-checkable)

### F0 — Capture & intent lock

| Exit | Check |
|------|--------|
| Intent lock in `00` | File exists; Lock section present |
| Twin risks + maturity in `01` | Present |
| This master plan pack | `docs/foundation/*.md` five files exist |
| No product code | No `shell-ui/src` app yet (or empty placeholders only) |

**Status:** done (capture + plan pack; curator Tasks 1–3).

### F1 — Lesson forensic

| Exit | Check |
|------|--------|
| `docs/AGENT-LESSONS.md` (or promoted `05`) | ≥12 mechanism lessons |
| KEEP/HARVEST/DROP table | Present; aligns with AGENT-KERNEL-SPEC |
| Git forensic notes | ≥10 anti-patterns **or** explicit “deferred with smoke-only” if operator prioritizes F2 |
| Preflight later fails if lessons missing | F5 |

Tracks F1.2–F1.12: see `04` / `10`–`13`. Parallel OK.

**Status:** done for F1.2–F1.12 (agent forensic + git + trash/parked + security/launcher/open, 2026-09-13).

### F2 — Skeleton (+ git when approved)

| Exit | Check |
|------|--------|
| Tree matches PROJECT-TREE | Dirs exist with `.gitkeep` or README stubs |
| Root not a dump | Capture under `docs/capture/`; product planes empty |
| README product + kinship | Points to monorepo read-only |
| `git init` | **Only if** operator said "do it"; else deferred |

**Status:** done (Task 1 skeleton; `git init` Task 13; baseline commit `17e0ffe` 2026-09-13 — no remote).

### F3 — Ignore firewall

| Exit | Check |
|------|--------|
| `.gitignore` / `.cursorignore` / indexing | Matches GIT-AND-IGNORE-SPEC |
| Preflight | Fails if missing or weak env/secrets lines |

**Status:** done (Task 2 + Task 6 harden).

### F4 — Agent kernel

| Exit | Check |
|------|--------|
| Rules R1–R9 | alwaysApply under `.cursor/rules/` |
| Hook kernel | `hooks.json` + semantic `policy_guard.py` selftest |
| Skills (historical cap 13) | Current disk: **13 skills**; language-bridge remains removed |
| Agents (historical 4) | Superseded: **5 agents** (inquisitor added) |
| DROP list | In AGENT-KERNEL-SPEC + AGENT-LESSONS |

**Status:** done (Tasks 4, 7, 9, 11 + **lifecycle skills 2026-09-13**).

### F5 — Preflight

| Exit | Check |
|------|--------|
| `scripts/preflight-foundation.sh` | Executable; exit 0; no WARN-as-PASS |
| Checks | skeleton, ignores, lessons, kinship, kernel, skills/agents, seed policy, no product flood |
| Operator accept | `docs/FOUNDATION-STATUS.md` §Operator accept |

**Status:** done (Tasks 5–6, 8, 10–11 harden + curator).

### F6 — Seed policy (text)

| Exit | Check |
|------|--------|
| `docs/seed/ORIGIN.template.md` | Exists; exact provenance headings |
| Allowlist / denylist | In `docs/seed/SEED-POLICY.md` exact `##` headings |
| No actual copy | Preflight fails on `shell-ui/package.json` / flood |

**Status:** done this task (Task 12) — **policy only; no FE copy**.

### F7+ (parked / partial)

| Piece | Status |
|-------|--------|
| F7 thin: `docs/NORTH-STAR.md` + `docs/WHERE-OS-LIVES.md` | done 2026-09-13 |
| F7: `docs/OS-SHELL-V0-SOT.md` | done 2026-09-13 |
| F7 remainder: fuller docs map / writing rules | optional later |
| F8 security defaults | **done 2026-09-13** — `docs/foundation/SECURITY-PRIVACY-DEFAULTS.md` |
| F9 signing stubs | **done 2026-09-13** — `docs/foundation/RELEASE-SIGNING-STUBS.md` (no secrets) |
| F10 substrate kinship | **done 2026-09-13** — `docs/foundation/SUBSTRATE-KINSHIP.md` (contract; no BR import) |

**Not** product Done.

---

## 7. Foundation task checklist (not product backlog)

- [x] **P0** Operator approves this master plan pack — **2026-09-11**; see `docs/FOUNDATION-STATUS.md` §Operator accept
- [x] **P1** F2: create dirs per PROJECT-TREE (empty product) — Task 1
- [x] **P2** F3: write `.gitignore` / `.cursorignore` / indexing — Task 2
- [x] **P3** F4 provisional: rules/hooks/skills/agents stubs — Tasks 4, 7, 9, 11
- [x] **P4** Relocate working capture → `docs/capture/` (Task 3, 2026-09-11); plan pack → `docs/foundation/`
- [x] **P5** F5: implement + harden `preflight-foundation.sh` — Tasks 5–6, 8, 10–11
- [x] **P6** F1: **F1.2–F1.12 done 2026-09-13** (agent + git + trash/parked + security/launcher/open)
- [x] **P7** F6: SEED-POLICY + ORIGIN template (no copy) — Task 12
- [x] **P8** `git init` local (Task 13) + baseline commit `17e0ffe` (2026-09-13 Commit Package) — **no remote**
- [ ] **P9** Optional: Linux user / OS git identity — later
- [x] **P10** Operator accept foundation → unlock **planning** only (not product code) — **2026-09-11**; pointer `docs/FOUNDATION-STATUS.md` §Operator accept
- [x] **P11** F7 docs: `NORTH-STAR` + `WHERE-OS-LIVES` + `OS-SHELL-V0-SOT` (2026-09-13); fuller docs map optional later
- [x] **P12** Product unlock recorded 2026-09-13 — `PRODUCT-UNLOCK.md` + historical wave-0 (now in `trash/`)
- [x] **P13** PW0 **A** scaffold 2026-09-13 — shell-ui Vite/React + runtime Rust stub + session CONTRACT + ORIGIN-gated anti-flood
- [x] **P14** Health IPC 2026-09-13 — loopback `/v0/health` + Shell client + smoke (not Tauri)
- [x] **P15** Wave B chrome seed 2026-09-13 — tokens + thin chrome + ORIGIN (no wholesale FE)
- [x] **P16** Named op `seat.status` 2026-09-13 — loopback IPC + unbound honesty
- [x] **P17** Bootstrap FSM v0 2026-09-13 — Identifying-until-resolved (Shell helpers + session contract)
- [x] **P18** Named op `runtime.version` 2026-09-13 — loopback `/v0/version`
- [x] **P19** Named op `update.channel_status` 2026-09-13 — F9 matrix honesty `hold`
- [x] **P20** Named op `enroll.status` 2026-09-13 — honesty stub `available:false` + FSM gate
- [x] **P21** Meta skill `capability-harvest` 2026-09-13 — skills **12/12**
- [x] **P22** Agent `inquisitor` 2026-09-13 — agents **5/5** (readonly adversarial)
- [x] **P23** Named op `edition.profile` 2026-09-13 — honesty stub `offline-class`
- [x] **P24** Meta `hooks-selftest` 2026-09-13 — tiny allow/deny gate (bindings still 7/8)
- [x] **P25** Hook `secrets-read` 2026-09-13 — bindings **8/8** (`beforeReadFile`)
- [x] **P26** Named op `device.identity` 2026-09-13 — honesty stub `bound:false`
- [x] **P27** Named op `entitlement.present` 2026-09-13 — honesty stub `present:false` + Start CTA gate
- [x] **P28** Named op `enroll.welcome_policy` 2026-09-13 — honesty stub `deferred` + welcome gate
- [x] **P29** Named op `enroll.carriers` 2026-09-13 — empty inventory honesty (USB/Air not wired)
- [x] **P30** Named op `host_network.status` 2026-09-13 — honesty stub `available:false` / `link:unknown`
- [x] **P31** Named op `transport.status` 2026-09-13 — honesty stub `listening:false` / `protected:false`
- [x] **P32** Named op `kernel_offline.status` 2026-09-13 — honesty stub `enforced:false` / `air_gap:false`
- [x] **P33** Named op `fs_bridge.status` 2026-09-13 — honesty stub `enabled:false` / `writable:false`
- [x] **P34** Named op `lifecycle.restart` status 2026-09-13 — honesty stub `allowed:false` (no execute)
- [x] **P35** Named op `auth.status` 2026-09-13 — honesty stub `authenticated:false` (no login/token)
- [x] **P36** Named op `license.status` 2026-09-13 — honesty stub `valid:false` (no license material)
- [x] **P37** Named op `seat.local_factory_reset` status 2026-09-13 — honesty stub `allowed:false` (no wipe)
- [x] **P38** Named op `product.open` status 2026-09-13 — honesty stub `allowed:false` (no URL-from-JS)
- [x] **P39** Meta skill `honesty-stub-wave` 2026-09-13 — skills **13/13** (lean bridge honesty waves)
- [x] **P40** Named op `update.apply` status 2026-09-13 — honesty stub `allowed:false` (no live OTA)
- [x] **P41** Harvest deepen enroll carriers (USB/Air) 2026-09-13 — map only; living = path pick/code, not udev
- [x] **P42** `enroll.carriers` path honesty 2026-09-13 — `usb/air:false` semantics; no Capsule/Air execute
- [x] **P43a** USB `.ecap` pick probe 2026-09-13 — `EMPIIEROS_USB_PICK=1` lab policy; default false
- [x] **P43b** USB `.ecap` pick execute 2026-09-13 — POST redacted metadata; lab file / optional zenity; no Capsule import
- [x] **P44** Harvest Capsule EECAP3 2026-09-13 — map only; next = structure validate (not full import)
- [x] **P45** `enroll.usb_ecap_validate` 2026-09-13 — structure-only; not crypto-open/import
- [x] **P46** Harvest device_keys 2026-09-13 — map only
- [x] **P47** `device.keys` status 2026-09-13 — `present:false` / `backend:none` (no keyring yet)
- [x] **P48** `device.keys_ensure` lab runtime-file 2026-09-13 — `keyring_wired:false`; not Capsule open
- [x] **P49** `enroll.usb_ecap_open` crypto-open lab 2026-09-13 — verify+decrypt; no payload IPC; not import
- [x] **P50** `enroll.usb_ecap_import` lab bind 2026-09-13 — redacted bind file; not entitlement/channel/Start
- [x] **P51** `entitlement.lab_import` lab store 2026-09-13 — `present:true`/`verified:false`; Start needs both
- [x] **P52** Language bridge (Gemma RU↔EN) Path A 2026-09-13 — caps unchanged; not ОДОБРЕНО auditor
- [x] **P53** `entitlement.lab_verify` Ed25519 lab 2026-09-13 — may set `verified:true`; not channel / not product Done
- [x] **P54** `device.keys` OS keyring ensure (Option A) 2026-09-13 — `keyring_wired:true` after Secret Service
- [x] **P55** Capsule open-from-keyring 2026-09-13 — crypto loaders use keyring SoT; lab file via env
- [x] **P56** `update.channel_status` deepen 2026-09-13 — hold + reason/package_present/local_stub; never fake live
- [x] **P57** Air `air_code` path probe 2026-09-13 — `EMPIIEROS_AIR_CODE=1`; no WSS execute
- [x] **P58** `host_network.status` nmcli deepen 2026-09-13 — backend/link/kind; nm_gui:false; no fake up

---

## 8. Pointers into capture (do not duplicate forever)

| Need | Read |
|------|------|
| Axis / isolation lock | `../capture/00-intent-and-understanding.md` |
| Hybrid / maturity / skin risks | `../capture/01-hybrid-and-os-shell-vision.md` |
| Tauri stance | `../capture/02-tauri-stance.md` |
| Repo isolation | `../capture/03-repo-isolation-and-identity.md` |
| Wave detail / F1 tracks | `../capture/04-foundation-waves.md` |
| KEEP/DROP draft | `../capture/05-lessons-to-harvest.md` |
| Guards | `../capture/06-non-goals-and-guards.md` |
| Kinship | `../capture/08-relationship-to-monorepo.md` |
| genEW / mono / git / trash / security plans | `../capture/09`–`../capture/13` |

---

## 9. Next gates (not auto-start)

- **Do not** add remote / push until operator asks. Further commits only via Commit Package + explicit approval.
- Living kernel (2026-09-18): **9 alwaysApply rules · 1 glob (`comfyui-media`) · 5 hook bindings · 13 skills · 5 agents**. Language bridge disabled. Unscoped product lean = **pause**; FoFline V2 is the locked exception.
- **Operator test announcements:** only when the named wave proof is green; lab `verified:true` ≠ channel ≠ product Done.
- F1–F10 foundation text slots **done**. F7 remainder (fuller docs map) optional.
- Anti-flood: ORIGIN required with `package.json`; never commit `node_modules`.
