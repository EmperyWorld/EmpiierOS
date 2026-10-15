# EmpiierOS — FoFline seat canons (metal-proven)

**Status:** Living canons. Updated 2026-09-21.
**Verdict:** `NOT_READY`. Not MATCH. Not official `out/`.
**Authority companions:** [`OS-INSTALL.md`](OS-INSTALL.md),
[`PORTAL-HOSTMAP.md`](PORTAL-HOSTMAP.md), [`PORTAL-TUNNEL.md`](PORTAL-TUNNEL.md),
[`AIR-HANDSHAKE.md`](AIR-HANDSHAKE.md), [`SHELL-GUI.md`](SHELL-GUI.md),
[`SEAT-LAUNCHER.md`](SEAT-LAUNCHER.md),
[`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).
**Lab chronology:** [`../docs/plans/SECURE-RESET-LIVE-DIARY.md`](../docs/plans/SECURE-RESET-LIVE-DIARY.md).

These are fail-closed rules learned on a real seat. Prose in the diary is
not acceptance. A USB plant is not official `out/` and not metal-ready.

## Journey (locked)

```text
Unlock → Activate (air | file) → Portal → Workspace → Sign-in → Open
```

Runtime owns PIN, consume, portal, pack, and OIDC. Shell only shows the
current step. No `autoOpened`. Tokens never enter Shell JS.

## Canons

### 1. Operator wall clock is seat SoT

- `systemd-timesyncd` stays **masked**. Empty NTP/FallbackNTP. No public
  or Google NTP. nft has no UDP/123 hole.
- Installer regional time is minute precision and writes
  `/etc/empiieros/install-wall-clock`. First boot may apply that stamp
  only when the RTC year is implausible (outside 2024–2100). It must not
  rewind a sane RTC.
- The running seat sets time with `time.sync` `{ "action": "set",
  "datetime": "YYYY-MM-DD HH:MM[:SS]" }`. `action=sync` is `ntp_masked`.
- Air (`launcher.air_activate`) and OIDC (`launcher.oidc_*`) fail closed
  with `wall_clock_required` when the local civil year is outside
  2024–2100. Junk RTC is not “network down.”
- Masked timesyncd must not be narrated as “operator clock ignored.”

### 2. Portal VIP ≠ auth WebView DNS

- Helper HTTPS (PAR, token, spa-fetch, air) connects to VIP
  `10.88.89.1:443` with SNI (`auth.eliteew.ru` / `app.eliteew.ru`).
  That path does **not** need `/etc/hosts`.
- The Keycloak **card** (`shell.auth_surface`) loads
  `https://auth.eliteew.ru/…` in WebKit. Without
  `auth.eliteew.ru → 10.88.89.1` in `/etc/hosts` the card stays white
  while PAR may already have succeeded. Keycloak LOGIN can look silent
  if only LOGIN is filtered; PAR is `PUSHED_AUTHORIZATION_REQUEST`.
- `launcher.channel_up` must leave `portal_hostmap=applied` (repair once
  if an older helper skipped the write). Missing map is
  `portal_hostmap_required`. Do not ask the operator for a second
  “repair host-map” click as the happy path.
- Split-tunnel `AllowedIPs` is exactly `10.88.89.1/32`. Default route
  stays on the uplink.

### 3. Native OIDC — PAR in helper, tokens out of JS

- Sign-in is RFC 9126 PAR from the helper, then
  `authorization_url` with `client_id` + `request_uri` only.
- Detail string `native Keycloak PAR login; tokens stay out of JS` means
  PAR applied — not a bug.
- Binding ticket / PKCE verifier stay in Runtime / helper POST. Never on
  the browser URL or IPC.
- DPoP `iat` follows IdP HTTP `Date`. Do not clamp to local+15 (installer
  minute clock lags Keycloak’s active window). Missing Date is
  `oidc_dpop_date_required`. A 400 may surface
  `oidc_http_400:invalid_dpop_proof` (allowlisted token only).
- Auth WebKit sandbox stays off so CA store and `/etc/hosts` are visible
  (same class as product HTTPS).

### 4. Shell package trust (root-owned assets)

- `empiieros-shell-gui` serves `empiier://shell/` only for **regular
  root-owned** files under `/opt/empiieros/shell` (mode without
  group/other write). Non-root `index.html` →
  `Shell asset is not trusted` → page
  «Ошибка загрузки EmpiierOS» / «Не удалось загрузить доверенный пакет
  Shell.»
- Apply `plant_seat_shell` must `chown -R 0:0` after `cp -a` from the USB
  share (USB files are often uid 1000). Contract dies with
  `seat shell is not root-owned` if that slips.
- The same root-owned rule applies to FoFline pack files under
  `fofline-spa://` (see [`PRODUCT-OPEN.md`](PRODUCT-OPEN.md)).

### 5. Console (tty1 only)

- tty1 is `agetty --autologin eliteew-ts` (no password prompt).
- Buildroot post-build **locks** `eliteew-ts` in `/etc/shadow` (`!`/`*`).
  Apply keeps that lock on the installed seat — no password login on extra VTs.
- **tty2–6** gettys are masked to `/dev/null`; apply `plant_target_console_policy`
  re-enforces on the target rootfs. Installer labwc blocks Ctrl+Alt+F* VT chords.
- Operator logs on metal: provisioned **key-only SSH** (`empiieros-maint` or
  operator policy), not a documented local console password.

### 6. PIN ≠ LUKS ≠ air SK

- First-boot PIN is a Runtime IPC gate (`fp_v2`).
- LUKS2 state volume recovery is not the PIN.
- Air recipient SK is `{state}/device_keys/recipient.sk.hex` on the opened
  LUKS volume. Production `device.keys.*` stays compiled out.
- OTA verify PK is `{state}/ota/artifact.pk.hex` planted on Air
  (`launcher.air_activate`, entitlement `payload.ota_artifact_pk`).
  Never an artifact SK. Not on installer media. Not a USB GRUB
  recovery helper.

### 7. USB plant ≠ factory SoT

- Official SoT is kinship `out/rootfs.tar.gz` (factory hash). A live USB
  mux or apply plant is **not** MATCH and **not** metal-ready.
- Flash only when `TRAN=usb` and `RM=1`. Do not wipe lab NVMe unless the
  operator named that disk.
- Reinstall is required to pick up planted Runtime / Shell / helpers on an
  already-running seat. Do not claim the seat has new binaries until
  apply planted them onto that rootfs.

## Operator glass checklist (when something fails)

| Symptom | First check |
|---------|-------------|
| «Не удалось загрузить доверенный пакет Shell» | `/opt/empiieros/shell` uid/mode (must be root `0644`) |
| White Keycloak card after «Войти» | `portal_hostmap` / `/etc/hosts` VIP line; syslog `auth load` |
| Green PAR detail, Keycloak “silent” | Event `PUSHED_AUTHORIZATION_REQUEST`, not only LOGIN |
| Air or OIDC fails with clock / DPoP | Seat clock year; `time.sync` set; no NTP |
| Need logs on metal | Key-only SSH when provisioned; no tty2 console |

## Pause note (2026-09-21)

Metal reinstalls are paused by operator choice. Source locks above are
in EmpiierOS / Trusted Substrate; the next USB plant carries them. The
already-installed seat stays on whatever binaries apply last wrote.
