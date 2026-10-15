# Tauri stance

**Capture date:** 2026-09-11

## Lock (operator)

- Do **not remove** Tauri for now.  
- The implementation in `connection-agent` is robust and living — it is a valuable carrier and historical asset.  
- When **planning and searching for future solutions**, do **not include** Tauri as the central axis: focus on "shell + runtime + Linux" rather than "how to do it via Tauri."  
- A living host may eventually become legacy; until then, do not break it.

## What Tauri actually is for us today

It is not the soul of the product. It is a ready-made **host**:

- windows / multi-webview;
- IPC + capabilities;
- integration with WebKitGTK / tray / updater packaging;
- RPM/NSIS on third-party OS (Fedora seats, etc.).

Business logic (license, WG/W-transport, Capsule, FoFLine gates) is primarily **our Rust**, not a "Tauri product."

## Why it felt necessary historically

Until a from-source OS existed, an application container was needed on third-party Linux/Windows → Electron/Tauri.  
With Trusted Substrate, we have our own: package allowlist, own session, own policy — we can aim for the **EliteEW Runtime** on our own floor.

## Honesty labels (mental model)

| Era | Label (parked wording) |
|-----|------------------------|
| Now | living host = Tauri (do not break) |
| Target host | EliteEW Runtime + OS Shell |
| Engine north star | web engine from-source when ready (kinship to Substrate W8 thinking) |

## Planning rule

In foundation docs, plans, and wave exits of this line:

- normative path: **OS Shell + Runtime + Substrate/Linux**;  
- Tauri: only in the "living carrier / migration later / do not break" section;  
- do not design new capabilities as `tauri command` / plugins.

## Study mandate (not plan axis)

Studying the living Tauri launcher / host / security **is permitted and encouraged** (F1.11 / F1.10) as kinship read-only — see [`13-security-tauri-launcher-and-open-harvest-plan.md`](13-security-tauri-launcher-and-open-harvest-plan.md).  
Studying ≠ "the axis of future solutions." Rewrite conclusions in terms of Shell + Runtime.
