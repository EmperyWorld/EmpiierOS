# EmpiierOS — Release & signing identity stubs (F9)

**Status:** Living foundation SoT — **F9 done 2026-09-13** (stubs only).  
**Not:** private keys in-repo. **Not:** live OTA pipeline. **Not:** RPM/NSIS/product packages. **Not:** remote push.  
**Sources:** `docs/capture/13` Track L/O · F8 §Update/signing trust · kinship VERSION/CHANNEL lessons.  
**Axis:** OS Shell + EliteEW Runtime + Linux floor — packaging honesty before any ship story.

## Purpose

Define **how EmpiierOS will talk about versions, channels, and signing roles** so agents cannot invent a “green release” without mirrors, matrix honesty, or key-role discipline. Implement packaging later against this file — do not copy monorepo signing scripts wholesale.

---

## Version SoT

When a shippable artifact exists, these must stay **one version story** (same semantic version / channel tag — refine names when Runtime lands):

| Mirror | Role |
|--------|------|
| Package manifest (future) | Human/install version |
| Runtime / bridge crate or binary metadata | Native binary version |
| Shell UI build stamp (future) | Face version — must not silently diverge |
| Release-state machine file (future) | Channel + platform matrix SoT |

**Law:** bump is a **single coordinated change** (one script or one Commit Package listing all mirrors). Partial bump = Dual SoT scar.

**Today (honest):** EmpiierOS has **no** product package version — only git commits. Do not claim a product version string.

---

## Channel matrix honesty

Every published channel entry must declare platform coverage explicitly:

| State | Meaning |
|-------|---------|
| `live` | Artifact published and installable for that platform |
| `lag` | Channel exists but this platform is behind (document how far) |
| `hold` | Intentionally not shipping |
| `omitted` | Platform not in this channel’s metadata — **not** “works everywhere” |

**Forbidden:** updater check OK while install fails because format/platform was omitted; silent “Windows same as Linux” claims; AppImage-only metadata sold to RPM-first clients (kinship scar).

Shell/Runtime UI must surface matrix state — never invent `live` from prose.

---

## Key roles

Signing trust uses **separate key roles**. Names are stubs; ceremony is operator-gated later.

| Role | Signs / verifies | Must not |
|------|------------------|----------|
| `entitlement` | Entitlement / license-class metadata | Artifact blobs |
| `denylist` | Revocation / denylist metadata | Entitlement or artifacts |
| `metadata` | Update plan / channel metadata (e.g. latest.v2-class) | Private mixing with artifact key |
| `artifact` | Package / payload digests | Entitlement/denylist documents |

**Laws:**

1. Roles are **not interchangeable** — wrong key = fail-closed verify.  
2. **Private material never enters EmpiierOS git** (see Ignore).  
3. Pubkeys may live as **documented fingerprints / stub paths** when products exist — not as “security theater” empty files claimed Done.  
4. Root of trust is **baked / pinned at build** for retail — runtime env must not silently downgrade trust (kinship lesson).

---

## Git identity vs release signing

| Identity | Purpose | Status |
|----------|---------|--------|
| Local git author `EmpiierOS <empiieros@local>` | Commit archaeology | **in use** (local only) |
| Optional OS Linux user / SSH / `gh` | Isolation max (`03`) | **not required** for F9 |
| Release / package signing keys | Ship authenticity | **stubs only — no keys yet** |
| Commit GPG signing | Optional later | not day-1 |

F9 does **not** require P9 Linux user. Do not block foundation on GPG.

---

## OTA / update UX stubs

Aligned with F8:

1. User-visible **consent** before silent auto-install when release notes unread (Linux dual-flag lesson).  
2. Check ≠ install ≠ restart — separate Runtime ops + honesty labels.  
3. When floor OS update and payload update both pending → **OS-before-payload** (F10 owns floor detail).  
4. Fail-closed result contracts for wipe/update (ticket + confirm).

---

## Ignore & repo hygiene (no secrets)

Already in F3 ignores; F9 reaffirms:

| Pattern / path | Why |
|----------------|-----|
| `*.pem`, `*.key`, `secrets/`, `.env*` | Never commit |
| Future `keys/private/**` (if created) | Must stay ignored + untracked |
| Kinship signing scripts / live pubkeys | Study only — do not copy private material |

Preflight may later scan for tracked secret-like paths; day-1 protect-secrets hook already blocks common writes.

**Stub location (documentation only):** when keys exist off-repo, record **ceremony notes** under `docs/foundation/` or operator vault — never private key bytes.

---

## Explicit DROP

- Committing private keys, `.p12`, entitlements with secrets  
- Porting monorepo `packaging/signing/` or updater pubkey JSON as EmpiierOS SoT  
- Claiming CHANNEL `live` without artifact proof  
- Designing OTA as Tauri plugin SoT  
- Fake “signed OS” while payload trust is only narrative  
- Mixing entitlement and artifact keys “for simplicity”

---

## Relation

| Artifact | Role |
|----------|------|
| This file | **Living F9 SoT** — release/signing stubs |
| [`SECURITY-PRIVACY-DEFAULTS.md`](SECURITY-PRIVACY-DEFAULTS.md) | F8 trust + update defaults |
| `docs/capture/13` | Forensic evidence (VERSION, CHANNEL_MATRIX, key roles) |
| F10 | Substrate kinship dual-flag + OS-before-payload — **done:** [`SUBSTRATE-KINSHIP.md`](SUBSTRATE-KINSHIP.md) |
| `GIT-AND-IGNORE-SPEC.md` | Ignore / no-secrets law |

When packaging first lands, update this file **same phase** as mirrors + matrix (R7).
