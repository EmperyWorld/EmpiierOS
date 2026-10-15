# EmpiierOS / EliteEW OS

**License:** [MIT](LICENSE) for first-party code. Third-party components keep
their upstream licenses; see [NOTICE](NOTICE).

**Status:** EmpiierOS preflight is green; verdict `NOT_READY`. Official-image
QEMU smokes were **not rerun** after the 2026-09-20 LUKS bake (prior H1 FAIL
on the WebKit sandbox string still stands). Full Trusted Substrate
`selftest-secure-reset.sh` is red on a missing kinship readiness document.
**Home:** this repo
**Started:** 2026-09-11 (session in `powered_by_emperyworld`; capture moved here same day).
**Rule:** evidence outranks historical wave labels. Unscoped product expansion
stays frozen. Living source: FoFline host (launcher, HUD, Seat Browser,
Seat Shield), V2 install (LUKS2 state, PIN ≠ volume key), Wi-Fi unit
`AF_INET` plant, no lab `/30` Settings defaults. On 2026-09-22 the
factory `images/rootfs.tar.gz` and `out/rootfs.tar.gz` are the same file
hash `cf0b46e1…` (matches the stamp). `5979e9cf…` is the 2026-09-20 bake
recorded in the ledger, not the archive on disk now. Metal is still not
accepted. Verdict stays `NOT_READY`. `product.open` stays fail-closed
without unlock + pack + native OIDC.
Glass map: [session/SEAT-FACE.md](session/SEAT-FACE.md). Install:
[session/OS-INSTALL.md](session/OS-INSTALL.md).

**Short SoT:** [docs/NORTH-STAR.md](docs/NORTH-STAR.md) · [docs/WHERE-OS-LIVES.md](docs/WHERE-OS-LIVES.md) · [docs/OS-SHELL-V0-SOT.md](docs/OS-SHELL-V0-SOT.md) · [docs/FOUNDATION-STATUS.md](docs/FOUNDATION-STATUS.md)

**Engineering bar (registry, not a filing):** [docs/engineering-bar/ENGINEERING.md](docs/engineering-bar/ENGINEERING.md) · [CONTRIBUTING.md](CONTRIBUTING.md). Status stays `NOT_READY`.

## Current authority

- [docs/NORTH-STAR.md](docs/NORTH-STAR.md)
- [docs/FOUNDATION-STATUS.md](docs/FOUNDATION-STATUS.md)
- [docs/plans/MATURITY-LEDGER.md](docs/plans/MATURITY-LEDGER.md)
- [session/SEAT-FACE.md](session/SEAT-FACE.md)
- [session/OS-INSTALL.md](session/OS-INSTALL.md)
- [runtime/HEALTH-IPC.md](runtime/HEALTH-IPC.md)
- [.cursor/orchestration/reports/deep_secure_reset_audit.md](.cursor/orchestration/reports/deep_secure_reset_audit.md)

## Purpose

Зафиксировать мышление про **EliteEW OS** как отдельную продуктовую линию:

- гибрид React (лицо ОС) + Linux (пол / исполнение операций);
- не ломать living Tauri / cloud SPA / Trusted Substrate kiosk в monorepo;
- новое = copy → paste → edit в **этом** контуре;
- фундамент (repo, identity, ignores, agent kernel, lessons) важнее спешки кодить UI.

## New chat / handoff

**Start here:** [START-HERE.md](START-HERE.md) — reading order + **copy-paste bootstrap prompt**.
**Compressed arc:** [docs/capture/SESSION-SUMMARY-2026-09-11.md](docs/capture/SESSION-SUMMARY-2026-09-11.md)
**Full dialogue export:** [docs/capture/SESSION-DIALOGUE-2026-09-11.md](docs/capture/SESSION-DIALOGUE-2026-09-11.md)

## Unified foundation plan pack

| File | What |
|------|------|
| [docs/NORTH-STAR.md](docs/NORTH-STAR.md) | Short axis + twin risks + maturity |
| [docs/WHERE-OS-LIVES.md](docs/WHERE-OS-LIVES.md) | Layout + non-goals + seed + kinship |
| [docs/foundation/FOUNDATION-MASTER-PLAN.md](docs/foundation/FOUNDATION-MASTER-PLAN.md) | Unified waves, exits, checklist, risks |
| [docs/foundation/PROJECT-TREE.md](docs/foundation/PROJECT-TREE.md) | Target directory tree + owners |
| [docs/foundation/AGENT-KERNEL-SPEC.md](docs/foundation/AGENT-KERNEL-SPEC.md) | Mini `.cursor` KEEP/HARVEST/DROP |
| [docs/foundation/GIT-AND-IGNORE-SPEC.md](docs/foundation/GIT-AND-IGNORE-SPEC.md) | git init / identity / ignore drafts |
| [docs/foundation/FILE-SIZE-AND-MODULARITY.md](docs/foundation/FILE-SIZE-AND-MODULARITY.md) | LOC limits, homes for docs/plans/specs |

Working capture archive: [docs/capture/](docs/capture/).

## Files in capture archive

See [docs/capture/README.md](docs/capture/README.md) for the `00`–`13` + SESSION-* inventory.

## How to extend

1. Новая мысль → дописать в подходящий файл в `docs/capture/` **или** в `docs/capture/07-open-capture.md` с датой.
2. Не превращать это сразу в «канон навсегда» без отдельного lock.
3. Продуктовый код / seed React — **после** зелёного фундамента (см. master plan).
4. Monorepo не дублирует эти файлы — только stub pointer.

## Explicit non-rush

Мы готовы потратить на фундамент столько времени, сколько нужно.
Несколько foundation waves ≠ размазывание задач; это осознанная несущая конструкция.
