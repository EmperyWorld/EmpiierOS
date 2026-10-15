# EmpiierOS — Substrate kinship contract (F10)

**Status:** Living foundation SoT — F10 contract, updated 2026-09-18.
**Not:** Buildroot factory copied into EmpiierOS. **Not:** writing `connection-agent`.
**Now:** the 2026-09-16 secure reset may write
`powered_by_emperyworld/pack-organization/trusted-substrate/` under an explicit
operator scope. Official EmpiierOS overlays are already the payload of that
factory. Product unlock exists; unscoped expansion does not.
**Sources:** `docs/capture/13` · `01` hybrid vision · `08` monorepo relationship · F8/F9 · kinship `trusted-substrate`.
**Axis:** OS Shell + EliteEW Runtime + Linux floor.

## Purpose

Lock how EmpiierOS relates to **Trusted Substrate** (living floor in monorepo): dual-flag payload profiles, update precedence, honesty labels, and the hard rule **do not break living kiosk/Tauri**. Payload implementation is a later product wave — this file is the contract those waves must obey.

---

## Kinship boundary

| Lives in | Owns | EmpiierOS may |
|----------|------|----------------|
| Monorepo `pack-organization/trusted-substrate/` | Buildroot factory, kiosk session, nft overlays, EmpiierOS payload slot | Study by default; **write only** under the recorded secure-reset / explicit kinship-task |
| EmpiierOS `session/` | Seat/session **contracts** and notes for future OS profile | Write contracts/docs |
| EmpiierOS `runtime/` / `shell-ui/` | Bridge + Shell (when unlocked) | Deliver **artifact** for a future OS payload profile |
| EmpiierOS docs (this file) | Kinship law | Normative for OS agents |

**Law:** do not import the BR factory tree into EmpiierOS. Do not flip the
monorepo default kiosk off Tauri from this repo. Delivery is a **second
payload profile** of the same factory (EmpiierOS overlay), not an in-place
rewrite of the living Tauri profile. Other monorepo paths stay read-only.

---

## Dual-flag

Exactly **one** active payload profile per image/build. Conflicting dual-active profiles = **build FAIL** (fail-closed).

| Profile class (names stub) | Intent |
|----------------------------|--------|
| Living host (today) | `tauri` / kiosk payload — **do not break** |
| Future OS Shell | EmpiierOS-built shell+runtime artifact as payload |
| Staff seat | `linkornew` (LinkornEW). Own shell, installer, and Command Post contour. Not selected on the FoFline image. Factory fragment is not applied yet. Contract [`../../session/LINKORNEW.md`](../../session/LINKORNEW.md). |
| Bootstrap / legacy | Historical only; honesty labels required |

**Forbidden:** activating living Tauri profile and OS Shell profile on the same image “to try both.” Dual-flag = exclusivity, not A/B runtime flip via env.

When the product payload wave starts, EmpiierOS agents propose profile names/labels here first — then kinship implementers apply them in monorepo factory under operator task (separate session).

---

## Layer split (mental model)

| Layer | Cadence | Example |
|-------|---------|---------|
| L0 toolchain | Slow | Cross compilers / BR infra |
| L1 OS floor | Medium | Rootfs, nft, session compositor policy |
| L2 shell payload | Faster | EmpiierOS Shell + Runtime binary/bundle |
| L3 content | Fastest | SPA/app packs when used |

**Law:** UI/payload updates must not require rebaking the whole OS floor when avoidable. Floor policy changes are deliberate and dual-flagged.

---

## Update precedence

When both apply:

1. **Pending OS / floor update wins** over payload update.
2. Payload update proceeds only after floor channel policy allows.
3. Channel matrix honesty from F9 (`live` / `lag` / `hold` / `omitted`) applies to floor and payload **separately**.

Aligns with F8/F9; this file owns the **floor vs payload** ordering rule.

---

## Honesty labels (payload trust)

| Label | When allowed |
|-------|----------------|
| `payload_trust=tauri-living` | Living host is still the seat payload |
| `payload_trust=eliteew-runtime` | First-party Runtime is the payload (target) |
| `payload_trust=engine-from-source` | Only when engine claim is true |

**Forbidden:** claiming from-source / “full OS” while the seat still runs `tauri-living`. Labels must match the **active** dual-flag profile.

---

## Pre-activation floor sketch

Until activation/enroll completes, floor egress stays **deny-by-default** with narrow exceptions (DNS/DHCP/NTP + documented enroll endpoints). No secrets in ISO. Post-activation transport policy is Runtime-owned (F8).

EmpiierOS does not ship nft overlays in this wave — kinship owns living overlays; OS line only records the contract.

---

## Session plane ownership

`session/` in EmpiierOS documents (and later implements contracts for):

- Seat bootstrap honesty (Identifying-until-resolved)
- Edition / profile awareness (`edition.profile`)
- What the OS Shell profile must declare vs kiosk profile
- Explicit **non-goals**: GNOME DE. LinkornEW (`linkornew`) is a separately scoped staff profile, not an admin switch inside the FoFline image.

Kiosk break-glass / tty policy stays kinship — do not re-home without operator task.

---

## Air egress note (policy only — not BR port)

Living EliteEW prelicense substrate may allow TCP **62200–63200** to a resolved
product host for Air enroll, and TCP **80/443** only to portal VIP
`10.88.89.1` after WireGuard is up (SPA OTA). EmpiierOS Runtime consumes those
paths through allowlisted helpers; Runtime itself keeps `IPAddressDeny=any`.
Do **not** import Buildroot/nft trees here. Empty resolve ⇒ Air fail-closed.
Do **not** open public-VIP `443`. Orthogonal to `kernel_offline`.

---

## Delivery path (current)

```text
EmpiierOS packager → overlay under ELITEEW_TS_WORK_ROOT
        │
        ▼
Monorepo Trusted Substrate factory
  EmpiierOS payload profile · dual-flag fail-closed
  living tauri/kiosk profile untouched
        │
        ▼
official trusted-substrate/out/ · QEMU smokes
```

Full `selftest-secure-reset.sh` still requires the missing kinship readiness
document `2026-09-08-trusted-substrate-mit-disk-installer.md`. Do not invent
that file from EmpiierOS. Parallel line: copy → edit in EmpiierOS; soft-retire
the living host only on journey parity.

---

## Explicit DROP

- Editing living monorepo kiosk/Tauri payload “for OS readiness” from EmpiierOS sessions
- Forking `trusted-substrate/` into EmpiierOS as a second factory
- Dual-active payload profiles on one image
- Planning OS caps as Tauri plugins that “become” the floor
- Fake `engine-from-source` / full-OS claims under `tauri-living`
- Flipping default kiosk off without operator + dual-flag profile ready

---

## Relation

| Artifact | Role |
|----------|------|
| This file | **Living F10 SoT** — substrate kinship contract |
| [`SECURITY-PRIVACY-DEFAULTS.md`](SECURITY-PRIVACY-DEFAULTS.md) | F8 Runtime/floor trust |
| [`RELEASE-SIGNING-STUBS.md`](RELEASE-SIGNING-STUBS.md) | F9 channels / signing |
| [`../KINSHIP.md`](../KINSHIP.md) | Read-only kinship map |
| `docs/capture/13` | Forensic evidence |
| Product substrate profile wave | Historical P62 record is in `trash/`. Production authority is the Unix IPC registry in `runtime/src/core.rs`, not `GET /v0/substrate/profile`. |

When a real OS payload profile is tasked, update this file **same phase** as profile labels and delivery docs (R7).
