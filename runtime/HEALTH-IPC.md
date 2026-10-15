# Runtime local IPC contract

**Protocol version:** 1
**Transport:** Unix stream socket, newline-delimited JSON
**Backend path:** `/run/empiieros/runtime.sock` (`0600 root:root`)
**Shell frontend path:** `/run/empiieros-shell/runtime.sock` (`0600 eliteew-ts`)

## Request

```json
{"id":"request-1","op":"runtime.health","params":{}}
```

- One request per connection.
- The request must end with `\n`.
- Maximum request body: 64 KiB.
- `id` and `op` use bounded ASCII identifiers.
- Unknown fields and non-object `params` fail validation.

## Response

```json
{
  "protocol_version": 1,
  "id": "request-1",
  "ok": true,
  "status": "ok",
  "data": {
    "version": "0.1.0",
    "transport": "unix-peercred-v1",
    "uptime_seconds": 1
  }
}
```

Failures set `ok:false`, return `data:null`, and include structured
`error.code`/`error.message`. Maximum response size is 256 KiB.

## Authorization

Runtime reads `SO_PEERCRED` from the backend socket and verifies:

1. the peer UID equals the configured bridge UID (`0` in production);
2. the peer PID is valid;
3. `/proc/<pid>/exe` equals the pinned `systemd-socket-proxyd` executable.

The native Shell connects only to the frontend socket owned by the kiosk
identity (`empiieros-shell-bridge.socket` on `sockets.target`, without
`After=empiieros-runtime.service` — that ordering cycles through
`basic.target` and systemd skips the ingress). A root-owned
`empiieros-shell-bridge.service` is the sole accepted backend peer, so
Runtime does not need `CAP_SYS_PTRACE`. The backend socket
parent must be owned by the Runtime effective UID and must not be group/world
writable. A second daemon cannot unlink an active listener.

`dev-peer-bypass` is test-only, mutually exclusive with `production`, and still
requires `EMPIIEROS_DEV_ALLOW_UNVERIFIED_PEER=1`.

## Status meanings

`system.snapshot` includes a monotonic `sequence`, collector provenance, and
typed domain records. `system.events` returns the bounded in-memory event log
after a caller-supplied `after_sequence`.

- `ok` / `observed` / `applied`: successful operation with observed data.
- `denied`: peer authorization failed.
- `invalid_request` / `request_too_large`: protocol failure.
- `not_available`: intentionally disabled by the secure-reset production
  profile.
- `not_found`: no registered operation.
- `failed`: substrate command, persistence, or verification failed.
- `audit_failed`: mutation was not attempted, or an applied result could not be
  durably audited.

HTTP routes and `X-EmpiierOS-Caller` are retired and have no authority.

## Production operations

| Operation | Class | Notes |
|-----------|-------|-------|
| `runtime.health` | observe | Process liveness and persisted `start_count` |
| `system.snapshot` | observe | Sequence, provenance, typed domains. Network interfaces include sanitized IPv4 and IPv6 CIDRs; connected still requires routed IPv4. DNS lists sanitized nameservers and `dns_source` from the resolv.conf symlink. Services include `systemd-resolved.service` and bounded `NRestarts`. Capabilities include family id `seat.browser` (`available`, overlay host) — not a MATCH claim. A launcher, product, or OTA capability is `available` only when that operation's first precondition is clear; the reason is that precondition's code. |
| `diagnostics.snapshot` | observe | Alias of `system.snapshot` |
| `system.events` | observe | Bounded in-memory log after `after_sequence` |
| `tray.status` | observe | Tray availability, quiet mode, badge/unread/critical counts, allowlisted `menu_items` under `{state}/kiosk_tray/`. |
| `tray.action` | mutation | Allowlisted tray action (`focus_shell`, `toggle_quiet_mode`, `request_quick_panel`, `clear_notifications`, `request_restart`); optional `param`; audits to `tray_audit.json`. |
| `notification.post` | mutation | Bounded tray queue under `{state}/kiosk_tray/` (`title`, `message`, `severity`, `source`, `action_hint`). Kiosk quiet mode suppresses info/warning, not critical. |
| `notification.list` | observe | Queued notifications + `unread_count` / `critical_count`. |
| `notification.dismiss` | mutation | Mark read or remove one id; optional `clear_all`. |
| `network.configure` | mutation | systemd-networkd IPv4 with rollback; verification reads `networkctl --json` `Address` as a string or a 4-octet array and requires the applied `.network` file to become `NetworkFile`. On this image, a new `05-*.network` is ignored and unmanages Wi-Fi; Runtime therefore rewrites the already-bound `20-wireless.network` in place (saved for rollback) and reloads, without restarting `systemd-networkd`. |
| `network.wifi.scan` | observe | Real `iw` scan |
| `network.wifi.connect` | mutation | Supervised `wpa_supplicant` (`scan_ssid=1`, `reset-failed`) + observed association, then `kick_dhcp`. Errors split: association not observed vs associated but no IPv4. A late lease must not roll back a proven PSK. Official SoT wifi unit still needs the apply `AF_INET` plant. |
| `network.wifi.disconnect` | mutation | Supervised stop + observed carrier loss |
| `network.wifi.reconnect` | mutation | Restart the persisted `wpa_supplicant-<iface>.conf` without reading or returning the PSK. Params are `{ "interface"?: name }`. Missing or insecure persist fails closed. |
| `network.wifi.radio` | mutation | Allowlisted WLAN rfkill through `empiieros-wifi-radio.service`. Params are `{ "action": "block"|"unblock", "type"?: "wlan", "name"?: rfkill name }`. Only `wlan` is accepted. Runtime does not write `/sys` itself (`ProtectKernelTunables`); the helper unit does. |
| `system.power` | mutation | Confirm-gated start of `empiieros-power-reboot.service` or `empiieros-power-poweroff.service`; those units isolate `reboot.target` / `poweroff.target`. Wrong `confirm` never starts a unit. |
| `display.control` | mutation | Allowlisted brightness write through `empiieros-display-brightness.service`. Params are `{ "action": "set_brightness", "percent": 10-100, "name"?: backlight }`. Percent below 10 is rejected so the glass cannot go dark. Runtime does not write `/sys` itself (`ProtectKernelTunables`); the helper unit does. Other `display.*` ops stay `not_available`. |
| `audio.control` | mutation | Allowlisted ALSA playback volume through `empiieros-audio-volume.service`. Params are `{ "action": "set_volume", "percent": 10-100, "name"?: allowlisted mixer }`. Percent below 10 is rejected. Other `audio.*` ops (`audio.action` mic select/mute included) stay `not_available`. Snapshot audio reports the current allowlisted playback percent via `empiieros-audio-volume --observe` (no mixer write). On Dell ALC3204 SSID `1028:09c1` the helper also enables the analog speaker-amp GPIO through in-tree HDA hwdep verbs (no foreign `hda-verb` binary). Other codecs are left untouched. |
| `time.sync` | mutation | Params are `{ "action": "set", "datetime": "YYYY-MM-DD HH:MM" }` (optional seconds). The helper `empiieros-time-sync` runs `date -s` in the seat timezone and `hwclock --systohc --utc`. Runtime stays `ProtectClock`. `action=sync` returns `ntp_masked`. timesyncd stays masked. No public NTP. Snapshot capability `time.sync` is `available` only when that helper binary, its unit, and `/usr/bin/systemctl` are installed; source is `operator-wall-clock`. A present timesyncd binary does not make the clock available. Other `time.*` ops stay `not_available`. |
| `seat.browser.status` | observe | Seat Browser status (`surface=shell_overlay`, `host=webkit_gtk`); policy mode/freshness plus bounded `allow`/`deny` host arrays; `tab_count`/`tab_cap`/`active_tab`/`tabs[{id,host,url}]`; `camera`/`microphone` (default true). |
| `seat.browser.policy` | observe | Effective mode + bounded `allow`/`deny` host arrays (host suffixes, not secrets) + counts. |
| `seat.browser.policy.set` | mutation + confirm | Seat-local policy; `ENABLE_BROWSER_POLICY`. Local file SoT restamps TTL on each Runtime open until API fetch exists. |
| `seat.browser.evaluate` | observe | URL allow/deny decision without host navigate. |
| `seat.browser.navigate` / `open` | mutation | policy gate; Allow → `host_action=load` + `tab_id` (creates tab 1 if empty). |
| `seat.browser.tab.open` | mutation | `{url?}` empty → `about:blank`. Cap 3. Fourth is `denied`/`tab_limit`. |
| `seat.browser.tab.select` | mutation | `{id}` → `host_action=tab_select`. |
| `seat.browser.tab.close` | mutation | `{id}`. Last tab → `host_action=hide`. Overlay hide does not purge tabs. |
| `seat.browser.close` | mutation | `host_action=hide` (tabs persist). |
| `seat.browser.reload` | mutation | `host_action=reload`. Native overlay / Shell reloads the Seat Browser WebKit. |
| `seat.browser.history` | mutation | `{direction:back\|forward}` → `host_action`. Overlay chrome + `decide_policy` still evaluate. |
| `seat.browser.media.set` | mutation | `{camera?,microphone?}`. Enabling either needs `ENABLE_BROWSER_MEDIA`. Overlay consults `status` before UserMedia. Display capture denied. |
| `seat.browser.download.begin` | mutation | `{url,name?}`. Same HTTPS allow/deny as navigate. One in flight. Dest `file://…/seat_browser/downloads/<basename>` only. Overlay must check dest before `set_destination`. |
| `seat.browser.download.finish` | mutation | `{id,ok,bytes}`. Cap 32 MiB. Status lists basename/host/bytes/phase, never a path. |
| `seat.browser.reset` | mutation + confirm | Purge `{state}/seat_browser/` including `tabs.json` and downloads (`RESET_SEAT_BROWSER`). |
| `seat.shield.status` | observe | Redacted mode (`observe_only` \| `armed` \| `unavailable`), `act_enabled`, `auto_act_protected`, `auto_quarantine_hash` (default true), `heuristics_enabled` (default true), `open_chains`, `open_chain_teasers` (≤5), `quarantine_count`, `watch_pending`, `last_watch_drained`, `cycle_interval_secs`, twin `security.integrity` + `integrity_twin_state=disabled`, `blocked_peers` (cap 32), last `connections[]` sorted by `risk_milli` when heuristics on (cap 128). Drains `watch.jsonl` (optional helper `sha256` used when Runtime cannot open the path) then calls `maybe_auto_observe` when the disk interval elapsed (HUD tick). Sparse legacy `policy.json` (pre-S4 keys) migrates once to armed defaults; explicit disarm is not overwritten. |
| `seat.shield.events` | observe | Bounded redacted event page (`after_sequence`, `limit`) from `events.json` ring. |
| `seat.shield.journal` | observe | Browse `journal.jsonl` (filters, analytics, optional `group_by`). Not decision SoT. |
| `seat.shield.policy` | observe | Protected ports, **`allowlist`**, allowlist count, `act_enabled`, `auto_act_protected`, `auto_quarantine_hash`, `heuristics_enabled`, `cycle_interval_secs`. |
| `seat.shield.policy.set` | mutation + confirm | Validated mutation; `ENABLE_ACTS` to arm. `auto_act_protected:true` only when acts are enabled or enabled in the same call. `auto_quarantine_hash:true` requires `act_enabled` + `ENABLE_QUARANTINE`. `heuristics_enabled` needs no floor confirm. |
| `seat.shield.cycle` | observe | Cycle over `connections[]`; persist events + journal dual-write; mutate floor only for armed protected-service `BlockPeer` (`applied_acts`). Optional auto-hold only after `ioc_hash_match` when `auto_quarantine_hash` is on (cap 1). Returns `connections[]`. |
| `seat.shield.observe` | observe | Prefer `seat-shield-probe` inet_diag dump (TTL); fail→`/proc/net/tcp{,6}` + bounded exe SHA-256 + same cycle path. Also drains `watch.jsonl`. Returns `connections[]` (optional `rtt_ms`). |
| `seat.shield.feed_status` | observe | Sealed IOC feed digest/version only. |
| `seat.shield.feed.install` | mutation | Absolute path under Runtime state root only. |
| `seat.shield.act` | mutation + confirm | Allowlisted acts; **Unavailable** without `/usr/libexec/empiieros-seat-shield-fw`; **Helper+nft** when binary present. Overlay packager ships fw+nft+unit; Substrate image bake still required for metal MATCH. |
| `seat.shield.reset` | mutation + confirm | Purge `{state}/seat_shield/` (`RESET_SEAT_SHIELD`). |
| `seat.shield.scan_path` | observe | On-demand hash of one regular file under `/tmp/` `/var/tmp/` `/dev/shm/` (32 MiB, no symlink escape). `/etc/passwd` denied. |
| `seat.shield.quarantine` | mutation + confirm | Hold one allowlisted file (`QUARANTINE_FILE`) into `{state}/seat_shield/quarantine/<sha256>`. |
| `seat.shield.quarantine.list` | observe | Redacted held items: sha256, size, epoch, reason, orig **basename** only. |
| `seat.shield.quarantine.restore` | mutation + confirm | Restore one sha256 (`RESTORE_QUARANTINE`) if dest is still legal. |
| `launcher.status` | observe | EmpiierOS launcher vehicle, harvested `fp_v2` fingerprint, PIN/capsule flags (`pin_issued` / `pin_unlocked` / `pin_locked` / `fingerprint_bound`), redacted `product_id` and `capsule_source` (`air`/`file`) from `bound.json`, plus inner flags (`entitlement_present` / `channel_present` / `allow_*`), channel validate (`channel_valid` / `channel_up` / `channel_code`), and SPA pack flags (`spa_state` / `spa_code` / `spa_app_build_id`). Portal conf must be split-tunnel `10.88.89.1/32`. Never returns the PIN, entitlement JSON, WG conf, or pack bytes. Portal/SPA helpers are OS-plane units; Runtime keeps `IPAddressDeny=any`. Not Tauri. `product.open` is `allowed` only when unlock + `allow_fofline` + pack ready. |
| `launcher.capsule_import` | mutation | Capsule v3 file import after first-boot unlock. Params `{ "path" }`. Before unlock: `pin_not_issued` / `pin_not_unlocked`. Allowlisted media/state paths only. Recipient SK from `{state}/device_keys/recipient.sk.hex` (installer plant; Runtime creates the file if absent, fails closed on an invalid existing file). Issuer PK: env / bake / state file / living prod verifying PK. Inner entitlement/channel persist under `{state}/capsule_inner/` (`0600`); sealed blob at `{state}/capsules/current.ecap`. Verify-only OTA PK plants from entitlement `payload.ota_artifact_pk` (Air admin-code is the priority path; USB sibling PK is fallback) (`ota_artifact_pk=planted|missing`). IPC is flags only — never the PK hex. |
| `launcher.air_activate` | mutation | Air access-code activation (EWAE1) after first-boot unlock. Params `{ "code" }`. Before unlock: `pin_not_issued` / `pin_not_unlocked`. Local wall clock year must be 2024–2100 (`wall_clock_required`) — junk RTC breaks air TLS the same way it breaks Keycloak DPoP. Parses Crockford code, redacts selector, never returns the secret, payload, or activation token. Recipient SK must exist before WSS (`device_request_v1`); install plants it, Runtime ensures if absent, `device_keys_invalid` if the file is garbage. Living delivery is baked `wss://app.eliteew.ru:{port}/api/v1/desktop-client/air-enroll/ws` through allowlisted `empiieros-air-enroll.service`. Helper persists inner **before** `activation_ack`. Runtime refuses helper `ok` without `{state}/capsule_inner/` (`air_helper_inner_missing`). FoFline entitlement `payload.ota_artifact_pk` plants `{state}/ota/artifact.pk.hex` (`ota_artifact_pk=planted|missing`; never the hex). Runtime keeps `IPAddressDeny=any`. |
| `launcher.pin_issue` | fail-closed | Production IPC does not mint a PIN (`not_available` / `installer_issued_only`). The installer plants `{state}/seat/first_boot_pin.json` (`0600`, hash only). `issue()` remains a test/installer-equivalent helper. |
| `launcher.pin_unlock` | mutation | Unlock with `{ "pin" }` (exactly four digits). Fail-closed on format, fingerprint mismatch, and five-failure lockout. One compatibility rebind: planted `no-machine-id` fp on the same host after systemd commits `/etc/machine-id`. `product.open` still needs `allow_fofline` + pack `ready`. |
| `launcher.channel_up` | mutation | OS-plane portal bring-up after unlock + `channel_valid`. Starts `empiieros-portal-up.service` (`ip` + `empiieros-wg`, no `wg-quick`). After the iface is up, applies the portal host-map and refuses `channel_up` unless `portal_hostmap=applied` (repairs once if an old helper skipped the map). Then proves a non-portal global IPv4 still exists (`portal_broke_uplink` — not a baked `eno1`). Runtime keeps `IPAddressDeny=any`. Conf stays `{state}/wireguard/wg-portal.conf` — not `/etc/wireguard`. |
| `launcher.hostmap_apply` | mutation | After unlock + portal up: write `app.eliteew.ru` / `auth.eliteew.ru` → `10.88.89.1` in `/etc/hosts`. Helper-only write. Public TCP 443 stays denied. |
| `launcher.spa_sync` | mutation | OS-plane **bootstrap** FoFline SPA pack after unlock + `allow_fofline` + `channel_up` (journey stage `spa`; OIDC not required). Helper `empiieros-spa-fetch.service` may talk only to VIP `10.88.89.1`. Verify minisign + sha256, then stage `{state}/fofline-spa/current`. After core `ready`, the same helper fetches `spa-assets-latest.json` and merges signed optional groups into `current/` (hash-skip; does not wipe core). `audio-guides` is deferred. User media stays `/api/v1/content`. Runtime `IPAddressDeny` stays. **Not** capsule-gated OTA — see `launcher.ota_*` / [`../session/OTA-CAPSULE.md`](../session/OTA-CAPSULE.md). |
| `launcher.ota_status` | observe | Capsule-gated OTA gates + `ota_apply` (`idle`/`ready`/`applied`) + `ota_applied_build_id` / `ota_staged_build_id` / `ota_next_action` (`fix_gates`/`check`/`install`/`current`). After a prior apply, a newer staged build returns `ready` again (not stuck `applied`). Plants verify PK from entitlement when missing. No secrets on IPC. |
| `launcher.ota_request` | mutation | Attest then VIP update-capsule path. Hard gates required (incl. planted artifact PK). Helper POST `/empiieros-ota/v1/attest` → grant → GET current+blobs → stage. Empty edge → `no_capsule`. Without helper binary → `ota_helper_required`. Missing PK → `artifact_pk_missing`. |
| `launcher.ota_apply` | mutation | Typed confirm `APPLY_OTA`. E4 cutover for `os_component` (P4 dest table + `ota_baseline.json`). Restart only changed binaries; Runtime/GUI restart is deferred past the IPC newline (`invalid runtime response framing` otherwise). `shell/` sets `ota_reload_shell`. FoFline → `os_before_payload`. Not A/B P93. |
| `launcher.edge_health` | mutation | After unlock + `allow_fofline` + portal up: helper GET `/api/health` to VIP `10.88.89.1` with SNI `app.eliteew.ru`. Runtime keeps `IPAddressDeny=any`. Body prefix only (no secret fields). |
| `launcher.oidc_start` | mutation | After unlock + `allow_fofline` + portal up + `portal_hostmap=applied` + sane wall clock: helper POSTs RFC 9126 PAR to VIP `10.88.89.1` (SNI `auth.eliteew.ru`). Missing host-map is `portal_hostmap_required` — without it the auth card stays white while PAR can still succeed by IP. Junk RTC year is `wall_clock_required`. DPoP `iat` is the issuer HTTP `Date` (IdP clock). Do not clamp to local+15 — installer clock without NTP can lag and produce `oidc_http_400`. Missing Date is `oidc_dpop_date_required`, not a local fallback. A 400 body may add an allowlisted `error` token (`oidc_http_400:invalid_dpop_proof`). IPC returns `client_id`, `request_uri`, and the baked `authorization_url` (Shell allowlists that URL before `shell.auth_surface`). Ticket/PKCE stay out of the browser URL. Verifier stays `{state}/oidc/tx.json` `0600`. |
| `launcher.oidc_complete` | mutation | Native auth surface captures `https://app.eliteew.ru/fofline-oidc-bridge.html?code=&state=`. Helper POSTs the token endpoint on VIP `10.88.89.1` with SNI `auth.eliteew.ru`. IPC returns `oidc_session=present` only — no tokens. |
| `launcher.oidc_inject` | native-only | Access/id token for `window.EmperyDesktopNativeOidc`. Shell JS `launcher.oidc_inject` is `token_ipc_forbidden`. Product `fofline-spa:` talks only through `empiierosOidc` (`refresh` / `logout`) — living `product_session_refresh` harvest. `params.force=true` always runs helper `grant_type=refresh_token` + DPoP; if that fails but access is still unexpired, inject returns the current pair. Open/inject without force still refresh only when `expires_at` is within 30s. Refresh token never leaves Runtime. Official bake also plants access in a `refreshToken` *field* and answers `/api/v1/auth/sso/refresh` via the same host refresh (old pack). Product `/api/v1` stays same-origin; GUI proxy plants `X-SSO-Replace-Confirm: 1`. Runtime `IPAddressDeny` stays. |
| `launcher.oidc_logout` | mutation | After unlock: overwrite-and-unlink `{state}/oidc/session.json` plus in-flight `tx.json` / `callback.json`. IPC returns `oidc_session=missing` only. DPoP key stays. No Keycloak end_session from Runtime (`IPAddressDeny=any`). Shell hides product/auth surfaces. |
| `shell.seat_browser_surface` | bridge (native, not `core.rs`) | `show`/`hide`/`load`/`tab_open`/`tab_select`/`tab_close` plus `reload`/`back`/`forward`/`stop`/`find`/`find_next`/`find_prev`/`zoom`/`go`. Overlay is ≤3 related WebViews in a GtkStack. `show`/`go` re-evaluate via `seat.browser.evaluate`; `go` also `seat.browser.navigate`. Find/zoom have no Runtime op. Ctrl+T/W/Tab go through Runtime. Escape/F9 hides. |
| `product.open` | mutation | Typed local pack after unlock + `allow_fofline` + pack `ready` + native OIDC session. If access is expired and a refresh token remains, the helper refreshes it (no tokens on IPC). Default target `fofline_local`. Runtime builds `fofline-spa://localhost/index.html?eliteewFoflineClient=1`. `connected_cloud` is `remote_spa_forbidden`. Without a refreshable session: `oidc_session_required`. Native Shell keeps an always-on top HUD (not FoFline-only); F9/Escape still return. The HUD seat panel observes `seat.shield.status`/`events`/`observe` and does not add a new IPC op. OS A/B image OTA is not this op. Bootstrap pack is `launcher.spa_sync`; capsule-gated updates are `launcher.ota_*`. |

`system.power` params are `{ "action": "reboot"|"poweroff", "confirm": "REBOOT"|"POWEROFF" }`.
The confirmation token must match the named action. Runtime does not hold
`CAP_SYS_BOOT`; isolate is performed by the allowlisted systemd units.
