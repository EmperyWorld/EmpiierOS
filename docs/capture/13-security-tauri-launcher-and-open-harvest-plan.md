# Security + Tauri launcher + open useful-findings plan

**Capture date:** 2026-09-11  
**Status:** F1.10–F1.12 **executed** 2026-09-13 (Tracks S/L/O read-only). Zero edits to living `connection-agent` / kiosk. Stance `02` unchanged — Tauri ≠ plan axis.  
**Purpose:** (1) study the **security system** of the living EliteEW; (2) study the **Tauri launcher** and living host as a strong exemplar; (3) leave an **open mandate** for plan implementers — record everything still useful for the EmpiierOS development foundation.

**Stance lock (must not drift):**

- Study Tauri / security **as kinship / living history** — do **not** break the living product.  
- Future plan axis remains **OS Shell + EliteEW Runtime + Linux/Substrate** ([`02-tauri-stance.md`](02-tauri-stance.md)).  
- Harvest **mechanisms** (capabilities, boundaries, launcher journey, honesty labels) → rewrite for Runtime/Shell — not “build OS through Tauri plugins.”

Companions: [`09`](09-genew-harvest.md)–[`12`](12-trash-and-parked-products-forensic-plan.md).

---

## Hard constraints

| Allowed | Forbidden |
|---------|-----------|
| Read docs + code under monorepo (security, connection-agent, Substrate) | Edits to living Tauri/FE/kiosk “under OS” |
| Append findings only to EmpiierOS capture | New capabilities designed as `tauri command` / plugin SoT |
| Map security → OS Runtime / session / Shell planes | Port full Tauri host into EmpiierOS day-1 |
| Opportunistic discoveries during F1 (see F1.12) | Scope creep into product coding / seed React |

---

## Track S — Security system study (F1.10)

### Goal

Understand how EliteEW currently handles trust, boundaries, allowlists, enrollment, and policy — to establish **security defaults** and a mental model for the OS Runtime / session (later F8), without copying Tauri vocabulary as the SoT.

### Primary sources (read-only)

| Topic | Path (under monorepo) |
|-------|------|
| Desktop security boundaries | `documentation/product/desktop/DESKTOP-CLIENT-SECURITY-BOUNDARIES.md` |
| Tauri connected client | `documentation/product/desktop/DESKTOP-TAURI-CONNECTED-CLIENT.md` |
| Atomic conditional access | `documentation/product/desktop/DESKTOP-ATOMIC-CONDITIONAL-ACCESS.md` |
| Air enrollment | `documentation/product/desktop/DESKTOP-CLIENT-AIR-ENROLL.md` |
| FoFline edition | `documentation/product/desktop/DESKTOP-FOFLINE-EDITION.md` |
| EBrowser / policy | `documentation/product/desktop/DESKTOP-EBROWSER.md` |
| Trusted Substrate floor | `documentation/product/desktop/ELITEEW-TRUSTED-SUBSTRATE.md` + PLAN |
| Agent rule / skill / matrix | `.cursor/rules/tauri-security-boundary.mdc` · `eliteew-tauri-security/` · `security-matrix.md` |
| Reviewer pattern | `.cursor/agents/tauri-security-reviewer.md` |
| Code (names only) | `pack-organization/connection-agent/src-tauri/` capabilities / build |
| Substrate tree | `pack-organization/trusted-substrate/` |

### Exit S

- [x] Security one-pager (OS planes wording) — §Findings Track S  
- [x] KEEP/HARVEST/DROP: mechanisms vs Tauri-specific APIs  
- [x] Inputs to future **F8** + F4 honesty  
- [x] Explicit: Tauri security skill/hooks = **pattern harvest**, not day-1 port  

---

## Track L — Tauri launcher & living host study (F1.11)

### Goal

Study the **launcher journey** and the architecture of the living host: session start, windows, chrome, packaging, updater — as a **strong implementation** that must not be broken; rewrite the lessons for **OS Shell session start + Runtime**, not as "the next Tauri app."

### Primary sources (read-only)

| Topic | Path |
|-------|------|
| Launcher journey | `DESKTOP-CLIENT-LAUNCHER-JOURNEY.md` |
| Shell lifecycle / chrome | `SHELL-LIFECYCLE-AND-CHROME-SSOT.md` |
| App registry | `DESKTOP-SHELL-U1-APP-REGISTRY-DESIGN.md` |
| Taskbar / windows | `DESKTOP-TASKBAR-ARCHITECTURE.md` (+ window-* peers) |
| Kernel offline mount | `DESKTOP-KERNEL-OFFLINE-MOUNT-BRIDGE.md` |
| Version / OTA SoT | `DESKTOP-CLIENT-VERSION-SOT.md` |
| Tree | `pack-organization/connection-agent/` |
| Stance | EmpiierOS [`02-tauri-stance.md`](02-tauri-stance.md) |

### Exit L

- [x] Launcher/session one-pager + Shell/Runtime rewrite map — §Findings Track L  
- [x] Do-not-break living surfaces list  
- [x] Explicit DROP: planning new OS features as Tauri commands  
- [x] Inputs to `session/` / `runtime/` naming + F6 seed (chrome/tokens only)  

---

## Track O — Open useful-findings mandate (F1.12)

### Goal

F1 implementers **are required** to record everything still useful for the foundation, even outside the predefined tracks.

### Exit O

- [x] ≥5 opportunistic findings logged with tags  
- [x] Triaged into AGENT-LESSONS or product-later  
- [x] No silent “we saw it but didn’t write it”  

---

## Findings (executed 2026-09-13)

### Track S — Security one-pager (OS planes)

```text
Linux floor (Trusted Substrate / host OS)
  nft pre-license egress · kiosk session · payload slot (tauri-living today)
        │
EliteEW Runtime (privileged Rust — kinship: connection-agent)
  keyring · typed egress · license gate · caller context · retail XOR ops
        │ IPC allowlisted ops only
   ┌────┴────┐
   │         │
 Shell     Session / identity
 main/hub  device+capsule · atomic bind · DPoP
   │
 Shell — product WebView (hostile SPA)
   lean ACL · NO oidc/license/fs/capsule/wg
   │
 Product / cloud (API, KC, edge) — over W-transport / WG
```

| Kinship surface | EmpiierOS plane |
|-----------------|-----------------|
| Launcher / hub / product chrome | `shell-ui/` |
| Privileged Rust + keyring + egress | `runtime/` |
| Seat / atomic bind / preflight contracts | `session/` |
| Buildroot / nft / payload profile | Linux floor |

**Mechanisms (KEEP / HARVEST):**

| Item | Class | EmpiierOS rewrite |
|------|-------|-------------------|
| Main vs product IPC + caller context | KEEP | Runtime bridge + surface ACL |
| Default-deny command inventory + compile-time retail/ops XOR | HARVEST | Runtime op manifest; drop Tauri AppManifest |
| Per-window capability ACL | HARVEST | Session capability contract (not Tauri JSON SoT) |
| Threat model: product WebView = attacker | KEEP | Shell-hosted web surfaces |
| Rust-only secrets + typed egress + kill-switch | KEEP | Runtime secret/network policy |
| Policy SoT in Runtime (EBrowser lesson; SPA blob forbidden) | KEEP | Align Dual SoT scar `f364829a5` |
| Action tickets + native confirm | KEEP | Destructive op auth |
| Opaque file grants / safe extract / quotas | KEEP | Runtime I/O policy |
| Atomic auth / Air enroll / FoFline pack | HARVEST | Session/floor later — not day-1 |
| Substrate dual-flag payload + OS-before-payload updates | HARVEST | F8 / F10 |
| F-01…F-13 security matrix | KEEP | Kinship checklist → Runtime test families later |
| System WG / pkexec / Tauri plugins as SoT | DROP | Ops-only or host kinship |
| `tauri command` / invoke as OS capability language | DROP | Runtime IPC contract instead |
| Full tauri-security-gate.py + Gemma-style zoo | DROP day-1 | Pattern only (reviewer workflow) |

**Honesty labels (harvest):** `protected` ≠ Start OK; `socks_listening` ≠ L7; W-transport ≠ “invisible to ISP”; FoFline ≠ air-gap; payload_trust tags (`tauri-living` / `engine-from-source`).

**F8 inputs (when tasked):** default-deny bridge registry; attacker model; secret taxonomy; policy locus = Runtime; compile-time profiles; honesty label registry; sensitive-action tickets; file caps; update trust; Start fail-closed; session atomic bind north star; floor pre-activation egress sketch; F-matrix → light test families.

**Pattern harvest only (not day-1 port):** monorepo `eliteew-tauri-security` skill + `security-matrix.md` + `tauri-security-reviewer` agent + `tauri-security-gate.py`.

---

### Track L — Launcher → Shell/Runtime rewrite map

| # | Living launcher step | OS map |
|---|----------------------|--------|
| 1 | Process start (RPM/NSIS) | Floor / seat process |
| 2 | Identifying (brand; no drop zone) | Shell bootstrap gate |
| 3 | Native boot resolve (ping + channel + entitlement) | Runtime probes; Shell waits typed state |
| 4 | Licensed → Hub / else Welcome+Activation | Session policy + Shell route |
| 5 | Welcome slides (first unlicensed) | Shell onboarding prefs |
| 6 | Air or USB `.ecap` (Rust-only; mutex) | Runtime enroll; Shell carrier UX |
| 7 | Hub chrome | Shell pre-product chrome |
| 8 | Host "Network" panel (not license-gated) | Runtime `host_network_*`; Shell panel |
| 9 | Start CTA (live preflight) | Runtime license.preflight; fail-closed UI |
| 10 | `product_open` typed target | Runtime `product.open` — not URL from JS |
| 11 | Product OIDC in product WebView | Runtime auth target; Shell product window |
| 12 | Releases + updater (notes suppress silent OTA) | Runtime update.*; Shell consent |
| 13 | Local factory reset → restart → Activation | Runtime wipe orchestrator; device-local only |
| 14 | Ops 7-tap | Runtime ops profile only |

**Two shells — do not conflate:** Tauri launcher+product WebView ≠ browser Desktop SPA shell (`shared/web/.../Desktop/`). Both inform EmpiierOS; neither is GNOME DE.

**Do-not-break living surfaces (kinship guards):** Journey W1–W7; Identifying-until-resolved; Capsule v3; retail sealed UI; F-04 Zero JS; main vs product ACL; W-transport default retail; Start honesty (NCR-004); local factory reset order; VERSION SoT mirrors; CHANNEL_MATRIX; Connected≠FoFline RPM conflict; browser registry/portal locks; CORS for native WebView; WebKit graphics layers.

**DROP (plan axis):** new OS caps as `#[tauri::command]` / plugins; wholesale port of `connection-agent`; in-place living rewrite “for OS”; retail system-WG as default; GNOME/NM GUI for host net; tray/updater env trust downgrades.

**`session/` naming seeds:** `bootstrap.phase` · `onboarding.*` · `entitlement.present` · `host_network.available` · `edition.profile` · `device.identity` · `enroll.welcome_policy`.

**`runtime/` op family seeds:** `enroll.*` · `transport.*` · `license.*` · `product.*` · `auth.*` · `host_network.*` · `seat.local_factory_reset` · `update.*` · `lifecycle.restart` · `fs_bridge.*` · `kernel_offline.*`.

**F6 seed allow (chrome/tokens only):** launcher `tokens.css` / chrome layout patterns / journey FSM **helpers**; browser `--desktop-*` tokens. **Deny:** full connection-agent, SPA Desktop tree, capabilities JSON, signing scripts, OIDC wiring.

---

### Track O — Open log

_Format: `FOUND | DATE | PATH | why | wave | KEEP/HARVEST/DROP`_

FOUND | 2026-09-13 | `DESKTOP-CLIENT-SECURITY-BOUNDARIES.md` + `security-matrix.md` | F-01…F-13 + command classification living SoT | F8 | KEEP  
FOUND | 2026-09-13 | `connection-agent/src-tauri/capabilities/*.json` + caller_context | Main/product ACL + defense-in-depth | F8 / runtime | HARVEST  
FOUND | 2026-09-13 | `ebrowser_policy_fetch.rs` / `DESKTOP-EBROWSER.md` | Policy SoT must be Runtime-fetched; SPA blob forbidden | F8 | KEEP  
FOUND | 2026-09-13 | `trusted-substrate/policy/payload_profile.py` | Dual-flag exclusivity (electron XOR tauri) | F10 | HARVEST  
FOUND | 2026-09-13 | `trusted-substrate/policy/update_channels.py` | OS-before-payload update precedence | F8/F9 | HARVEST  
FOUND | 2026-09-13 | `DESKTOP-CLIENT-LAUNCHER-JOURNEY.md` + `src/lib/launcher/journey/` | Boot FSM; Identifying-until-resolved; Vitest-locked pure helpers | shell-ui / session | HARVEST  
FOUND | 2026-09-13 | `DESKTOP-CLIENT-VERSION-SOT.md` + `release-state.v1.json` | VERSION mirrors + CHANNEL_MATRIX honesty (lag/omitted) | F9 | KEEP  
FOUND | 2026-09-13 | `SHELL-LIFECYCLE-AND-CHROME-SSOT.md` + `appRegistry.catalog.ts` | One registry line per app; portal sole-owner | shell-ui | HARVEST  
FOUND | 2026-09-13 | `license_local_factory_reset` result shape | Fail-closed wipe `{ok,cleared,skipped,failed}` | runtime | HARVEST  
FOUND | 2026-09-13 | Host "Network" vs W-transport split | Host NM ≠ tunnel; mutate main-only | runtime | KEEP  
FOUND | 2026-09-13 | `DESKTOP-KERNEL-OFFLINE-MOUNT-BRIDGE.md` | Offline mount policy package pattern | runtime / F6 | HARVEST  
FOUND | 2026-09-13 | Packaging signing key-role separation | Entitlement/denylist/metadata/artifact keys never mixed | F9 | KEEP  

**Triage:** lessons #38–42 in `AGENT-LESSONS.md`; packaging/OTA → F9 product-later; FoFline SPA pack / Meet stage → product-later; no product code from these finds.

---

## Exit criteria (F1.10–F1.12 done)

- [x] Security one-pager + KEEP/HARVEST/DROP (Track S)  
- [x] Launcher/host one-pager + rewrite map to Shell/Runtime (Track L)  
- [x] Open log has real findings; triaged (Track O)  
- [x] Folded into AGENT-LESSONS  
- [x] Stance check: no new plan text that makes Tauri the normative OS axis  
- [x] Zero edits to living connection-agent / kiosk required by this study  

---

## Relation to foundation waves

| Wave | Feed |
|------|------|
| **F1.10** | Track S security |
| **F1.11** | Track L Tauri launcher / living host |
| **F1.12** | Track O open mandate |
| F2 | `session/` / `runtime/` informed by rewrite map |
| F4 | Honesty, boundaries, independent security-reviewer **pattern** (thin) |
| F5 | Preflight may require security+launcher notes present |
| F6 | Seed allowlist: chrome/tokens only — not whole Tauri host |
| F8+ | Security/privacy defaults from Track S |
| F9+ | Signing/OTA stubs from opportunistic packaging finds |
| F10 | Substrate kinship / payload dual-flag |

See [`04-foundation-waves.md`](04-foundation-waves.md).
