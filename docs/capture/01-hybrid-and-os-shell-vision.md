# Hybrid vision — React as EliteEW OS UI

**Capture date:** 2026-09-11

## One-sentence target

EliteEW is a hybrid platform: **React as the single product UI layer of the OS**, **Linux as the trusted runtime floor**, and a **first-party Runtime as the privileged seam** between them. Tauri may serve as the seam today; in the future, the seam will not be called "Tauri."

## Two planes (do not collapse)

| Plane | Question | Lives in |
|-------|----------|----------|
| **Ops / execution** | How to safely execute a privileged operation? | Linux services + EliteEW Runtime (bridge) |
| **Shell / OS UI** | What does the human see as the OS "desktop"? | React (OS Shell mode) |

The mechanics of operation execution are relatively clear (floor + allowlisted bridge).  
Open design: how to make the current EliteEW React **OS session UI** an **OS shell mode**, rather than a new UI product from scratch.

## Frontend meaning (OS Shell mode of existing DNA)

Not "drawing a different application." The meaning for frontend:

- system status / network / updates / identity — **honest OS states**;
- no **browser assumptions** (address bar, "open in new tab" as a world model);
- apps = **Kernel registry / manifests**, not an arbitrary Linux desktop;
- offline / permission / recovery — like a **product**, not a sysadmin DE;
- seed = a narrow copy (tokens, chrome, 1–2 surfaces) from Desktop Kernel DNA — not the entire `shared/web/frontend`.

## Twin risks (must hold both)

| Risk | Failure mode | Guard |
|------|--------------|-------|
| **Skin** | React draws everything; system actions are scattered across Tauri/plugins/shell | Every OS capability = shell surface **+** bridge op **+** substrate policy |
| **GNOME-bloat** | Shell expands into a general-purpose DE | Only EliteEW journeys; thin WM on the floor; manifests, not an app zoo |

## Maturity criterion (normative)

A new OS capability is planned as:

```text
shell surface  +  bridge op  +  substrate policy
```

**Not** as: "add a Tauri command / plugin."

## React "on par with KDE/GNOME" — in what sense

**Yes (role as session shell):**

- one visual world for the entire seat;
- Start / app surfaces / statuses / recovery — in React;
- no second "real" desktop underneath it;
- uniqueness = only EliteEW journeys, not a general-purpose DE.

**No (GNOME/KDE clone):**

- do not bring an zoo of arbitrary desktop apps;
- React does not become a Wayland compositor;
- a thin WM (labwc or a descendant) remains on the floor;
- the specificity of EliteEW OS is an advantage, not a deficiency.

## Layer sketch (future-facing, no Tauri in normative names)

```text
L-session   Linux: seat, compositor/WM policy, getty/router, power, net policy
L-bridge    EliteEW Runtime: ops IPC (first-party)
L-shell     React: OS Shell (from EliteEW frontend DNA via copy-paste-edit)
L-apps      Desktop-app-like surfaces inside shell (manifests), not GNOME apps
```

## Relation to Trusted Substrate (today)

Trusted Substrate is already moving toward a sealed floor + product payload (currently kiosk → Tauri living).  
EliteEW OS Shell is a **future separate payload / session face**, not an urgent breaking of the kiosk profile.

Substrate kiosk profile and OS Shell profile must coexist (dual-flag fail-closed) when we reach the payload wave — **not** during the foundation coding rush.

## Conscious duplication (seed era)

At the start, **duplication is conscious** (copy-paste-edit).  
Move common packages / monorepo-shared **later**, once the OS Shell contract is stable — otherwise, we will just stitch lines together and break the "living" version for the sake of "beauty."

v0 seed: **do not** port WG / license / Capsule ops to a new host — use honest stubs in the UI ("channel through Runtime later"), otherwise the new version will immediately become a second Tauri.

## Parked product-doc artifacts (after foundation lock — still text, not code)

| Artifact | Content |
|----------|---------|
| **WHERE OS LIVES** | [`WHERE-OS-LIVES.md`](../WHERE-OS-LIVES.md) — landed F7 thin |
| **EliteEW OS Shell v0 SoT** | [`OS-SHELL-V0-SOT.md`](../OS-SHELL-V0-SOT.md) — landed F7 |
