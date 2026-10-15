# Dependency policy

Revenue recorded for the registry pack: **0**. Foreign royalties: **0**. Do not invent contracts.

## Direct dependencies checked 2026-09-23

Shell (`shell-ui/package.json`): react, react-dom, and the dev toolchain resolve to MIT, except `typescript` which is Apache-2.0. No paid SDK.

Runtime direct crates resolve to MIT, Apache-2.0, BSD-3-Clause, ISC, or CDLA-Permissive-2.0 (`webpki-roots`). No royalty field. Transitive crates are not re-audited by this file; `cargo metadata` on a lockfile change is the next check.

## Classes

| Class | Rule |
|-------|------|
| allow | SPDX or `license-file` on the crate or npm package, no royalty, already used here |
| review | GPL/LGPL/AGPL, or a license string that is not SPDX |
| deny | paid SDK, unknown license, a prebuilt OS image checked in as a dependency |

STOP and ask a human before **deny**, and before linking a **review** crate into `empiieros-runtime` or the shell.

## Add a dependency

1. Read the upstream license field. If it is missing, do not add the package.
2. Keep `license` / `license-file` on every Cargo package and `license` on `shell-ui/package.json`.
3. Say whether the bytes ship in the OS image. A devDependency that is not bundled is not an image row, but it still cannot be unknown.
4. A paid or foreign-royalty package needs an ADR and a finance recount before the manifest changes.
5. Run `bash scripts/check-eng-bar.sh`.

## Image debts that are not in this repo

dejavu, host localedef, the Buildroot source archive, and the labwc license-file hash are Trusted Substrate debts. They are listed in the monorepo dependency policy. Do not "fix" them by vendoring fonts or glibc into this repo.
