# State LUKS ask-password capture (Phase 0)

**Status:** Procedure. **Not** MATCH.  
**Owner:** [`TZ-STATE-LUKS-UNLOCK-AGENT.md`](TZ-STATE-LUKS-UNLOCK-AGENT.md)

## Goal

Replace the placeholder fixture with a real `ask.*` file from **no-TPM**
installed boot so the agent allowlist matches production `Id=` / `Message=`.

## Placeholder (until capture)

Kinship: `trusted-substrate/state-unlock-agent/fixtures/ask-cryptsetup-sample.ini`

## Capture on metal or QEMU

1. Install seat (no TPM or TPM disabled); ensure `state tpm=no` in apply log.
2. Reboot installed root; stop at LUKS prompt (before kiosk).
3. On **another VT or serial** (if available), copy one file:

   ```bash
   cp /run/systemd/ask-password/ask.* /tmp/state-luks-ask.capture
   ```

4. Redact nothing except do **not** commit passphrases. The capture is the
   `.ini` ask file only (no secrets).
5. Store as `state-unlock-agent/fixtures/ask-captured-<date>.ini` and point
   unit tests at it.
6. Update allowlist in code only if the capture proves a narrower matcher is
   safe.

## Stock systemd ordering reference

Captured from host systemd (2026-09-22):

- `systemd-ask-password-console.path`: `DirectoryNotEmpty=/run/systemd/ask-password`,
  `Before=cryptsetup.target`, `DefaultDependencies=no`.
- `systemd-ask-password-console.service`: `systemd-tty-ask-password-agent --watch --console`.

Our units mirror `Before=cryptsetup.target getty@tty1.service` and mask stock
console + wall paths.

## After capture

- [x] Refresh fixture + `cargo test` allowlist fixtures (`ask-captured-20260921.ini`, kinship `state-unlock-agent/`)  
- [x] Extend `qemu-installed-system-smoke.sh` installed-boot phase (Phase 2)  
- [x] M5 adversarial in same script: wrong recovery + empty passphrase → no state mount, then happy unlock  
- [x] Diary entry in `SECURE-RESET-LIVE-DIARY.md` with capture path only  
