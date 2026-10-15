# EmpiierOS Runtime

**Plane:** First-party privileged operations bridge.
**Verdict:** `NOT_READY`. EmpiierOS source tests pass. Official-image QEMU
smokes are **not current** (prior H1 FAIL stands). Full Trusted Substrate
`selftest-secure-reset.sh` is red on a missing kinship readiness document.
Representative-metal verification remains.

## Active production entrypoints

`runtime/Cargo.toml` selects:

- library: `src/core.rs`
- daemon: `src/daemon.rs`

Historical Rust modules remain source archaeology but are not linked into the
production crate.

## Trust boundary

- Transport: newline-framed JSON. Shell uses `/run/empiieros-shell/runtime.sock`;
  Runtime listens on `/run/empiieros/runtime.sock`.
- Peer identity: Linux `SO_PEERCRED` against the root-owned
  `systemd-socket-proxyd` bridge, not the kiosk GUI process.
- Browser JavaScript never chooses a caller role or connects to Runtime
  directly; a confined `empiier://shell/` asset handler and
  `runtime/shell-gui/native_bridge.c` own the browser/transport boundary.
- Requests and responses are bounded.
- Socket and state parents must be real, owner-controlled directories.
- Production defaults to the `production` Cargo feature.
- The test-only peer bypass requires a separate `dev-peer-bypass` build and an
  explicit environment flag; it cannot be combined with `production`.

## Active operations

| Operation | Behavior |
|---|---|
| `runtime.health` | Process version, transport, monotonic uptime, and persisted start count |
| `system.snapshot` | Observed host/network/power/security snapshot |
| `diagnostics.snapshot` | Alias of the observed snapshot |
| `network.configure` | Atomic systemd-networkd IPv4 configuration with rollback and matching network-file/address-source verification |
| `network.wifi.scan` | Real `iw` scan |
| `network.wifi.connect` | Root-only wpa_supplicant configuration, supervised restart, observed association, then DHCP kick; late lease does not roll back a proven PSK |
| `network.wifi.disconnect` | Supervised stop and observed carrier loss |
| `network.wifi.reconnect` | Restart the already-persisted wpa_supplicant file; PSK never enters IPC |
| `network.wifi.radio` | Allowlisted WLAN soft-block/unblock through sysfs rfkill helper |
| `display.control` | Allowlisted backlight brightness through the sysfs helper unit |
| `audio.control` | Allowlisted ALSA playback volume through the mixer helper unit |
| `time.sync` | Operator wall clock through the allowlisted helper (`action=set`). `action=sync` is `ntp_masked`. Snapshot source is `operator-wall-clock` when the helper binary, unit, and `systemctl` are present; a masked timesyncd binary is not a gate |
| `system.events` | Bounded in-memory observed event log |
| `system.power` | Confirm-gated start of allowlisted reboot/poweroff isolate units |
| `launcher.status` | Observe vehicle, `fp_v2`, PIN/inner flags, portal channel (`valid_idle` / `valid_up` via sysfs); never PIN/WG |
| `launcher.capsule_import` | Capsule v3 file import after first-boot unlock; inner persist, flags only on IPC |
| `launcher.air_activate` | Living EWAE1 after first-boot unlock; recipient SK from install plant or Runtime ensure-if-absent; same inner persist; Runtime keeps `IPAddressDeny=any` |
| `launcher.channel_up` | OS-plane portal bring-up after unlock + `channel_valid`; helper `ip` + `empiieros-wg`; then host-map; Runtime keeps `IPAddressDeny=any` |
| `launcher.hostmap_apply` | `app`/`auth.eliteew.ru` → `10.88.89.1` in `/etc/hosts` after portal up |
| `launcher.spa_sync` | OS-plane SPA pack OTA after unlock + `allow_fofline` + portal up; helper VIP-only; minisign+sha256 |
| `launcher.edge_health` | Helper GET `/api/health` to VIP `10.88.89.1` after unlock + `allow_fofline` + portal up; body prefix only |
| `launcher.oidc_start` | PAR then `authorization_url` + `request_uri` after unlock + portal. PKCE/DPoP stay in Runtime. |
| `launcher.oidc_complete` | Native auth-card callback; helper token POST to VIP/`auth.eliteew.ru`. IPC is `oidc_session` only. |
| `launcher.oidc_inject` | Native-only access/id for `EmperyDesktopNativeOidc`. Shell JS is `token_ipc_forbidden`. |
| `launcher.oidc_logout` | After unlock: wipe session/tx/callback. IPC is `oidc_session=missing` only. |
| `product.open` | Typed local FoFline pack after unlock + pack `ready` + native OIDC session. Entrypoint is `fofline-spa://localhost`. Remote SPA denied. |
| `launcher.pin_issue` | Fail-closed (`installer_issued_only`); installer plants the PIN |
| `launcher.pin_unlock` | Unlock the installer-planted 4-digit first-boot PIN |
| `seat.browser.status` / `policy` / `evaluate` | Observe overlay host + redacted policy + URL decision + tab ledger |
| `seat.browser.navigate` / `open` / `reload` / `history` / `close` | Policy gate + `host_action` for the native overlay |
| `seat.browser.tab.open` / `tab.select` / `tab.close` | Cap 3; persist `{state}/seat_browser/tabs.json`; last close hides |
| `seat.browser.policy.set` / `reset` | Confirm-gated seat-local policy / purge (reset clears tabs) |
| `seat.browser.media.set` | Persist camera/microphone; enable needs `ENABLE_BROWSER_MEDIA` |
| `seat.browser.download.begin` / `finish` | Policy gate; dest only `{state}/seat_browser/downloads/`; 32 MiB / one in flight |
| `seat.shield.*` | S2 observe-only cycle/events/policy; `act` stays helper-gated |

Remote `product.open` targets stay fail-closed. Legacy simulated capabilities return
`not_available` or `not_found`. Contract detail: [`HEALTH-IPC.md`](HEALTH-IPC.md).

## State and commands

- State root: `/var/lib/empiieros-runtime`, mode `0700`.
- Audit: `audit.jsonl`, mode `0600`, bounded and synchronized.
- Mutations require a durable pre-audit entry before execution.
- External commands use absolute paths, a cleared environment, bounded output,
  and timeouts.

## Verification

```bash
cargo test --locked --manifest-path runtime/Cargo.toml
cargo check --locked --all-targets --manifest-path runtime/Cargo.toml
bash scripts/runtime-health-smoke.sh
```

The smoke compiles a separate development transport profile, proves peer
rejection, refuses active-socket replacement, and exercises framing and
connection limits. Image/QEMU acceptance is owned by Trusted Substrate;
representative-metal acceptance remains separate.
