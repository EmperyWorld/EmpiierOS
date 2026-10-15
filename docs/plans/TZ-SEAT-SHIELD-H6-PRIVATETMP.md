# TZ — Seat Shield H6 + PrivateTmp (design lock)

**Status:** Binding design / contract. **Not** acceptance. **Not** MATCH.
**Date:** 2026-09-21
**Product:** EmpiierOS Seat Shield floor (`seat.shield.*` + Runtime unit + watch helper)
**Parent:** [`TZ-SEAT-SHIELD-HARDENING.md`](TZ-SEAT-SHIELD-HARDENING.md) · [`TZ-SEAT-SHIELD-S5C.md`](TZ-SEAT-SHIELD-S5C.md)
**Living contract:** [`../../session/SEAT-SHIELD.md`](../../session/SEAT-SHIELD.md)
**Twin:** `security.integrity.*` stays **disabled**.

This wave is **docs + preflight needles only**. No bake. No reinstall. No Runtime unit flip.
Full H6 measure closes only after a **fresh install** from an image whose systemd is `+SECCOMP`.

---

## 1. Two debts, one root honesty

| Debt | Symptom on live lab | Real cause | Closes when |
|------|---------------------|------------|-------------|
| **H6** | Runtime journal `seccomp mode=0`; `/proc/.../status` `Seccomp: 0` despite unit `SystemCallFilter=~@debug` | Installed systemd built **`-SECCOMP`** / no usable libseccomp for pid1 | Fresh install of kinship image that already has `BR2_PACKAGE_LIBSECCOMP=y` and QEMU-proven `+SECCOMP` |
| **PrivateTmp split** | `seat.shield.scan_path` on host `/tmp/...` denied or silent; watch journal still sees creates | Runtime unit **intentionally** `PrivateTmp=true`; watch unit `PrivateTmp=false` | Never “fixed” by flipping Runtime. S5C helper `sha256` is the bridge for IOC-on-drain |

Do not conflate them. Muxing a new Runtime binary does not attach systemd filters. Flipping Runtime `PrivateTmp=false` is not hardening.

---

## 2. Cause

### 2.1 H6

1. Kinship and official bake already select `BR2_PACKAGE_LIBSECCOMP=y`.
2. QEMU on that bake shows systemd `+SECCOMP` and Runtime `seccomp mode=2 filters=28`.
3. The living lab seat still runs the **older install** whose systemd feature string is `-SECCOMP`.
4. `SystemCallFilter=~@debug` in the unit file is present and ignored. Redeploying the unit or muxing Runtime cannot change pid1.

### 2.2 PrivateTmp

1. Runtime must keep a private `/tmp` so hostile host drops are not ambient in the privilege plane (`IPAddressDeny=any`, `CAP_NET_ADMIN`, bridge SoT).
2. Watch helper must see host `/tmp` `/var/tmp` `/dev/shm` to journal creates (S5B).
3. Those are **different mount namespaces**. Sharing them would punch the Runtime jail.
4. Therefore Runtime cannot open many host `/tmp` paths for `scan_path` / `hold_file`. That is expected, not a crate bug.
5. S5C puts `sha256` on the watch journal so drain can IOC-match when Runtime cannot open the file. Auto-hold still requires a path Runtime can open (typically `/dev/shm`).

---

## 3. Lock (durable law)

### 3.1 H6

1. Keep `SystemCallFilter=~@debug` on `empiieros-runtime.service` (kinship overlay).
2. Keep kinship `BR2_PACKAGE_LIBSECCOMP=y`. Do not invent a Rust `seccompiler` rewrite of the unit filter.
3. Do **not** claim H6 closed on a seat that still reports systemd `-SECCOMP` or Runtime `Seccomp: 0`.
4. Do **not** green H6 from mux-only evidence.
5. WebKit bubblewrap / `ENABLE_BUBBLEWRAP_SANDBOX` remains **separate** image debt — not this TZ.

### 3.2 PrivateTmp split

1. Runtime unit: **`PrivateTmp=true`** (required).
2. Watch unit: **`PrivateTmp=false`** (required). Comment in unit must forbid sharing Runtime’s namespace.
3. Do **not** set Runtime `PrivateTmp=false` to make `scan_path /tmp` pass.
4. Do **not** give the watch helper `CAP_NET_ADMIN` or nft verbs.
5. Vanished / Runtime-invisible allowlisted paths → **silent skip**, not `file_scan_denied` (S5C).
6. Journal `sha256` may feed IOC; it must **not** move a file by itself.

### 3.3 Preflight needles (this wave)

EmpiierOS `scripts/preflight-foundation.sh` must assert:

- watch unit contains `PrivateTmp=false` (already)
- kinship Runtime unit contains `PrivateTmp=true`
- kinship Runtime unit contains `SystemCallFilter=~@debug`

Adversarial: flipping either PrivateTmp value or deleting the filter needle must FAIL preflight.

---

## 4. Measure

### 4.1 This wave (design lock — no reinstall)

```bash
scripts/preflight-foundation.sh   # exit 0; needles above
```

Diary line records Cause / Lock / Measure. Verdict stays `NOT_READY`. H6 stays **blocked on live**.

### 4.2 After operator resumes install (full H6 close — later)

On a **fresh** seat from the current official image (not mux):

1. `systemctl --version` / feature string includes `+SECCOMP` (not `-SECCOMP`).
2. `systemctl show empiieros-runtime.service -p SystemCallFilter` is non-empty / applied.
3. Runtime journal after ready: `seccomp mode=2` (filters > 0).
4. `/proc/$(systemctl show -p MainPID --value empiieros-runtime)/status` → `Seccomp: 2`.
5. Watch: `systemctl show empiieros-seat-shield-watch.service -p PrivateTmp --value` → `no`.
6. Runtime unit file on disk still `PrivateTmp=true`.
7. Plant under host `/tmp`: watch journal may carry `sha256`; Runtime `scan_path` may still deny open — that remains honest under the split.
8. Auto-act / auto-quarantine remain off unless the operator arms them.

Until those metal numbers exist, H6 is **not closed**.

---

## 5. Out of scope

- Lab reinstall / USB plant / official bake
- Runtime `PrivateTmp=false`
- Integrity twin / notify on `/etc`
- Landlock / AF_PACKET / musl
- ENABLE_ACTS / ENABLE_QUARANTINE on glass
- Claiming MATCH because QEMU once showed `mode=2`

---

## 6. Operator phrase (full H6 later)

> Unlock metal reinstall from current official `out/` to measure Seat Shield H6 (+SECCOMP). No Runtime PrivateTmp flip.

Design lock phrase for this turn (already chosen as option 3):

> EmpiierOS write for Seat Shield H6/PrivateTmp design lock only (docs + preflight). No bake. No reinstall.
