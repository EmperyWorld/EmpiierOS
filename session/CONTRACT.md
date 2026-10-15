# EmpiierOS — session plane contract

**Status:** Session-plane index. **Not** metal-ready. **Not** `PASSED`.
**Living glass map:** [`SEAT-FACE.md`](SEAT-FACE.md).
**Production IPC:** Unix `SO_PEERCRED` NDJSON. Allowlist:
[`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md) and
`runtime/src/core.rs`. HTTP `GET/POST /v0/*` and `X-EmpiierOS-Caller` are
retired. Historical P62–P92 contract files are in `trash/` (2026-09-21).
The **Owns** table below is a historical P-wave archive: a `wired` row
there is **not** a production op.

**Plane:** `session/` — seat / session contracts (L-session).
**Floor kinship:** [`../docs/foundation/SUBSTRATE-KINSHIP.md`](../docs/foundation/SUBSTRATE-KINSHIP.md).
**Bootstrap:** [`bootstrap/BOOTSTRAP-PHASE.md`](bootstrap/BOOTSTRAP-PHASE.md).
**Production IPC detail:** `/run/empiieros-shell/runtime.sock` →
`systemd-socket-proxyd` → `/run/empiieros/runtime.sock`. Disabled prefixes
(`audio.*` except `audio.control`, `display.*` except `display.control`,
`input.*`, `time.action`, `settings.*`, `security.integrity.*`,
`operator.auth.*`, `update.*`, `enroll.*`, `power.action`, …) return
`not_available`. Confirm-gated power is `system.power` only. Brightness
mutation is `display.control` only. Volume mutation is `audio.control`
only (ALSA playback 10–100; no Pulse/PipeWire). Snapshot audio reports
allowlisted ALSA playback name and percent from
`empiieros-audio-volume --observe` without changing the mixer; on Dell
ALC3204 SSID `1028:09c1` that helper may enable speaker-amp GPIO (not a
mixer write). Snapshot Ethernet interfaces report sysfs speed/duplex when
valid. Snapshot interfaces report sanitized IPv6 CIDRs from `ip -j -6`
without treating link-local as routed. Snapshot DNS reports sanitized
nameservers plus `dns_source` from the `/etc/resolv.conf` symlink
(`systemd-resolved-uplink` / `systemd-resolved-stub` /
`static-resolv-conf`). Snapshot services include
`systemd-resolved.service`. Snapshot Wi-Fi associations report BSSID,
frequency, signal, and rx/tx Mbps from `iw`. Saved-network restore is
`network.wifi.reconnect` only and never carries a PSK. WLAN
soft-block/unblock is `network.wifi.radio` only and writes sysfs through
`empiieros-wifi-radio.service`. Snapshot storage lists observed block
devices from sysfs and root `statvfs` used/total/available bytes.
Snapshot USB lists sysfs device nodes (not interfaces) with
vendor/product/speed. Snapshot services report allowlisted systemd
ActiveState, Result, bounded NRestarts, and whether
ExecMainStartTimestamp is set. `empiieros-wifi-restore` is ready when
inactive/success/ran after an IPv4/SSID wait. Host snapshot reports CPU,
memory, swap from `/proc/meminfo`, and load 1/5/15 from `/proc/loadavg`.
Display snapshot reports the DRM driver and PCI id. Runtime
`start_count` increments on each process start. Clock control is
`time.sync` only: `{ "action": "set", "datetime" }` writes the operator
wall clock through the allowlisted helper; `action=sync` is
`ntp_masked`. The image **masks** `systemd-timesyncd` and ships an
empty NTP/FallbackNTP drop-in so compiled Google hosts are not used.
Installer wall clock is operator-set and is the seat SoT. Apply persists
`/etc/empiieros/install-wall-clock`; first boot applies it when the RTC
year is implausible. Air and OIDC refuse junk RTC years
(`wall_clock_required`). Snapshot may say timesyncd inactive — that is
not “the operator clock was ignored.” Metal-proven FoFline canons:
[`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md). Snapshot power
lists sysfs supplies (including bounded battery charge/voltage/current
and sanitized model text) and thermal zones. Snapshot display lists DRM
connector enabled/dpms/modes. Snapshot input reports evdev bits,
including whether `REL_WHEEL` or `/dev/uinput` was observed. Snapshot
identity reports kiosk/maint users, effective `sshd -T`
AllowUsers/password/root/pubkey/kbd-interactive/authentication methods,
authorized_keys presence without reading key bytes, and whether the
temporary sudoers file exists; it does not read sudoers contents.
Snapshot services include `sshd.service`. Watchdog observation never
reads `timeleft` or `/dev/watchdog`.

## Owns

| Concern | Note |
|---------|------|
| Seat identity bind | Device + capsule contracts when Runtime wires them |
| `seat.status` IPC | **wired** — P60 local session phase (`unbound` vs `bound`) from seat state |
| `edition.profile` | **wired** — honesty stub `offline-class` via Runtime `main_read` (F8) |
| `device.identity` | **wired** — honesty stub `bound:false` / empty id via Runtime `main_read` (F8) |
| `device.keys` | **wired** — default OS Secret Service ensure; Capsule open/import/verify load keyring SoT (P55); lab `EMPIIEROS_DEVICE_KEYS_BACKEND=runtime-file`; SK never in IPC |
| `entitlement.present` | **wired** — lab store → `present` + `verified` bits; Start needs both via `mayEnableStartCta` |
| `entitlement.lab_import` | **wired** — POST lab store from Capsule; redacted license id; verified:false until lab-verify |
| `entitlement.lab_verify` | **wired** — POST Ed25519 verify vs lab trust root; may set verified:true; **not** channel / not product Done |
| `enroll.welcome_policy` | **wired** — honesty stub `deferred` / `show_welcome:false`; welcome gated by `mayShowWelcomeSurface` |
| `enroll.carriers` | **wired** — path/policy honesty; default `usb/air:false`; lab `EMPIIEROS_USB_PICK=1` → `usb_ecap_pick`; lab `EMPIIEROS_AIR_CODE=1` → `air_code` (no WSS/Air execute) |
| `enroll.usb_ecap_pick` | **wired** — POST pick; redacted `name_redacted` + `bytes`; no Capsule import; needs `EMPIIEROS_USB_PICK=1` |
| `enroll.usb_ecap_validate` | **wired** — POST structure validate; `valid`≠crypto-open; no import |
| `enroll.usb_ecap_open` | **wired** — POST crypto-open lab; `opened` only; payload never in IPC; no import/entitlement |
| `enroll.usb_ecap_import` | **wired** — POST lab import/bind; redacted bind file; **not** entitlement/channel/Start |
| `enroll.capsule_bind_status` | **wired** — GET lab `bound`; ≠ `device.identity.bound` · ≠ entitlement |
| `enroll.air_probe` | **wired** — P61 Air WSS protocol & access code probe (`port`, `selector_redacted`, `endpoint`, `probe_status`); secret zeroized and never in IPC |
| `enroll.air_execute` | **wired** — P64 live RFC 6455 WebSocket handshake execution with Air edge controller; binary capsule stream, crypto-open with entitlement, atomic bind; secret zeroized and never in IPC |
| `host_network.status` | **wired** — P58 nmcli probe (`backend`/`link`/`kind`); `nm_gui:false`; never fake `up` without connected device; not W-transport |
| `network.link_snapshot` | **wired** — product taskbar chip (`kind` ethernet\|wifi + `active`); same order as the shell tray (associated Wi-Fi, else up ethernet); no IP/DNS/gateway/SSID |
| `transport.status` | **wired** — P59 loopback SOCKS probe (`listening`/`proxy_url`/`reason`); default port probe; `protected:false`; never fake listening or L7 protected |
| `kernel_offline.status` | **wired** — P69 offline catalog & presenter presentation; schema v7 mount policy SoT (`kernel_offline.catalog`); presenter mount execution (`kernel_offline.open-mount`, `presentation_url` only); `enforced:true`, strict `air_gap:false` honesty; writes `last_mount.json` (0o600) |
| `kernel_offline.storage` | **wired** — P70 offline storage discovery & encrypted mount management; allowlisted volumes (`offline_vault`, `offline_media`, `offline_backup`); safe mount/unmount execution under `offline_storage`; `air_gap:false` honesty; writes `storage_audit.json` (0o600) |
| `fs_bridge.status` | **wired** — P71 FS bridge staging & import isolation; `staging_active:true`, privilege honesty `enabled:false` / `writable:false` (Shell not privilege SoT); isolated staging root (`fs_bridge/staging/`); allowlisted source volumes (`offline_vault`, `offline_media`, `offline_backup`); quota enforcement (50 MiB/file, 200 MiB total); writes `staging_audit.json` (0o600) |
| `tray.status` | **wired** — P72 system tray & notification coordination (`GET /v0/tray/status`); `tray_available:true`, `kiosk_quiet_mode`, unread badge count, allowlisted menu items |
| `tray.action` | **wired** — P72 allowlisted tray action dispatch (`POST /v0/tray/action`); `focus_shell`, `toggle_quiet_mode`, `request_quick_panel`, `clear_notifications`, `request_restart`; writes `tray_audit.json` (0o600) |
| `notification.post` | **wired** — P72 kiosk notification queue routing (`POST /v0/notification/post`); severity hierarchy (`info`, `warning`, `critical`); kiosk quiet mode suppression; bounded queue (max 50); writes `tray_audit.json` (0o600) |
| `notification.list` | **wired** — P72 query active notification queue (`GET /v0/notification/list`); unread count, critical count, quiet mode status |
| `notification.dismiss` | **wired** — P72 dismiss notifications (`POST /v0/notification/dismiss`); individual dismiss, mark as read, or clear all; writes `tray_audit.json` (0o600) |
| `seat.window_status` | **wired** — P73 virtual seat floor layout & window status (`GET /v0/seat/window-status`); `virtual_seat_id:"seat0"`, `layout_mode`, `focused_window_id`, allowlisted windows |
| `seat.window_action` | **wired** — P73 allowlisted window action dispatch (`POST /v0/seat/window-action`); `focus`, `minimize`, `maximize`, `restore`, `close`, `set_layout`, `reset_layout`; z-order focus cascade fallback; pinned shell protection; writes `seat_audit.json` (0o600) |
| `audio.status` | **wired** — P74 audio subsystem & hardware device discovery (`GET /v0/audio/status`); `sound_server:"pipewire-pulse"`, `server_state`, volume levels, default sink/source, discovered devices across buses (`pci_hda`, `hdmi`, `usb`) |
| `audio.action` | **wired** — P74 allowlisted audio action dispatch (`POST /v0/audio/action`); `set_master_volume`, `set_master_mute`, `set_input_volume`, `set_input_mute`, `select_sink`, `select_source`, `set_profile`, `rescan_devices`; bounded volume [0, 100]; writes `audio_audit.json` (0o600) |
| `display.status` | **wired** — P75 display topology & multi-screen projection (`GET /v0/display/status`); `drm_driver:"modesetting"`, `server_protocol:"wayland-kiosk"`, `projection_mode`, `multi_display_active`, discovered displays across connectors (`eDP-1`, `HDMI-A-1`, `DP-1`) |
| `display.action` | **wired** — P75 allowlisted display action dispatch (`POST /v0/display/action`); `set_projection_mode`, `set_resolution`, `set_orientation`, `hotplug_simulate`, `rescan_displays`; primary shell display protection; writes `display_audit.json` (0o600) |
| `power.status` | **wired** — P76 power subsystem & thermal health monitoring (`GET /v0/power/status`); `power_source`, `ac_online`, `battery_level`, `battery_state`, `power_profile`, `thermal_throttling`, `thermal_status`, `supplies`, `thermal_zones` |
| `power.action` | **wired** — P76 allowlisted power action dispatch (`POST /v0/power/action`); `set_power_profile`, `set_display_sleep_timeout`, `toggle_sleep_inhibit`, `set_screen_blank`, `simulate_power_event`, `rescan_sensors`; writes `power_audit.json` (0o600) |
| `input.status` | **wired** — P77 input peripherals & device authorization (`GET /v0/input/status`); `kiosk_grab_mode`, `block_unauthorized_usb`, `virtual_keyboard_enabled`, `devices_count`, `authorized_count`, `unauthorized_count`, `devices` |
| `input.action` | **wired** — P77 allowlisted input action dispatch (`POST /v0/input/action`); `set_device_authorization`, `toggle_kiosk_grab`, `toggle_block_unauthorized_usb`, `toggle_virtual_keyboard`, `simulate_device_event`, `rescan_devices`; primary seat input protection; writes `input_audit.json` (0o600) |
| `time.status` | **wired** — P78 system time, NTP sync & regional locale (`GET /v0/time/status`); `unix_timestamp`, `monotonic_uptime_secs`, `ntp_enabled`, `ntp_status`, `ntp_server`, `timezone`, `utc_offset_mins`, `dst_active`, `locale` |
| `time.action` | **wired** — P78 allowlisted system time action dispatch (`POST /v0/time/action`); `sync_ntp`, `toggle_ntp`, `set_timezone`, `set_locale`, `set_ntp_server`, `adjust_time_manual`, `simulate_clock_drift`; bounded manual step adjustments; writes `time_audit.json` (0o600) |
| `settings.status` | **wired** — P79 system settings hub & multi-subsystem preferences (`GET /v0/settings/status`, alias `/v0/settings/hub`); `active_preset`, `operator_mode`, `theme`, `diagnostics_enabled`, `auto_lock`, `power_summary`, `display_summary`, `audio_summary`, `input_summary`, `time_summary` |
| `settings.action` | **wired** — P79 allowlisted settings action dispatch (`POST /v0/settings/action`); `apply_preset`, `set_operator_mode`, `set_theme`, `toggle_diagnostics`, `toggle_auto_lock`, `reset_preferences`; writes `settings_audit.json` (0o600) |
| `network.status` | **wired** — P80 network manager & connection wizard (`GET /v0/network/status`, alias `/v0/network/manager`); `interfaces`, `active_connection`, `wifi_scan_results`, `offline_mode`, `dns_servers`, `saved_profiles_count` |
| `network.action` | **wired** — P80 allowlisted network action dispatch (`POST /v0/network/action`); `wifi_scan`, `connect_wifi`, `disconnect_wifi`, `configure_ipv4`, `set_dns`, `toggle_offline_mode`, `reset_network_config`; writes `network_audit.json` (0o600) |
| `onboarding.status` | **wired** — P81 first-run onboarding & kiosk welcome experience (`GET /v0/onboarding/status`, alias `/v0/onboarding/wizard`); `first_run_completed`, `current_step`, `completed_steps`, `locale`, `timezone`, `network_verified`, `hardware_verified`, `terms_accepted`, `terms_version`, `welcome_dismissed`, `can_complete` |
| `onboarding.action` | **wired** — P81 allowlisted onboarding action dispatch (`POST /v0/onboarding/action`); `start_onboarding`, `set_step`, `set_locale_timezone`, `verify_network`, `verify_hardware`, `accept_terms`, `complete_onboarding`, `dismiss_welcome`, `reset_onboarding`; writes `onboarding_audit.json` (0o600) |
| `diagnostics.status` | **wired** — P82 shell diagnostics & IPC telemetry hub (`GET /v0/diagnostics/status`, alias `/v0/diagnostics/telemetry`); `diagnostics_status`, `total_requests`, `ok_requests`, `denied_requests`, `error_requests`, `avg_latency_ms`, `max_latency_ms`, `uptime_seconds`, `extended_telemetry`, `crash_count`, `error_count`, `subsystems`, `events` |
| `diagnostics.action` | **wired** — P82 allowlisted diagnostics action dispatch (`POST /v0/diagnostics/action`); `record_event`, `clear_events`, `reset_metrics`, `toggle_extended_telemetry`, `run_self_test`; writes `diagnostics_audit.json` (0o600) |
| `diagnostics.export` | **wired** — P82 sanitized diagnostic export bundle (`POST /v0/diagnostics/export`); packages subsystem snapshot, metrics, and event log into `diagnostics_export.json` (0o600) with credentials redacted |
| `virtual_keyboard.status` | **wired** — P83 dedicated on-screen virtual keyboard layout engine (`GET /v0/virtual-keyboard/status`, alias `/v0/virtual-keyboard/state`); `visible`, `layout`, `layouts_available`, `accessibility_mode`, `touch_feedback`, `key_sound`, `auto_caps`, `gesture_navigation`, `input_target`, `buffer_len`, `buffer_masked`, `total_keystrokes`, `last_interaction_epoch` |
| `virtual_keyboard.action` | **wired** — P83 allowlisted virtual keyboard action dispatch (`POST /v0/virtual-keyboard/action`); `show`, `hide`, `set_layout`, `toggle_accessibility`, `toggle_touch_feedback`, `toggle_gesture_navigation`, `key_press`, `clear_buffer`, `set_target`, `reset_settings`; sensitive input protection; writes `virtual_keyboard_audit.json` (0o600) |
| `security.integrity.status` | **wired** — P84 security integrity guard & substrate verity (`GET /v0/security/integrity/status`, alias `/v0/security/integrity/state`); `verity_status`, `root_hash`, `verity_device`, `read_only_root`, `tamper_detected`, `measured_boot`, `secure_boot_enabled`, `kernel_lockdown_mode`, `lockdown_active`, `lockdown_reason`, `lockdown_level`, `tamper_events_count`, `last_scan_epoch` |
| `security.integrity.action` | **wired** — P84 allowlisted security integrity action dispatch (`POST /v0/security/integrity/action`); `run_integrity_check`, `trigger_lockdown`, `release_lockdown`, `acknowledge_tamper`, `reset_integrity_state`; two-stage lockdown release confirmation; writes `security_integrity_audit.json` (0o600) |
| `seat.browser.*` | **wired (S3 overlay chrome + ≤3 tabs + downloads)** — status/policy/evaluate + navigate/reload/history/tab `host_action`; Shell tile + GtkStack of related WebViews; `media.set`; `download.begin`/`finish` into `{state}/seat_browser/downloads/`; `system.snapshot` family `seat.browser`; clearnet debt; contract [`SEAT-BROWSER.md`](SEAT-BROWSER.md) |
| `seat.shield.*` | **wired (S5A paste)** — Unix NDJSON via Runtime `core.rs`; status calls `maybe_auto_observe`; cycle/observe may apply protected auto-act only when armed; on-demand `scan_path`/`quarantine*` tmp/shm only; auto-hold only after `ioc_hash_match` + `ENABLE_QUARANTINE`; `act` needs helper; twin `security.integrity` stays `disabled`; contract [`SEAT-SHIELD.md`](SEAT-SHIELD.md) |
| `operator.auth.status` | **wired** — P85 local operator authenticator & kiosk maintenance (`GET /v0/operator/auth/status`, alias `/v0/operator/auth/state`); `operator_role`, `authenticated`, `maintenance_active`, `session_expires_epoch`, `session_ttl_remaining_secs`, `failed_attempts`, `max_failed_attempts`, `locked_out`, `locked_out_until_epoch`, `pin_enrolled` |
| `operator.auth.action` | **wired** — P85 allowlisted operator auth action dispatch (`POST /v0/operator/auth/action`); `authenticate_pin`, `elevate_maintenance`, `exit_maintenance`, `extend_session`, `change_pin`, `reset_operator_state`; brute-force lockout protection; writes `operator_auth_audit.json` (0o600) |
| `seat.workspace.status` | **wired** — P86 product shell seed & workspace navigation (`GET /v0/seat/workspace/status`, alias `/v0/seat/workspace/state`); `active_workspace`, `available_workspaces`, `operator_elevated`, `kiosk_mode`, `workspace_count`, `last_switch_epoch` |
| `seat.workspace.action` | **wired** — P86 allowlisted workspace action dispatch (`POST /v0/seat/workspace/action`); `switch_workspace`, `toggle_kiosk_layout`, `reset_workspace_nav`; writes `workspace_audit.json` (0o600) |
| `product.payload.status` | **wired** — P87 real application payload packaging & cutover status (`GET /v0/product/payload/status`, alias `/v0/product/payload/state`); `payload_present`, `package_id`, `app_build_id`, `version`, `channel`, `manifest_valid`, `entrypoints`, `total_files`, `total_bytes`, `installed_epoch`, `detail` |
| `product.payload.action` | **wired** — P87 allowlisted application payload action dispatch (`POST /v0/product/payload/action`); `install_seed_payload`, `verify_payload`, `rollback_payload`, `clear_payload`; writes `payload_audit.json` (0o600) |
| `lifecycle.restart` | **wired** — P67 ticketed restart coordination (`allowed:true` when ticket pending); two-stage cryptographic ticket (`rek_<hex32>`, TTL 300s) + restart execute; writes `restart_signal.json` (0o600) for kiosk supervisor; fail-closed `{ok, action, reason}` |
| `auth.status` | **wired** — P60 local session auth state (`authenticated`, `session_id_redacted`); `lab-login` / `lab-logout` ops; token never in IPC |
| `license.status` | **wired** — P63 license certificate verification & seat query (`valid`, `license_id_redacted`, `edition`, `exp`); Ed25519 lab trust root check |
| `license.activate` | **wired** — P63 seat license activation (`POST /v0/license/activate`); verifies signed document or promotes verified entitlement; binds seat session |
| `license.deactivate` | **wired** — P63 seat license deactivation (`POST /v0/license/deactivate`); revokes local seat license binding |
| `seat.local_factory_reset` | **wired** — P65 factory reset orchestrator (`allowed:true`); two-stage cryptographic ticket (`ticket`, TTL 300s) + wipe execute; clears auth, seat, license, capsule, entitlement, device keys, channel packages, lifecycle, product, kernel_offline, offline_storage, fs_bridge, kiosk_tray, seat_floor, audio_hardware, display_topology, power_subsystem, input_peripherals, system_time, settings_hub, network_manager, onboarding, diagnostics, virtual_keyboard, security_integrity, **seat_shield**, operator_auth, workspace_nav, payload_packaging, substrate_image; fail-closed `{ok, cleared, skipped, failed}` |
| `product.open` | **wired** — P68 typed product launch (`POST /v0/product/open`); Typed Target Law (`fofline_local`, `connected_cloud`, `spa_deep_link`); `allowed:true` when verified package in `current`; writes `last_launch.json` (0o600); no URL-from-JS |
| `update.apply` / `update.channel_status` / `channel.package_unpack` | **disabled (production)** — secure-reset `core.rs` blocks prefix `update.`; historical lab HTTP / `channel_package` is **not** the OS OTA SoT. Capsule-gated OTA is `launcher.ota_*` per [`OTA-CAPSULE.md`](OTA-CAPSULE.md) (P0 lock; apply fail-closed until plant). A/B P93 stays frozen. |
| `launcher.ota_status` / `launcher.ota_request` / `launcher.ota_apply` | **wired (P1–E4)** — hard gate license+hardware+OIDC+portal+artifact PK; request uses VIP helper; empty edge → `no_capsule`; missing PK → `artifact_pk_missing`; without helper → `ota_helper_required`; apply cutover planted for `os_component` (`ota_not_staged` / `os_before_payload` / success). See [`OTA-CAPSULE.md`](OTA-CAPSULE.md). |
| `substrate.profile` | **wired** — P62 substrate dual-flag profile query (`GET /v0/substrate/profile`); resolves payload profile (`empiieros-shell` vs `tauri`), payload trust, and exclusivity validation |
| `substrate.image.status` | **wired** — P88 substrate dual-flag OS image integration status (`GET /v0/substrate/image/status`, alias `/v0/substrate/image/state`); `recipe_ready`, `dual_flag_valid`, `payload_profile`, `payload_trust`, `os_trust`, `target_device`, `device_verified`, `device_model`, `device_size_bytes`, `removable_usb`, `rootfs_sha256`, `kernel_sha256`, `last_action_epoch`, `detail` |
| `substrate.image.action` | **wired** — P88 allowlisted substrate image action dispatch (`POST /v0/substrate/image/action`); `prepare_image_recipe`, `probe_target_media`, `stage_image_plan`, `clear_audit`; writes `substrate_image_audit.json` (0o600) |
| `substrate.metal.status` | **wired** — P89 substrate metal boot reference validation status (`GET /v0/substrate/metal/status`, alias `/v0/substrate/metal/state`); `boot_mode`, `secure_boot_enabled`, `kernel_cmdline`, `cmdline_compliant`, `payload_profile`, `payload_trust`, `os_trust`, `install_complete`, `kiosk_disabled`, `recovery_mode_active`, `metal_root_device`, `last_action_epoch`, `detail` |
| `substrate.metal.action` | **wired** — P89 allowlisted substrate metal action dispatch (`POST /v0/substrate/metal/action`); `switch_profile`, `toggle_disable_kiosk`, `verify_boot_compliance`, `clear_audit`; writes `substrate_metal_audit.json` (0o600) |
| `substrate.self_healing.status` | **wired** — P90 offline self-healing & crash isolation status (`GET /v0/substrate/self-healing/status`, alias `/v0/substrate/self-healing/state`); `status`, `watchdog_active`, `heartbeat_miss_count`, `max_heartbeat_misses`, `compositor_healthy`, `shell_ui_healthy`, `runtime_healthy`, `last_heartbeat_epoch`, `crash_count`, `consecutive_crashes`, `quarantine_active`, `quarantined_items`, `hang_recovery_enabled`, `last_recovery_action`, `last_recovery_epoch`, `audit_entries_count`, `detail` |
| `substrate.self_healing.action` | **wired** — P90 allowlisted self-healing action dispatch (`POST /v0/substrate/self-healing/action`); `heartbeat`, `record_crash`, `quarantine_state`, `recover_compositor`, `reset_crash_counters`, `clear_audit`; writes `self_healing_audit.json` (0o600) |
| `substrate.watchdog_hw.status` | **wired** — P91 hardware watchdog daemon & hardware ping status (`GET /v0/substrate/watchdog-hw/status`, alias `/v0/substrate/watchdog-hw/state`); `watchdog_present`, `device_path`, `driver_name`, `identity`, `timeout_secs`, `timeleft_secs`, `pretimeout_secs`, `options_flags`, `bootstatus_flags`, `daemon_active`, `ping_count`, `last_ping_epoch`, `magic_close_supported`, `systemd_watchdog_active`, `audit_entries_count`, `detail` |
| `substrate.watchdog_hw.action` | **wired** — P91 allowlisted hardware watchdog action dispatch (`POST /v0/substrate/watchdog-hw/action`); `ping`, `configure`, `arm`, `disarm`, `trigger_test_reboot`, `clear_audit`; writes `watchdog_hw_audit.json` (0o600) |
| `substrate.shell_gui.status` | **wired** — P92 Wayland kiosk runner & shell UI packaging status (`GET /v0/substrate/shell-gui/status`, alias `/v0/substrate/shell-gui/state`); `gui_present`, `binary_path`, `assets_present`, `assets_path`, `index_html_present`, `bundle_size_bytes`, `serving_mode`, `active_url`, `fullscreen`, `wayland_backend`, `last_reload_epoch`, `reload_count`, `audit_entries_count`, `detail` |
| `substrate.shell_gui.action` | **wired** — P92 allowlisted shell GUI action dispatch (`POST /v0/substrate/shell-gui/action`); `verify_assets`, `reload`, `configure_url`, `clear_audit`; writes `shell_gui_audit.json` (0o600) |
| Session bootstrap FSM | **v0 helpers** — Identifying-until-resolved in Shell (`bootstrapPhase.ts`) |
| Floor policy pointers | labwc / getty / router / power / net — notes only |
| Dual-flag honesty | One active payload profile; do not break living kiosk |

## Must not

- Import Buildroot / Trusted Substrate factory into this repo
- Edit monorepo kiosk profiles from EmpiierOS sessions
- Hold secrets or license enforcement (Runtime / F8)
- Show enroll/activation chrome during Identifying
- Pretend Shell UI owns privilege policy
- Fake `connected` / `fofline` without tunnel or local pack
- Invent a durable `device_id` while unbound
- Enable Start while entitlement is absent
- Show welcome chrome while policy is deferred or during Identifying
- Fake authenticated session without Runtime auth material
- Fake valid license without Runtime license material
- Execute factory reset / wipe without ticket + orchestrator
- Execute restart / reload without ticket + supervisor coordination
- Open product via URL from JS / fake `product.open` allowed
- Open arbitrary presenter mounts outside allowlisted presentation_url mode / fake air-gap claim
- Perform arbitrary storage mounts or path traversal / fake hardware air-gap claim
- Perform arbitrary host filesystem read/write or path traversal outside allowlisted staging roots / fake host filesystem bridge enabled
- Spawn arbitrary processes from tray menu items or unconfined DBus notification daemons
- Spawn unconfined desktop window managers (GNOME/KDE) or close pinned primary desktop shell window
- Spawn unconfined external sound daemons or execute root command line audio tools
- Spawn unconfined external display managers (GDM/SDDM) or invoke root xrandr scripts / disable primary kiosk display
- Spawn unconfined external power daemons (upowerd/tlp) or execute root power management scripts
- Spawn unconfined external input managers (udevd/xinput/evtest) or execute root input rules scripts / deauthorize all seat inputs
- Spawn unconfined external time daemons (systemd-timesyncd/chronyd/ntpd) or execute root time scripts (timedatectl/date/hwclock) / perform unbounded clock leaps
- Spawn unconfined desktop control centers (gnome-control-center/systemsettings5/lxqt-config) or execute desktop configuration scripts / bypass fail-closed subsystem ACLs
- Spawn unconfined network management applets (nm-connection-editor/gnome-control-center) or execute root network scripts (netplan/ip link/iptables) / bypass fail-closed network ACLs
- Spawn unconfined setup assistants (gnome-initial-setup/oobe scripts) or bypass first-run onboarding invariants / fake completed onboarding
- Spawn unconfined diagnostic tools (journalctl/systemd-analyze/top/gdb) or execute root debugging scripts / leak unredacted credentials in diagnostic exports
- Spawn unconfined virtual keyboard daemons (onboard/maliit/florence/wvkbd) or execute root input injection scripts / leak unmasked keystrokes or PINs in IPC status or audit logs
- Spawn unconfined integrity agents (aide/tripwire/osquery/falco) or execute uncontained root lockdown scripts / fake verity verified status without cryptographic hash measurement or bypass emergency lockdown mode
- Spawn unconfined local elevation agents (sudo/polkit/pkexec/PAM shadow editing) or execute root elevation scripts / bypass maintenance mode PIN verification or leak cleartext operator PINs in IPC status or audit logs
- Spawn unconfined desktop window managers (GNOME/KDE) or execute unconfined external windowing scripts / permit arbitrary unregistered workspaces
- Launch untyped, unsigned, or unverified application payloads outside `{state_dir}/channel_packages/current` / bypass Ed25519 manifest verification or leak arbitrary file URLs to untrusted callers
- Flash raw block devices without removable USB validation (`TRAN=usb`, `RM=1`) or target host storage partitions (`nvme`/internal disks) / fake substrate image recipe ready without Buildroot SoT digest verification
- Tamper with bootloader entries or kernel cmdlines without policy validation / fake metal boot integrity without sysfs/proc probing
- Spawn unconfined watchdog crash loops or delete state unconfined without quarantine audit / fake self-healing health without heartbeat verification
- Direct raw unconfined writes to `/dev/watchdog` from untrusted callers or fake hardware watchdog status without sysfs/device probe
- Apply live OTA / fake `update.apply` or `launcher.ota_apply` while gates fail, capsule missing (`ota_not_staged`), or fofline-before-OS (`os_before_payload`)
- Transfer an update capsule without license + hardware bind + live OIDC + portal VIP attest
- Label bootstrap `launcher.spa_sync` as capsule OTA (see [`SPA-PACK.md`](SPA-PACK.md) vs [`OTA-CAPSULE.md`](OTA-CAPSULE.md))
- Treat `usb`/`air` as media scanners / fake path available without Runtime proof
- Capsule import / Air WSS execute from Shell
- Flip `usb:true` without `EMPIIEROS_USB_PICK=1` (or later real pick probe)
- Flip `air:true` without `EMPIIEROS_AIR_CODE=1` (or later real Air path proof)
- Claim Air WSS / one-code execute from path probe alone
- Claim live activation from Air protocol probe alone
- Return raw Air access code secret or activation token over Shell IPC
- Return Capsule bytes / plaintext path in Shell IPC from usb pick
- Enable Start while `verified:false` (lab present alone is not enough)
- Claim channel / product Done from lab `verified:true`
- Claim `keyring_wired:true` without Secret Service proof (or sell `runtime-file` as keyring)
- Flip `device.identity.bound` from lab capsule bind
- Return Capsule plaintext payload / activation token / raw license_id over Shell IPC

## Honesty

Shell gates display with pure FSM helpers; Runtime remains ACL SoT for bridge honesty ops. Floor execution stays kinship until a later wave.

## Production path (secure reset)

The historical P-wave table above records lab HTTP simulations. Production Runtime
on metal is Unix `SO_PEERCRED` IPC at `/run/empiieros-shell/runtime.sock` →
`systemd-socket-proxyd` → `/run/empiieros/runtime.sock`. Those prefixes stay
fail-closed except the allowlisted operations in [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md).
`power.action` remains disabled; the production reboot/poweroff op is
`system.power`.
