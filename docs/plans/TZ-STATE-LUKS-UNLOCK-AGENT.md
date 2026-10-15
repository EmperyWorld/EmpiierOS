# TZ — State LUKS unlock agent (branded console, security-first)

**Status:** Design lock / execution plan. **Not** acceptance. **Not** MATCH.
**Date:** 2026-09-22 (code sync 2026-09-22, pass 2)
**Plane:** Trusted Substrate kinship (`pack-organization/trusted-substrate/`) + session contracts
**Parent:** [`TZ-DISK-AT-REST.md`](TZ-DISK-AT-REST.md) · [`../../session/OS-INSTALL.md`](../../session/OS-INSTALL.md)
**Integration:** [systemd Password Agents](https://systemd.io/PASSWORD_AGENTS/)
**Kinship SoT (apply):** `config/buildroot/external/board/eliteew-ts/rootfs_overlay/usr/local/bin/eliteew-ts-install-apply` (mirrored in `payload-templates/usr-local-bin/eliteew-ts-install-apply`)

This wave adds a **first-party MIT** console agent for the **state LUKS2**
slot (`empiieros-state` → `/var/lib/empiieros-runtime`). It does **not**
replace cryptsetup, systemd-cryptsetup, or TPM enroll from apply. It replaces
only the **default TTY prompt UX** when recovery input is required.

**Explicit non-goals:** Plymouth, Clevis/Tang, initramfs/dracut unlock,
GUI/labwc unlock (v2 may be a separate TZ), keyfiles on disk, PIN as LUKS
passphrase, network unlock.

---

## 1. Security objective

**Goal:** Minimize early-boot trusted computing base (TCB) while preserving
**fail-closed** behavior: without a valid recovery secret (or successful TPM
slot), the state volume stays closed, Runtime has no `{state}`, no consume,
no portal.

**Non-goals (honesty):** This TZ does **not** defeat evil-maid boot
substitution, shoulder surfing, or stolen-TPM + stolen-recovery together.
It reduces **software attack surface at prompt time** and **operator error**
(unclear prompt, accidental echo/log).

---

## 2. Threat model (what we defend / what we do not)

| Threat | Mitigation in scope | Out of scope |
|--------|---------------------|--------------|
| Casual shoulder surfing on tty1 | Masked input (`Echo=0`), no echo in UI copy | Physical privacy, camera |
| Secret in logs/journal | Agent never logs passphrase; apply pattern already secret-only for recovery staging | Compromised root after unlock |
| Weak/default LUKS key | Install already rejects PIN as volume key; recovery entropy rules unchanged | Operator chooses weak user secret |
| Multiple/confused prompts | Single winning agent; mask stock **console** (+ **wall**) agents | Malware after boot |
| **wall(1) leak** of unlock hint to other VTs | Mask `systemd-ask-password-wall.path` on product image | Extra VT shoulder surf |
| Large TCB at unlock (WebKit/GPU/compositor) | Console-only agent, small Rust binary | — |
| Fake prompt (phishing UI) | Signed/image SoT + optional static banner hash in agent | Attacker with full disk replace |
| Brute force on LUKS | LUKS2 + Argon2id (apply); no online rate limit in agent (disk offline) | — |
| TPM absent / PCR drift | Recovery prompt; fail closed if wrong/empty | TPM policy tuning |
| Agent crash → silent unlock | No fallback to empty password; boot stalls until recovery or TPM | — |
| Competing ask-password agents | One primary agent; disable or lose race by design | — |
| Memory retention | Minimize copies; zeroize buffer after reply; short-lived process | Cold boot RAM attacks |
| VT escape during unlock | Agent on **tty1** only; tty2–6 gettys masked (product policy) | Kernel `console=` on other VTs |

---

## 3. Architecture lock

### 3.1 Boot order (unchanged crypto — verified against overlay)

```text
GRUB (root=PARTUUID=… rootwait rw console=tty1) → plain root p2 mounted
  → systemd local-fs-pre / cryptsetup target
  → systemd-cryptsetup@empiieros_state.service
       (crypttab name empiieros-state → unit instance empiieros_state)
  → /run/systemd/ask-password/ask.* (when TPM path does not unlock)
  → [NEW] empiieros-state-unlock-agent (MIT, console on tty1)
  → /dev/mapper/empiieros-state → fstab mount /var/lib/empiieros-runtime
  → local-fs.target
  → empiieros-runtime.service
       (After=local-fs.target; drop-in RequiresMountsFor=/var/lib/empiieros-runtime)
  → getty@tty1 autologin → eliteew-ts-session-router → eliteew-ts-kiosk
```

Facts from disk:

- **No initramfs** in `eliteew_ts_x86_64_defconfig` (`BR2_TARGET_ROOTFS_*` only).
- **State partition:** GPT slot from `write_gpt_esp_root_state` — EFI 512 MiB, root ext4,
  **2 GiB LUKS2** on p3 (`STATE_MIB=2048`, `STATE_MAPPER=empiieros-state`, label
  `eliteew-state`). Root must stay ≥ 4 GiB after p3 (`min_root` check in apply).
- **crypttab** (apply `write_target_state_unlock`):
  **single line, whole file** — `empiieros-state UUID=<luks_uuid> none luks,discard`
  plus `,tpm2-device=auto` when install-time `enroll_state_tpm` succeeded.
- **Kernel:** `buildroot-build.sh` requires `CONFIG_DM_CRYPT=y` on the shipped `bzImage`
  (LUKS2 state slot is not userspace-only).
- **Live USB** boots installer session; **installed seat** has `INSTALL-COMPLETE`, no
  `live-installer`, no installer GUI — unlock agent targets **installed runtime overlay only**.

### 3.2 Agent contract (systemd Password Agents)

1. Watch `/run/systemd/ask-password/` via `.path` (`ask.*`, `IN_CLOSE_WRITE|IN_MOVED_TO`).
2. Parse `[Ask]` ini; **ignore unknown keys** (forward compatible).
3. **Allowlist** ask requests:
   - `Id=` must start with **`cryptsetup:`** (prefix match — do **not** hard-code
     `cryptsetup:empiieros-state`; systemd may include UUID or device path in `Id=`).
   - Additionally require the request belongs to the **state volume** (e.g. `Message=`
     mentions `empiieros-state`, or mapped device / UUID matches p3 installed on seat).
   - **Phase 0 measure:** on first no-TPM QEMU boot, copy one `ask.*` file into the
     diary — lock the matcher in code + unit test fixtures from that capture.
   - Refuse OpenSSL, PKCS#11, polkit, and any non-cryptsetup `Id=`.
4. Respect `Echo=`, `Silent=`, `NotAfter=`, dead `PID=` (skip stale files).
5. Reply on `Socket=` with **`+`** + passphrase (or **`-`** cancel); use
   `systemd-reply-password` or equivalent privileged reply path per spec.
6. **Never** write passphrase to disk, journal, kmsg, or crash dumps.

Reference implementation study only (do not ship as dependency):
[Raspberry Pi `cryptsetup-passphrase-agent`](https://github.com/raspberrypi/cryptsetup-passphrase-agent)
(BSD-3-Clause, **non-interactive** — protocol map only).

### 3.3 UX (console, not security boundary)

- UTF-8 (ru_RU.UTF-8 on seat), ANSI colors aligned with installer tokens
  (ink `#060A10`, lime `#24FC24`, muted text).
- Box-drawing banner; operator-facing copy: **«ключ восстановления»**,
  not «cryptsetup», not «PIN».
- **Auto recovery** (install default): prompt for **8×4 lowercase hex groups with dashes**
  (`^[0-9a-f]{4}(-[0-9a-f]{4}){7}$` — apply `generate_state_recovery` / `luksFormat`
  uses that **exact** string; agent must **not** strip dashes or upper-case unless a
  measured test proves LUKS accepts it.
- **User recovery** (operator secret at install): **12–128 characters**, not a 4–8 digit
  PIN (`valid_state_user_secret` in apply) — copy must not imply hex-only.
- **No** scroll of kernel/cryptsetup tool output in the agent UI.
- Optional: rate-limit **visible** retry messaging (LUKS still enforces crypto).

### 3.4 systemd units (kinship overlay)

| Unit | Role |
|------|------|
| `empiieros-state-unlock-agent.path` | Spawn agent when `ask.*` appears |
| `empiieros-state-unlock-agent.service` | Oneshot or simple runner; root; tty bound |
| Drop-in or mask | Mask **`systemd-ask-password-console.path`** (and evaluate **`systemd-ask-password-wall.path`**) so only the branded agent answers |
| `empiieros-runtime.service.d/state-luks.conf` | **Already planted by apply** — `RequiresMountsFor=/var/lib/empiieros-runtime`; agent must not weaken this |

Ordering hints:

- Same activation class as stock agents: `.path` on `/run/systemd/ask-password/ask.*`
  (see [Password Agents](https://systemd.io/PASSWORD_AGENTS/)). **Phase 0:** copy
  `WantedBy=` / `Before=` / `After=` / `DefaultDependencies=` from the image’s stock
  `systemd-ask-password-console.path` + `.service` (no custom ask-password units in overlay
  today — only Buildroot `BR2_PACKAGE_SYSTEMD`).
- **After** `systemd-vconsole-setup.service` (DejaVu / `ru_RU.UTF-8` — image generates
  `en_US.UTF-8` + `ru_RU.UTF-8` per `selftest-installer` / fragment).
- **Before** `getty@tty1.service`: unlock blocks `local-fs.target` until the mapper exists;
  `empiieros-runtime.service` already declares `Before=getty@tty1.service`. The agent must
  own **tty1** during the ask window so `agetty --autologin eliteew-ts` does not race the
  branded prompt (same problem stock `systemd-ask-password-console` solves).
- Unlock completes as part of **`local-fs.target`**; **`getty@tty1`** autologin and
  **`empiieros-runtime`** run **after** state mount — do not assume the kiosk XKB env
  exists during unlock; set `LANG`/`LC_ALL`/`XKB_*` on the **agent service** if early tty
  lacks Alt+Shift layout toggle.

**Console topology (overlay + apply):**

| VT | Role |
|----|------|
| tty1 | Kernel `console=tty1`, LUKS agent UI, then autologin → kiosk |
| tty2–6 | **Masked** to `/dev/null`; apply `plant_target_console_policy` + `verify_target_install_contract` (`selftest-installer` needles). No password getty on extra VTs (`OS-INSTALL.md`). |

**sysrq:** `echo 0 > /proc/sys/kernel/sysrq` exists in **live installer-session only**,
not in `eliteew-ts-kiosk`. Disabling sysrq on installed boot is **out of scope** for this
TZ unless a separate hardening wave tasks it.

### 3.5 Code & license lock

| Item | Lock |
|------|------|
| Crate location | `trusted-substrate/state-unlock-agent/` (sibling of `installer/`, **not** live-installer-only tree) |
| License | **MIT** first-party; deps **MIT** preferred (e.g. `crossterm` / `ratatui`) |
| Fonts | System **DejaVu** only — same fail-closed as installer (no embedded non-MIT fonts) |
| Build | `build-state-unlock-agent-from-source.sh` mirroring installer zig/glibc ceiling |
| Overlay | `/usr/local/bin/empiieros-state-unlock-agent` + units under `/etc/systemd/system/` |
| Binary provenance | New manifest (e.g. `opt/eliteew-ts-state-unlock-agent/build-manifest.json`) + extend `scripts/buildroot-build.sh` and `post-build.sh` post-image verifier — **today** only `opt/eliteew-ts-installer/build-manifest.json` and `opt/empiieros/build-manifest.json` are hashed |
| Ship path | **`/usr/local/bin/empiieros-state-unlock-agent`** on **runtime rootfs overlay** — **not** under `eliteew-ts-installer` share (apply strips installer from installed target) |

**Forbidden:** prebuilt `.deb`/AppImage agents, Clevis, Plymouth, vendored
binary agents, GPL UI stacks for this wave.

### 3.6 Keyboard / VT lock

- **Preserve:** `XKB_DEFAULT_LAYOUT=us,ru`, `grp:alt_shift_toggle` (install + seat canon).
- **Do not** re-enable getty on tty2–6 from the agent; unlock UX stays on tty1.
- Installer GUI blocks Ctrl+Alt+F* VT chords; installed kiosk does not expose a console login VT.

---

## 4. Invariants (must not regress)

1. **Recovery ≠ PIN** — agent copy and validation must not suggest 4-digit PIN.
2. **No keyfile** in image or ESP for state LUKS.
3. Wrong or empty recovery → **no mount** of state → Runtime fail-closed.
4. TPM enroll path from apply **unchanged** (`systemd-cryptenroll --unlock-key-file=-
   --tpm2-device=auto` when `/dev/tpmrm0` or `/dev/tpm0` and **`systemd-cryptenroll`
   exists on the live installer**); when TPM opens the volume, agent **must not prompt**.
   Missing TPM, missing binary, or enroll failure logs `state tpm=no` and falls back to
   recovery — not an install abort. Do **not** add new packages solely for enroll in this TZ.
5. **`empiieros-runtime` must not start** without the real LUKS mount: keep apply-planted
   `RequiresMountsFor=/var/lib/empiieros-runtime`. Note: unit also declares
   `StateDirectory=empiieros-runtime` — do not add drop-ins that create a hollow
   `{state}` on plain root if mount failed.
6. Apply `log` remains **secret-free** for recovery (existing needles).
7. Live installer USB remains **unencrypted** install media (unchanged).
8. Agent answers **only** allowlisted cryptsetup requests for the state mapper.
9. **`BR2_PACKAGE_CRYPTSETUP=y`** stays required (`apply` dies if target lacks
   `/usr/sbin/cryptsetup`); this TZ does not add Clevis or extra crypto packages.
10. **No recovery leak on installed root:** apply `verify_target_install_contract` rejects
    `run/eliteew-ts-installer/state-recovery` or `var/lib/empiieros-runtime/state-recovery`
    on the target tree — agent must not recreate staging paths.
11. **Apply script mirror:** changes to install-time LUKS/crypttab logic stay in sync in
    **both** `payload-templates/usr-local-bin/eliteew-ts-install-apply` and
    `rootfs_overlay/usr/local/bin/eliteew-ts-install-apply` (kinship convention).

---

## 5. Implementation phases

### Phase 0 — Design + needles (this doc)

- [x] TZ written
- [x] Code sync against kinship apply / overlay / `selftest-installer.sh` (LUKS, crypttab, getty)
- [ ] Capture real `ask.*` `Id=` / `Message=` from no-TPM installed boot (QEMU or lab)
- [x] Stock console `.path`/`.service` ordering captured from host systemd (2026-09-22)
- [x] `selftest-installer.sh` needles: agent binary, `.path`/`.service`, console/wall mask policy
- [x] Optional `preflight-foundation.sh` overlay checks for agent binary + units (EmpiierOS)
- [x] Capture procedure: [`STATE-LUKS-ASK-CAPTURE.md`](STATE-LUKS-ASK-CAPTURE.md) (real `ask.*` still pending)

**Already enforced elsewhere (do not duplicate blindly):** LUKS2 + Argon2id format,
crypttab line, PIN-not-LUKS, recovery staging stdin, `selftest-state-luks.sh`,
getty@tty2–6 mask + tty1-only console policy, `tee -a "$LOG" >&2` log hygiene.

### Phase 1 — MVP agent (security over polish)

**Deliverables**

1. MIT Rust agent: parse `ask.*`, allowlisted `Id=`, TTY prompt, masked read, reply `+`. **Landed:** `trusted-substrate/state-unlock-agent/`.
2. `.path` + `.service` installed from kinship overlay. **Landed:** `empiieros-state-unlock-agent.{path,service}` + `cryptsetup.target.wants`.
3. Mask or drop-in so stock console agent does not double-prompt. **Landed:** mask `systemd-ask-password-console.path` + `systemd-ask-password-wall.path`.
4. `cargo test` + unit tests: ini parse, allowlist, no-log guard (grep/lint rule). **Landed:** `selftest-installer.sh` + `build-state-unlock-agent-from-source.sh` manifest gate.

**Measure**

```bash
# Dev test (after agent installed): systemd-ask-password --no-tty … triggers agent queue
# Use Id=/Message= from captured ask.* — not guessed strings.

scripts/selftest-state-luks.sh   # existing LUKS recipe (host loop)
bash trusted-substrate/scripts/selftest-installer.sh  # existing + new agent needles
# Installed path (documented in qemu-installed-system-smoke.sh):
# no-TPM boot → recovery on console → then Runtime/kiosk — extend smoke when agent lands
```

Adversarial:

- Wrong key → still locked (`cryptsetup luksOpen` fails); agent must not loop-pass empty.
- Non-cryptsetup ask file → agent **ignores** (no reply).
- `rg` agent sources for `println!.*pass` / logging passphrase → **FAIL** CI needle.

### Phase 2 — Branded console + operator hardening

- ANSI banner, localized strings (RU primary), installer-aligned colors.
- Cancel path (`-` reply) explicit; no partial passphrase leak on cancel.
- `mlock`/`zeroize` best-effort on passphrase buffer (document limits on swap).
- QEMU smoke: install → reboot → unlock → `empiieros-runtime` active → mount state.

### Phase 3 — Metal + docs same-phase

- Update [`session/OS-INSTALL.md`](../../session/OS-INSTALL.md) boot bullet (English).
- Diary entry in [`SECURE-RESET-LIVE-DIARY.md`](SECURE-RESET-LIVE-DIARY.md) with evidence paths only.
- **Do not** claim MATCH until fresh metal install + wrong-key FAIL + TPM path walk on representative hardware.

---

## 6. Measure checklist (acceptance for this TZ only)

| # | Check | Pass criterion |
|---|--------|----------------|
| M1 | Source | MIT LICENSE in crate; `selftest-installer` license needles |
| M2 | Build | Overlay binary rebuilt; manifest hash matches build script |
| M3 | Allowlist | Agent ignores non-`cryptsetup:` ask IDs |
| M4 | Secret hygiene | No passphrase in journal; apply log pattern unchanged |
| M5 | Fail-closed | Wrong recovery → no `/var/lib/empiieros-runtime` mount |
| M6 | TPM | When install logged `state tpm=yes` and TPM unlock succeeds, no manual prompt; when `state tpm=no`, branded prompt appears |
| M7 | Console policy | tty2–6 masked on overlay and after apply; eliteew-ts shadow stays locked; agent does not enable extra gettys |
| M8 | Kiosk after | After unlock, tty1 kiosk starts; PIN flow unchanged |
| M9 | Adversarial | Double-agent test: only one prompt visible |

**Not sufficient for MATCH / metal-ready:** QEMU-only, mux USB, or agent green without fresh install image.

---

## 7. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Agent/ console/wall agent race | M9 test; mask console.path (+ wall.path if needed) |
| Wrong `Id=` matcher | Capture real ask file in Phase 0; fixture tests |
| Runtime hollow state dir | Keep RequiresMountsFor drop-in; never bypass |
| Early boot no DRM/font | DejaVu on tty via vconsole; no GPU dependency |
| Operator confuses recovery vs PIN | Copy lock + install Done unchanged |
| GPL dependency creep | Dependency allowlist in `Cargo.toml` review in selftest |
| Scope creep to GUI | Separate future TZ; this TZ closes console MVP |

---

## 8. Out of scope

- Graphical unlock (labwc/egui at cryptsetup stage)
- Clevis, Tang, NBDE network
- Encrypting root (p2) or initramfs mandate
- Secure Boot / measured boot ceremony
- Changing LUKS parameters or PIN hash domain
- Storing recovery on disk or QR in agent

---

## 9. Estimated effort

| Phase | Effort | Notes |
|-------|--------|-------|
| Phase 1 MVP | ~3–7 engineer-days | Protocol + units + tests |
| Phase 2 polish | ~2–4 days | ANSI/branding + hardening |
| Phase 3 metal | operator + 1–2 days evidence | Not MATCH by default |

---

## 10. References

- [`TZ-DISK-AT-REST.md`](TZ-DISK-AT-REST.md) — volume layout, recovery semantics
- [`../../session/OS-INSTALL.md`](../../session/OS-INSTALL.md) — tty1-only console, apply LUKS lock
- [`../../session/FIRST-BOOT-PIN.md`](../../session/FIRST-BOOT-PIN.md) — PIN after unlock (not LUKS)
- [systemd Password Agents](https://systemd.io/PASSWORD_AGENTS/)
- Kinship: `format_state_luks` (`--type luks2`, `--pbkdf argon2id`) / `open_state_luks` /
  `write_target_state_unlock` / `enroll_state_tpm` / `verify_target_install_contract` /
  `plant_target_console_policy` in `eliteew-ts-install-apply`
- Kinship docs: `trusted-substrate/docs/ROOTFS-FROM-SOURCE.md` (console table),
  `trusted-substrate/installer/README.md` (Plymouth not enabled)
- Existing measure: `trusted-substrate/scripts/selftest-state-luks.sh`,
  `trusted-substrate/scripts/qemu-installed-system-smoke.sh` (LUKS p3; Runtime boot after unlock deferred today)
