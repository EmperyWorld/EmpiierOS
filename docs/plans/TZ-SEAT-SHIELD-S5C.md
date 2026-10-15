# TZ — Seat Shield S5C (helper hash in the watch journal)

**Status:** Binding work order. **Muxed + baked 2026-09-20.** Current official SoT `5979e9cf…` includes this slice; prior `82ac20ac…` superseded. **Not** MATCH. **Not** consumer on-access AV.
**Date:** 2026-09-21
**Product:** EmpiierOS Seat Shield (`seat.shield.*`)
**Parent:** [`TZ-SEAT-SHIELD-POST-S4.md`](TZ-SEAT-SHIELD-POST-S4.md)
**Write-home:** this repo (OS crate). Do not write `Monitoring/`.

## Cause

Live Runtime unit `dd6bb219…` keeps `PrivateTmp=true`. The watch helper must use `PrivateTmp=false` so it can see host `/tmp` and `/var/tmp`. After S5B mux, the helper journals those paths, but Runtime cannot hash them. IOC-on-drain for `/tmp` was dead unless the file also lived in `/dev/shm`.

Rewriting the live Runtime unit is out of scope. The helper must never quarantine.

## Lock

1. Watch helper hashes a file only after `resolve_allowed_file` (tmp/shm, regular, no symlink escape, ≤ 32 MiB).
2. Journal line may carry `sha256` (64 hex). Old lines without the field still parse.
3. Runtime drain uses the journal digest **only** when the path is still under a scan root and Runtime cannot open the file.
4. `/etc/passwd` (or any planted outside-root line) ignores a planted digest.
5. Auto-hold still requires `act_enabled` + `ENABLE_QUARANTINE` + `hold_file` on a path Runtime can see. A journal hash alone does not move a file.
6. Helper still never calls nft.
7. A vanished allowlisted path (`ENOENT` / `PrivateTmp`) is a **silent skip**. It must not persist `file_scan_denied`. That kind is policy-only (outside roots / symlink escape). Journal lines without `sha256` for a gone `/tmp` file are also silent.

## Measure

- Crate tests include: deserialize without `sha256`; helper hash of a tmp file; drain feed-hit from journal hash when the file is gone; planted `/etc/passwd` + digest is `file_scan_denied` and not a hit; vanished `/tmp` without hash is silent (no `file_scan_denied`).
- Lab: `/tmp` journal hash `82f08bc5…`; Runtime `scan_path /tmp` denied. Auto-act off.
- Official: `image_contract_ok` `out/rootfs.ext2` `82ac20ac…` overlay `e0a3fb8a…` watch `c40d6d74…` (stripped; live `8525c72b…`). QEMU not rerun.

## Out of scope

- Live Runtime unit rewrite / `PrivateTmp=false` on Runtime
- Helper-side quarantine
- `ENABLE_ACTS` / `ENABLE_QUARANTINE` on glass
- Integrity twin
- `fanotify` / recursive `/home`
