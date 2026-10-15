# EmpiierOS — Git & ignore specification

**Status:** Spec. `git init` performed Task 13 (local identity only). **Do not** add remotes, change global git config, or create Linux users until operator asks.
**Date:** 2026-09-11
**Related:** `03-repo-isolation-and-identity.md`, PROJECT-TREE, AGENT-KERNEL-SPEC.

---

## 1. When to `git init`

| Condition | Action |
|-----------|--------|
| Operator has **not** said "do it" / «git init» | Keep working tree unversioned or leave as-is; continue capture/plans |
| Operator approves F2 **and** explicitly requests init | `git init` at this repo |
| Separate Linux user ready | Optional: init/commits as that user — **not** required for first init |

**Never:** mutate `powered_by_emperyworld` git config “for company”; force-push; commit secrets.

---

## 2. Identity (who)

### Start (locked enough)

- Work as current host user on this repo.
- Cursor window: **only** this workspace for OS foundation sessions.
- Do not open monorepo + EmpiierOS in one agent session without kinship-task.

### Git author (when init)

| Field | Provisional | Operator lock |
|-------|-------------|---------------|
| `user.name` | e.g. `EmpiierOS` or human name + `(EmpiierOS)` | TBD |
| `user.email` | OS-line email or `empiieros@local` | TBD |
| Signing | Stubbed F9 | See `RELEASE-SIGNING-STUBS.md`; no private keys in-repo |
| Remote | None until operator chooses host (GitHub/Cursor/…) | TBD |

Use **local** repo config (`git config user.name` **inside** EmpiierOS only) — do not change global if it would poison monorepo habits; prefer local.

### Linux user (optional later)

Create separate OS Linux user / `$HOME` / SSH / `gh` when operator starts isolation stretch — **not** a blocker for init or F2/F3.

---

## 3. First commits structure (after init)

Small, semantic, **no shotgun**:

| # | Message intent | Contents |
|---|----------------|----------|
| 1 | `chore: initialize EmpiierOS foundation skeleton` | PROJECT-TREE dirs, README stubs, ignore files, empty planes |
| 2 | `docs: add foundation master plan pack` | `foundation/` or `docs/foundation/` |
| 3 | `chore: add provisional agent kernel stubs` | `.cursor/` from AGENT-KERNEL-SPEC |
| 4 | `chore: add foundation preflight script` | `scripts/preflight-foundation.sh` |
| 5 | `docs: relocate working capture under docs/capture` | move `00`–`13`, SESSION-* |

Each commit = one concern. No monorepo paths. No product React.

**Commit Package rule:** agent proposes; human approves (R4).

---

## 4. Never commit

- `.env`, `*.pem`, `*.key`, credentials, tokens, `secrets/`
- `node_modules/`, `dist/`, `target/`, `.venv/`, build outs
- `.cursor/orchestration/audit/*.jsonl`
- `trash/**` contents (keep `trash/README.md` only)
- Monorepo checkouts / copies of `powered_by_emperyworld` or `genEW` trees
- Large binaries / RPM / USB images
- Operator personal dumps unrelated to OS line
- Hook state caches if any

---

## 5. `.gitignore` draft (spec text — implement on approval)

```gitignore
# Secrets & env
.env
.env.*
!.env.example
*.pem
*.key
secrets/
**/credentials*.json

# Dependencies & build
node_modules/
dist/
build/
target/
.venv/
__pycache__/
*.pyc
.pytest_cache/
.ruff_cache/
.turbo/
*.rpm
*.exe

# Cursor / agent noise
.cursor/orchestration/audit/*.jsonl
.cursor/hooks/state/
!.cursor/orchestration/audit/.gitkeep

# Local trash landing (soft-delete)
trash/**
!trash/README.md

# OS / editor
.DS_Store
*.swp
.directory

# Kinship mirrors (must never appear)
powered_by_emperyworld/
genEW/
**/connection-agent/
```

---

## 6. `.cursorignore` draft (spec text)

Goal: keep agent context on **OS foundation + planes**; hide dumps and future build noise; **do not** hide rules/skills/docs/foundation.

```gitignore
# Build & deps
node_modules/
dist/
build/
target/
.venv/
__pycache__/

# Secrets
.env
.env.*
secrets/
*.pem
*.key

# Agent audit noise (keep designs/reports readable)
.cursor/orchestration/audit/
.cursor/hooks/state/

# Soft-delete landing
trash/

# Never index accidental kinship copies
powered_by_emperyworld/
genEW/
```

**Do not ignore:** `.cursor/rules/`, `.cursor/skills/`, `.cursor/agents/`, `docs/`, `foundation/`, plane README stubs, `scripts/preflight-foundation.sh`.

---

## 7. `.cursorindexingignore` draft (recommended)

Softer than `.cursorignore` — out of `@codebase` but Read-by-path OK:

```gitignore
docs/capture/SESSION-DIALOGUE-*.md
trash/
.cursor/orchestration/audit/
```

Keep `docs/capture/00-*.md` indexed or not — operator choice; default: **index** thematic `00`–`08`, soft-ignore huge dialogue dump.

---

## 8. Session hygiene (git-adjacent)

| Rule | Enforce |
|------|---------|
| One agent session ↔ one workspace root | Rule R1 + optional warn hook |
| Kinship reads | Explicit task; no commits in monorepo from OS chat |
| Remote publish | No `git push` until remote + identity locked; hook may block push early |

---

## 9. Preflight hooks into ignores (F5)

`preflight-foundation.sh` must fail if:

- `.gitignore` or `.cursorignore` missing
- `.gitignore` does not contain `.env` / `node_modules/` / `trash/**`
- Tracked files match secret patterns (basic scan)
- `powered_by_emperyworld/` directory exists inside EmpiierOS tree

---

## 10. Operator locks still open

- [x] Signing keys stubs (F9) — `RELEASE-SIGNING-STUBS.md`; **no private keys in repo**
- [x] Local `git init` (Task 13) — done; remote still open
- [ ] Exact `user.name` / `user.email` for OS line (beyond local EmpiierOS identity)
- [ ] Whether first remote exists and where
- [ ] Separate Linux user timing
