# EmpiierOS — Security & privacy defaults (F8)

**Status:** Living foundation SoT — **F8 done 2026-09-13** (text defaults).  
**Not:** Runtime implementation. **Not:** Tauri capability JSON as SoT. **Not:** product unlock.  
**Sources:** `docs/capture/13` Track S · lessons #38–42 · stance `docs/capture/02-tauri-stance.md`.  
**Axis:** OS Shell + EliteEW Runtime + Linux floor.

## Purpose

Lock the **mental model and defaults** for privileged ops, secrets, policy locus, and honesty labels **before** Runtime/Shell product code. Implementers must map new bridge ops and session contracts to this file — not invent Tauri-shaped APIs.

---

## Trust model

```text
Linux floor (substrate / host OS)
  pre-activation egress · seat · payload slot
        │
EliteEW Runtime (privileged bridge)
  keyring · typed egress · license/session gates · caller context
        │ allowlisted IPC only
   ┌────┴────┐
 Shell     Session
 main/hub  identity · bind · enroll contracts
   │
 Shell — hosted web / app surface (hostile by default)
   lean ACL · no secrets · no policy enforcement blobs
```

**Attacker model:** content and IPC parameters from Shell-hosted web/app surfaces are **hostile**. Verified Runtime binary + floor keyring (within model) are trusted. Shell UI displays **redacted status** only — it does not enforce privilege.

| Plane | Security role |
|-------|----------------|
| `shell-ui/` | Surfaces + honesty UI; never secret store / policy SoT |
| `runtime/` | Bridge op registry, secrets, egress, tickets, I/O caps |
| `session/` | Seat contracts: bootstrap, edition, entitlement presence, enroll policy |
| Linux floor | Pre-activation egress, payload profile, update channel precedence |

---

## Default-deny bridge

1. Every privileged capability is a **named Runtime op** in a registry (not an open-ended host plugin surface).  
2. Unclassified ops **block release** (fail-closed).  
3. Classify each op: `main_read` | `main_user_action` | `product_scoped` | `ops_only` | `remove`.  
4. Assign each op to a **caller surface** (main/hub vs product/app vs ops). Defense-in-depth: ACL + Runtime caller check.  
5. **DROP as SoT:** `tauri command` / Tauri plugin vocabulary for new EmpiierOS capabilities.

### Suggested op families (from F1.11; names may refine)

`enroll.*` · `transport.*` · `license.*` · `product.*` · `auth.*` · `host_network.*` · `seat.local_factory_reset` · `update.*` · `lifecycle.restart` · `fs_bridge.*` · `kernel_offline.*`

---

## Policy locus

| Domain | Authoritative SoT | Forbidden |
|--------|-------------------|-----------|
| Navigation / browser policy | Runtime fetch + enforce | SPA-supplied policy blobs for enforcement |
| License / Start gate | Runtime live preflight | Product surface bypassing gate |
| Secrets / capsule / peer conf | Runtime + keyring only | Shell storage, logs, events, JS conf |
| Egress | Runtime typed egress table | Caller-supplied arbitrary URLs |
| Host network mutate | Main/hub surface only | Product mutate; GNOME NM GUI as OS path |

**Law:** privilege, egress, and navigation policy are **enforced in Runtime (and floor)** — Shell is display.

---

## Secret taxonomy (defaults)

| Class | Store | Shell may see |
|-------|-------|----------------|
| Device / capsule material | Runtime keyring | Never |
| Entitlement / refresh material | Runtime keyring | Redacted status only |
| Peer / transport conf | Runtime only | Never (Zero JS secrets) |
| User prefs (onboarding flags) | Session-local prefs | Yes (non-secret) |
| Logs / crash | Scrubbed | No secrets |

No secrets in ISO/image builds when floor ships; handoff post-install only (substrate kinship).

---

## Compile-time / profile defaults

- **Retail vs ops** (or equivalent) profiles are **mutually exclusive** at build — no runtime env flip that elevates retail → ops.  
- Ops-only surfaces (system WG, pkexec helpers, 7-tap editors) stay **out of retail** defaults.  
- Floor **payload profile** exclusivity and **OS-before-payload** update precedence → F10 / F9; do not dual-activate conflicting payload slots.

---

## Sensitive actions

Destructive or trust-changing ops (update install, factory reset, restart into wipe, enroll overwrite) require:

1. **Action ticket** (short TTL, single-use or idempotent retry contract)  
2. **Native / main-surface confirmation** (not product WebView alone)  
3. **Fail-closed result shape** (e.g. `{ok, cleared, skipped, failed}`) — no silent partial success sold as Done  

Local factory reset = **device-local** only — never implied server revoke.

---

## File / resource caps

- Opaque **TTL file grants** (not ambient path access from Shell)  
- Safe archive extract bounds  
- Streaming downloads with size quotas  
- Central Runtime I/O policy module owns caps — Shell does not negotiate wider access

---

## Network & honesty labels

| Label / signal | Means | Does **not** mean |
|----------------|-------|-------------------|
| Channel protected / L7 OK | Probe via intended tunnel path | Start / license allowed |
| SOCKS / tunnel listening | Local proxy up | End-to-end product ready |
| Host network available | Link/NM path for seat | W-transport / product tunnel |
| FoFline (when used) | Local pack + mandatory tunnel policy | Air-gap |
| `payload_trust=tauri-living` | Living host honesty | From-source engine claim |
| `payload_trust=engine-from-source` | Only when true | Marketing shortcut |

**Start CTA:** disabled until **live** license/session preflight succeeds. Channel-down ≠ “no server” mask; missing main ACL on preflight = fail-closed dead Start (document, don’t hide).

**Pre-activation floor egress (sketch):** narrow allow (DNS/DHCP/NTP + documented enroll endpoints) — deny-by-default until activation. Post-activation: product API via Runtime transport policy.

---

## Session north star (contracts)

Session plane owns (names from F1.11; refine in code later):

- `bootstrap.phase` — Identifying-until-resolved before enroll UI  
- `entitlement.present` — not a Start promise  
- `edition.profile` — connected / fofline / offline-class honesty  
- `device.identity` — survives local reset where intended  
- `enroll.welcome_policy` / onboarding prefs  

**Atomic bind** (license ∧ device ∧ channel ∧ proof) before treating tokens as authoritative — **HARVEST for later session/auth**; not day-1 Runtime code in this wave.

---

## Update / signing trust (defaults → F9 stubs)

- Signed metadata; **key roles separated** (entitlement / denylist / metadata / artifact — never mixed)  
- Channel matrix honesty: `live` / `lag` / `hold` / `omitted` — no silent cross-platform claim  
- User-visible consent before silent OTA when notes unread (Linux dual-flag lesson)  
- OS update pending **wins over** payload update when both apply (substrate lesson)

---

## Privacy defaults (OS repo)

- Prefer **redacted** status strings in Shell and logs  
- No clipboard/exfil of secrets via product surface IPC  
- Kinship study stays read-only; no copy of living key material into EmpiierOS  
- Ignore firewall already covers secrets paths (F3) — do not invent second secret trees in-repo  

---

## Test families (light — when Runtime exists)

Map kinship F-01…F-13 **ideas** to EmpiierOS checks — **not** a day-1 gate zoo:

| Family | Intent |
|--------|--------|
| ACL | Unregistered op denied; product cannot call main-only |
| Policy locus | SPA policy blob rejected for enforcement |
| Secrets | No secret strings in Shell storage/events |
| Start honesty | Start disabled without live preflight proof |
| Tickets | Destructive op without ticket/confirm fails |
| Profiles | Retail binary rejects ops-only elevation |

Day-1 machine gate remains `scripts/preflight-foundation.sh` + honesty rules. Full security-reviewer / gate.py port = **pattern only** (optional later).

---

## Explicit DROP

- Designing OS capabilities as Tauri commands/plugins  
- Porting `connection-agent` or Tauri security hook mesh wholesale  
- Gemma / full HA security stop-loop  
- Retail system-WG / pkexec as default path  
- Second-universe auth/cloud (Keycloak encyclopedia always-on)  
- Claiming “fully from-source OS security” while payload is `tauri-living`

---

## Relation

| Artifact | Role |
|----------|------|
| This file | **Living F8 SoT** for security/privacy defaults |
| `docs/capture/13` | Forensic archive + evidence |
| `docs/OS-SHELL-V0-SOT.md` | Shell surface maturity |
| [`RELEASE-SIGNING-STUBS.md`](RELEASE-SIGNING-STUBS.md) | F9 version/channel/key-role stubs |
| [`SUBSTRATE-KINSHIP.md`](SUBSTRATE-KINSHIP.md) | F10 dual-flag / floor kinship |
| `runtime/README.md` / `session/README.md` | Plane pointers |

When Runtime code lands, update this file **same phase** as bridge contracts (R7).
