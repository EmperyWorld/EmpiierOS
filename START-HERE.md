# START HERE — handoff for a new Cursor session

**Home of this capture:** this repo (EliteEW OS / EmpiierOS project root)
**Status:** EmpiierOS preflight is green. Official-image QEMU smokes are
**not current** (not rerun after the 2026-09-20 LUKS bake; prior H1 FAIL
stands). Full Trusted Substrate `selftest-secure-reset.sh` is red on a
missing kinship readiness document. Verdict `NOT_READY`. Living glass map:
[`session/SEAT-FACE.md`](session/SEAT-FACE.md). Install:
[`session/OS-INSTALL.md`](session/OS-INSTALL.md). On 2026-09-22 factory `images/rootfs.tar.gz` and `out/rootfs.tar.gz`
share hash `cf0b46e1…`. `5979e9cf…` is the previous 2026-09-20 ledger
stamp, not that file. Metal is not accepted. Verdict stays `NOT_READY`.
**F2 Task 1:** empty project skeleton dirs are in place — see plane READMEs.
**F3 Task 2:** ignores landed (`.gitignore`, `.cursorignore`, `.cursorindexingignore`).
**Task 3:** capture → `docs/capture/`; foundation pack → `docs/foundation/` (dual path closed).

---

## What to open in Cursor

1. Open workspace: **this repo**
2. Paste the **bootstrap prompt** below as the first message.
3. Monorepo `powered_by_emperyworld` — look up **read-only** when kinship needed (paths below). Do not develop OS foundation inside the monorepo.

---

## Current-session intake

Read the short living authority first:

1. `docs/NORTH-STAR.md`
2. `docs/FOUNDATION-STATUS.md`
3. `docs/plans/MATURITY-LEDGER.md`
4. `docs/WHERE-OS-LIVES.md`
5. `session/SEAT-FACE.md` — what the seat actually shows
6. `session/OS-INSTALL.md` — V2 install, LUKS2 state, PIN ≠ disk key
7. `runtime/HEALTH-IPC.md` — production Unix ops
8. `.cursor/orchestration/reports/deep_secure_reset_audit.md`

Historical capture and wave plans provide context only. They cannot promote a
capability over the maturity ledger. EmpiierOS preflight is green; QEMU
is not current; the factory selftest is red on missing kinship readiness
docs. Unscoped product expansion and P93 remain frozen. Scoped work is
the FoFline host plus install/Wi-Fi/LUKS locks — all `partial`, not
official `out/`.

## Historical bootstrap prompt (archive)

```text
Ты продолжаешь работу над EliteEW OS (EmpiierOS) — отдельной продуктовой линией
(гибрид React-as-OS-UI + Linux floor + first-party Runtime).

Workspace: this repo

Сначала прочитай в этом порядке (working capture; не ломай living monorepo product):
1) README.md
2) START-HERE.md
3) docs/foundation/FOUNDATION-MASTER-PLAN.md
4) docs/capture/00-intent-and-understanding.md
5) docs/capture/01-hybrid-and-os-shell-vision.md
6) docs/capture/02-tauri-stance.md
7) docs/capture/03-repo-isolation-and-identity.md
8) docs/capture/04-foundation-waves.md
9) docs/capture/05-lessons-to-harvest.md
10) docs/capture/06-non-goals-and-guards.md
11) docs/capture/08-relationship-to-monorepo.md
12) docs/capture/SESSION-SUMMARY-2026-09-11.md
13) docs/capture/SESSION-DIALOGUE-2026-09-11.md — при необходимости деталей формулировок

Жёсткие рамки:
- Не кодить продукт / seed React, пока не скажу или пока foundation preflight не согласован.
- Не ломать living Tauri, cloud SPA, Trusted Substrate kiosk в monorepo.
- Новое живёт ЗДЕСЬ (EmpiierOS); monorepo = kinship/read-only.
- Tauri остаётся living host там, но НЕ ось будущих планов здесь.
- Дописывай идеи в docs/capture/07-open-capture.md или в тематические файлы docs/capture/.

Сейчас: подтверди, что прочитал capture, кратко перескажи понимание в 5–8 пунктах,
и спроси какой foundation wave (F0–F6+) продолжаем — или жди мой новый материал.
```

---

## Reading order (short)

| Order | File |
|------:|------|
| 1 | [README.md](README.md) |
| 2 | [docs/NORTH-STAR.md](docs/NORTH-STAR.md) · [docs/WHERE-OS-LIVES.md](docs/WHERE-OS-LIVES.md) · [docs/OS-SHELL-V0-SOT.md](docs/OS-SHELL-V0-SOT.md) |
| 3 | [docs/foundation/FOUNDATION-MASTER-PLAN.md](docs/foundation/FOUNDATION-MASTER-PLAN.md) — **unified foundation plan** |
| 4 | [docs/capture/SESSION-SUMMARY-2026-09-11.md](docs/capture/SESSION-SUMMARY-2026-09-11.md) |
| 5 | [00](docs/capture/00-intent-and-understanding.md) → [06](docs/capture/06-non-goals-and-guards.md), [08](docs/capture/08-relationship-to-monorepo.md) |
| 6 | Companion specs under [docs/foundation/](docs/foundation/) |
| 7 | [04](docs/capture/04-foundation-waves.md), [05](docs/capture/05-lessons-to-harvest.md) |
| 8 | [09](docs/capture/09-genew-harvest.md)–[13](docs/capture/13-security-tauri-launcher-and-open-harvest-plan.md) as needed |
| 9 | Append → [docs/capture/07-open-capture.md](docs/capture/07-open-capture.md) |

**Agent-practice kinship (read-only):**

- `../genEW` — orchestration that scaled
- `../powered_by_emperyworld/.cursor/` — living EliteEW agent OS (study via `10`)
- Do not develop OS foundation inside those trees.

---

## Monorepo kinship (look here when needed)

Root: `../powered_by_emperyworld`

| Topic | Path |
|-------|------|
| Trusted Substrate SoT | `documentation/product/desktop/ELITEEW-TRUSTED-SUBSTRATE.md` |
| Substrate plan | `documentation/product/desktop/ELITEEW-TRUSTED-SUBSTRATE-PLAN.md` |
| Tauri living client | `documentation/product/desktop/DESKTOP-TAURI-CONNECTED-CLIENT.md` |
| Editions / Kernel vs App | `documentation/platform/ELITEEW-PRODUCT-EDITIONS-AND-APP-PLATFORM.md` |
| Substrate tree | `pack-organization/trusted-substrate/` |
| Tauri tree (do not break) | `pack-organization/connection-agent/` |
| Security + launcher study plan (EmpiierOS) | `docs/capture/13-security-tauri-launcher-and-open-harvest-plan.md` |
| Launcher journey doc | `documentation/product/desktop/DESKTOP-CLIENT-LAUNCHER-JOURNEY.md` |
| Security boundaries doc | `documentation/product/desktop/DESKTOP-CLIENT-SECURITY-BOUNDARIES.md` |
| Agent lessons source | `.cursor/rules/`, `.cursor/hooks/`, `.cursor/skills/` |
| Agent forensic plan (EmpiierOS) | `docs/capture/10-monorepo-agent-forensic-plan.md` |
| Monorepo pointer stub | `documentation/product/desktop/ELITEEW-OS-FOUNDATION-POINTER.md` |

Transcript id of origin session: `dcd58400-257d-456d-954b-f4eeb5aadefa`

---

## Move note (2026-09-11)

Capture relocated from monorepo
`documentation/product/desktop/eliteew-os-foundation/` → **this repo**.
Monorepo keeps only a stub pointer; full files live here.
