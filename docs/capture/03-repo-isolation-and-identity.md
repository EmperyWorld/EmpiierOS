# Repo isolation and identity

**Capture date:** 2026-09-11

## Decision lean (operator + agent agree)

For the EliteEW OS line, a **separate git repository is more productive** than another large folder within the `powered_by_emperyworld` monorepo.

Reasons:

- The monorepo already contains many products → context bloat;
- Cursor SoT / hooks / dirty tree / shotgun commits;
- Need for **own** rules, tasks, docs, and plans;
- Copy-paste-edit at the **project** level, not just the file level.

This capture folder lived briefly in the monorepo as a notebook.
**Home as of 2026-09-11:** this repo (full capture + handoff).
The monorepo serves only as a stub pointer.

## Stronger isolation (operator — clarified 2026-09-11)

| Level | What | When |
|-------|------|------|
| **Minimum (enough now)** | Separate repo + separate Cursor window + prohibition of mixing monorepo and OS in a single agent session without a kinship-task | **Foundation start — locked** |
| **Maximum (desirable, not blocker)** | Separate Linux user + `$HOME` / SSH / `gh` + git author/signing under OS identity | **Later on identity wave (F2)** — when the operator starts it; does **not** block F0–F1 capture / lessons |

The value of the maximum level is to physically complicate the mix-up (credentials/push/index), **not** the magic of `user.name`.
Do not wait for a Linux user to continue the foundation waves in this repo.

### What separate identity does / does not (when we add it)

| Does | Does not alone |
|------|----------------|
| Reduce credential/push mixups | Stop bad plans if both trees are open in one chat |
| Clean Cursor ignore/rules | Replace hooks/DoD |
| Clear git archaeology for OS line | Prevent copying the entire EliteEW agent zoo |

## Anti-DEWA / anti-second-cloud

A separate repo **≠** a second full EliteEW (api/spa/mobile/stack0) **and ≠** reviving DEWA/EWPower/EWGen as parallel roots.

Parked soft-retired trees (trash / host copies) = **study/harvest sources** — see [`12-trash-and-parked-products-forensic-plan.md`](12-trash-and-parked-products-forensic-plan.md). EmpiierOS must not become DEWA-2.

Kinship framework:

```text
eliteew-os (new git)
  shell-ui/   runtime/   session/   docs/   .cursor/

powered_by_emperyworld (living)
  API, cloud SPA, Tauri connection-agent, Trusted Substrate kiosk
  = seed origin + contracts; not daily OS workspace
```

## Why not «just a folder in monorepo»

`pack-organization/eliteew-os/` is logically clean, but it **provides almost no protection for Cursor**: the workspace is still the monorepo → one agent context, one dirty tree, one `.cursor` SoT.

A separate git enables **project-level** copy-paste-edit: copy the seed → edit only there → do not break the living EliteEW.

## Conscious duplication + no early shared package

The Seed era = **intentional** file copy + `ORIGIN.md`, not a submodule-as-SoT UI and not a bidirectional shared package before a stable OS Shell contract is established.

## Parked text before create-repo

Once foundation waves are agreed upon / the operator gives the word — a one-pager **WHERE OS LIVES** (layout + non-goals + seed allow/deny).
`git init` / user identity — only upon explicit "start" command.

## Ignore-first foundation

Before product code in the new repo:

1. `.gitignore`
2. `.cursorignore`
3. `.cursorindexingignore` (if used)
4. An explicit list of what **never** to index (secrets, build noise, monorepo mirrors)

Properly configured ignore files are part of the foundation, not something to "clean up later."

## Bootstrap note

`git init` and the first commits under the OS identity is a ritual for the **new** repo (operator).
Do not mutate the monorepo's git config "for the sake of it."
