from pathlib import Path
import os

manifest = Path("runtime/Cargo.toml").read_text(encoding="utf-8")
daemon = Path("runtime/src/daemon.rs").read_text(encoding="utf-8")
core = Path("runtime/src/core.rs").read_text(encoding="utf-8")

assert 'default = ["production"]' in manifest
assert 'path = "src/core.rs"' in manifest
assert 'path = "src/daemon.rs"' in manifest
assert "empiieros-air-enroll" in manifest
assert "UnixListener" in daemon
assert "SO_PEERCRED" in daemon
assert "fn parse_seccomp_status" in daemon
assert "empiieros-runtime: seccomp mode=" in daemon
assert "DEFAULT_ALLOWED_BRIDGE_EXE" in daemon
assert "/usr/lib/systemd/systemd-socket-proxyd" in daemon
assert "CAP_SYS_PTRACE" not in daemon
assert "EMPIIEROS_ALLOWED_SHELL_" not in daemon
secure_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "/run/empiieros-shell/runtime.sock" in secure_main
assert "webkit_web_context_set_sandbox_enabled(context, FALSE)" in secure_main
assert "host CA/NSS product HTTPS" in secure_main
assert "webkit_web_context_set_sandbox_enabled(context, TRUE)" not in secure_main
assert "CAP_SYS_PTRACE" not in secure_main
assert "dev-peer-bypass" in daemon
assert "disabled_operation" in core
assert "system.power" in core
assert "display.control" in core
assert "mod display_control" in core
assert "audio.control" in core
assert "mod audio_control" in core
assert "network.wifi.radio" in core
assert "mod radio_control" in core
assert "time.sync" in core
assert "mod time_control" in core
assert Path("runtime/helpers/empiieros-audio-volume.c").is_file()
assert Path("runtime/helpers/hda_speaker_amp.c").is_file()
assert Path("runtime/helpers/hda_speaker_amp.h").is_file()
assert Path("runtime/helpers/empiieros-audio-volume.service").is_file()
assert Path("runtime/helpers/empiieros-audio-amp.service").is_file()
audio_amp_src = Path("runtime/helpers/hda_speaker_amp.c").read_text(encoding="utf-8")
assert "0x10ec0236" in audio_amp_src
assert "0x102809c1" in audio_amp_src
assert "0x715" in audio_amp_src
assert "hda-verb" not in audio_amp_src
assert Path("runtime/helpers/empiieros-audio-volume.c").read_text(encoding="utf-8").count(
    "empiieros_hda_enable_speaker_amp"
) >= 2
assert Path("runtime/helpers/empiieros-wifi-radio").is_file()
assert Path("runtime/helpers/empiieros-time-sync").is_file()
assert Path("runtime/helpers/empiieros-wifi-restore").is_file()
assert Path("runtime/helpers/empiieros-air-enroll.service").is_file()
assert Path("runtime/helpers/seat-shield/seat_shield.nft").is_file()
assert Path("runtime/helpers/seat-shield/empiieros-seat-shield-nftables.service").is_file()
assert Path("runtime/seat_shield_core/Cargo.toml").is_file()
assert Path("runtime/seat_shield_probe/Cargo.toml").is_file()
assert Path("runtime/seat_shield_probe/ORIGIN.md").is_file()
probe_origin = Path("runtime/seat_shield_probe/ORIGIN.md").read_text(encoding="utf-8")
assert "inet_diag" in probe_origin
assert "No eBPF" in probe_origin or "no eBPF" in probe_origin
assert "AF_PACKET" in probe_origin
nft_unit = Path("runtime/helpers/seat-shield/empiieros-seat-shield-nftables.service").read_text(encoding="utf-8")
assert "nft -f /usr/share/empiieros/seat-shield/seat_shield.nft" in nft_unit
assert "NoNewPrivileges=true" in nft_unit
assert "CapabilityBoundingSet=CAP_NET_ADMIN" in nft_unit
assert "ProtectHome=tmpfs" in nft_unit
assert "ProtectSystem=strict" in nft_unit
assert "Type=oneshot" in nft_unit
assert "IPAddressDeny=" not in nft_unit
fw_unit = Path("runtime/helpers/seat-shield/empiieros-seat-shield-fw.service").read_text(encoding="utf-8")
assert "Type=oneshot" in fw_unit
assert "ListenStream" not in fw_unit
assert "ListenDatagram" not in fw_unit
assert "NoNewPrivileges=true" in fw_unit
assert "CapabilityBoundingSet=CAP_NET_ADMIN" in fw_unit
assert "ProtectHome=tmpfs" in fw_unit
assert "ProtectSystem=strict" in fw_unit
assert "IPAddressDeny=" not in fw_unit
watch_unit = Path("runtime/helpers/seat-shield/empiieros-seat-shield-watch.service").read_text(
    encoding="utf-8"
)
assert "PrivateTmp=false" in watch_unit
assert "Do not share the Runtime unit's PrivateTmp namespace." in watch_unit
assert "nft" not in watch_unit
assert "IPAddressDeny=" not in watch_unit
assert "CapabilityBoundingSet" not in watch_unit
assert "CAP_NET_ADMIN" not in watch_unit
assert "--state-dir /var/lib/empiieros-runtime" in watch_unit
runtime_toml = Path("runtime/Cargo.toml").read_text(encoding="utf-8")
assert 'panic = "abort"' in runtime_toml
shield_toml = Path("runtime/seat_shield_core/Cargo.toml").read_text(encoding="utf-8")
assert 'panic = "abort"' in shield_toml
assert 'unsafe_code = "forbid"' in shield_toml
assert Path("scripts/runtime-cargo-audit.sh").is_file()
audit_sh = Path("scripts/runtime-cargo-audit.sh").read_text(encoding="utf-8")
assert "--deny unsound" in audit_sh
assert "--file runtime/Cargo.lock" in audit_sh
assert "--file runtime/seat_shield_core/Cargo.lock" in audit_sh
assert "GPL-fail" in audit_sh
assert "blocked4" in Path("runtime/helpers/seat-shield/seat_shield.nft").read_text(encoding="utf-8")
assert "blocked6" in Path("runtime/helpers/seat-shield/seat_shield.nft").read_text(encoding="utf-8")
nft_rs = Path("runtime/seat_shield_core/src/fw/nft.rs").read_text(encoding="utf-8")
assert "fn parse_nft_set_text" in nft_rs
assert "JSON support not compiled-in" in nft_rs
assert Path("runtime/seat_shield_core/src/parser_probe.rs").is_file()
probe_rs = Path("runtime/seat_shield_core/src/parser_probe.rs").read_text(encoding="utf-8")
assert "must not exec the floor helper" in probe_rs
assert Path("runtime/seat_shield_core/tests/parser_fuzz.rs").is_file()
fuzz_rs = Path("runtime/seat_shield_core/tests/parser_fuzz.rs").read_text(encoding="utf-8")
assert "corpus_and_prng_do_not_panic_or_exec" in fuzz_rs
assert Path("runtime/seat_shield_core/tests/parser_corpus").is_dir()
assert Path("runtime/seat_shield_core/fuzz/README.md").is_file()
fw_md = Path("session/SEAT-SHIELD-FW.md").read_text(encoding="utf-8")
assert "nft list set FAMILY TABLE SET" in fw_md
assert Path("runtime/src/bin/air_enroll_helper.rs").is_file()
helper_unit = Path("runtime/helpers/empiieros-air-enroll.service").read_text(encoding="utf-8")
assert "EMPIIEROS_CAPSULE_SIGNING_PK=26b36dae25359712f8b328482a54f835c558c9bf708fb4a1d513ca7f165992d6" in helper_unit
capsule = Path("runtime/src/capsule_v3.rs").read_text(encoding="utf-8")
assert "26b36dae25359712f8b328482a54f835c558c9bf708fb4a1d513ca7f165992d6" in capsule
assert 'payload_obj.get("activation_token")' in capsule
assert "load_issuer_verifying_pk" in capsule
harness = Path("scripts/metal-live.sh").read_text(encoding="utf-8")
assert "deploy-helper" in harness
assert "deploy-seat-shield" in harness
assert "deploy-seat-shield-watch" in harness
assert "mod launcher_air" in core
assert "mod device_request" in core
assert "launcher.air_activate" in core
assert "launcher.pin_issue" in core
assert "installer_issued_only" in core
assert "launcher_pin::issue(self.state.root()" not in core
assert "launcher.pin_unlock" in core
assert "mod launcher_pin" in core
activator = Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "launcher.pin_unlock" in activator
assert "launcher.pin_issue" not in activator
pin = Path("runtime/src/launcher_pin.rs").read_text(encoding="utf-8")
assert "empiieros-first-boot-pin-v1|" in pin
assert "fn require_unlocked" in pin
assert "pin_not_unlocked" in pin
assert "require_unlocked(state_root)" in Path("runtime/src/launcher_capsule.rs").read_text(encoding="utf-8")
air = Path("runtime/src/launcher_air.rs").read_text(encoding="utf-8")
assert "require_unlocked(state_root)" in air
assert "air_helper_inner_missing" in air
assert air.index("bind_verified_blob") < air.index("session.send_ack")
assert "fn persist" in Path("runtime/src/launcher_inner.rs").read_text(encoding="utf-8")
assert "fn open_v3_inner" in Path("runtime/src/capsule_v3.rs").read_text(encoding="utf-8")
assert "entitlement_present" in Path("runtime/src/launcher_status.rs").read_text(encoding="utf-8")
assert "payload discarded" not in Path("runtime/src/launcher_capsule.rs").read_text(encoding="utf-8")
assert "fn validate_portal_conf" in Path("runtime/src/wg_conf.rs").read_text(encoding="utf-8")
assert "full_tunnel_forbidden" in Path("runtime/src/wg_conf.rs").read_text(encoding="utf-8")
assert "10.88.89.1/32" in Path("runtime/src/wg_conf.rs").read_text(encoding="utf-8")
assert "channel_valid" in Path("runtime/src/launcher_status.rs").read_text(encoding="utf-8")
assert "channel_up" in Path("runtime/src/launcher_status.rs").read_text(encoding="utf-8")
assert "inner.channel_up" in Path("runtime/src/launcher_status.rs").read_text(encoding="utf-8")
assert "launcher.channel_up" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "w_transport_start" not in Path("runtime/src/launcher_inner.rs").read_text(encoding="utf-8")
portal = Path("runtime/src/launcher_portal.rs").read_text(encoding="utf-8")
assert "launcher.channel_up" in core or "launcher_portal::bring_up" in core
assert "kernel_wg_missing" in portal
assert "wg-quick" not in portal
assert "boringtun" not in portal
assert "IPAddressDeny" not in Path("runtime/helpers/empiieros-portal-up.service").read_text(encoding="utf-8")
portal_unit = Path("runtime/helpers/empiieros-portal-up.service").read_text(encoding="utf-8")
assert "/etc/hosts" in portal_unit
hostmap = Path("runtime/src/portal_hostmap.rs").read_text(encoding="utf-8")
assert "# BEGIN empiieros-portal-hostmap" in hostmap
assert "10.88.89.1 app.eliteew.ru auth.eliteew.ru meet.eliteew.ru" in hostmap
assert "meet.eliteew.ru" in hostmap
assert "hostmap_symlink" in hostmap
assert "launcher_portal::apply_hostmap" in core
assert "hosts-air" in Path("runtime/helpers/empiieros-air-enroll.service").read_text(encoding="utf-8")
assert Path("session/PORTAL-HOSTMAP.md").is_file()
assert Path("runtime/src/bin/portal_up_helper.rs").is_file()
assert Path("scripts/build-wireguard-from-source.sh").is_file()
wg_frag = Path(os.environ["ELITEEW_TS_ROOT"] + "/config/buildroot/external/board/eliteew-ts/linux-wireguard.fragment").read_text(encoding="utf-8")
assert "CONFIG_WIREGUARD=m" in wg_frag
assert "linux-wireguard.fragment" in Path(os.environ["ELITEEW_TS_ROOT"] + "/config/buildroot/external/configs/eliteew_ts_x86_64_defconfig").read_text(encoding="utf-8")
prelicense = Path(os.environ["ELITEEW_TS_ROOT"] + "/config/buildroot/external/board/eliteew-ts/rootfs_overlay/etc/eliteew-trusted-substrate/nft/prelicense.nft").read_text(encoding="utf-8")
assert "tcp dport { 80, 443 } ip daddr 10.88.89.1 accept" in prelicense
assert "udp dport 13400 ip daddr 93.77.165.185 accept" in prelicense
assert "tcp dport 443 accept" not in prelicense
assert "deploy-wg" in harness
assert "deploy-portal" in harness
assert "deploy-spa" in harness
spa = Path("runtime/src/launcher_spa.rs").read_text(encoding="utf-8")
assert "launcher_spa::sync" in core
assert "allow_fofline_required" in spa
assert "channel_not_up" in spa
assert "signature_invalid" in Path("runtime/src/spa_pack.rs").read_text(encoding="utf-8")
assert "desktop-client-updates" in Path("runtime/src/spa_pack.rs").read_text(encoding="utf-8")
assert "FOFLINE_UPDATER_MINISIGN_PK" in Path("runtime/src/spa_pack.rs").read_text(encoding="utf-8")
assert "10.88.89.1" in Path("runtime/src/https_vip.rs").read_text(encoding="utf-8")
spa_unit = Path("runtime/helpers/empiieros-spa-fetch.service").read_text(encoding="utf-8")
assert "IPAddressDeny=any" in spa_unit
assert "IPAddressAllow=10.88.89.1" in spa_unit
assert "TimeoutStartSec=900" in spa_unit
assert Path("runtime/src/ota_capsule_store.rs").is_file()
ota_store = Path("runtime/src/ota_capsule_store.rs").read_text(encoding="utf-8")
assert "empiieros_ota_capsule_v1" in ota_store
assert "verify_and_stage" in ota_store
assert "artifact.pk.hex" in ota_store
_core_rs = Path("runtime/src/core.rs").read_text(encoding="utf-8")
assert '"factory_reset.",' in _core_rs
assert "fn retired_operation_families_stay_not_available" in _core_rs
assert '"factory_reset.execute",' in _core_rs
assert "Cutover (P4 + E4 plant" in Path("docs/plans/TZ-CAPSULE-OTA.md").read_text(
    encoding="utf-8"
)
assert "OS-before-payload" in Path("docs/plans/TZ-CAPSULE-OTA.md").read_text(encoding="utf-8")
assert "/usr/local/bin/empiieros-runtime" in Path("docs/plans/TZ-CAPSULE-OTA.md").read_text(
    encoding="utf-8"
)
assert "Cutover (E4 planted" in Path("session/OTA-CAPSULE.md").read_text(encoding="utf-8")
assert Path("docs/plans/OTA-OPERATOR-RUNBOOK.md").is_file()
_ota_rb = Path("docs/plans/OTA-OPERATOR-RUNBOOK.md").read_text(encoding="utf-8")
assert "stack-empiieros-ota" in _ota_rb
assert "invalid runtime response framing" in _ota_rb
assert "813cfc81" in _ota_rb
assert "OTA-OPERATOR-RUNBOOK.md" in Path("docs/foundation/PROJECT-TREE.md").read_text(
    encoding="utf-8"
)
assert Path("runtime/src/ota_cutover.rs").is_file()
_cutover = Path("runtime/src/ota_cutover.rs").read_text(encoding="utf-8")
assert "apply_current" in _cutover
assert "EMPIIEROS_OTA_CUTOVER_ROOT" in _cutover
assert "os_before_payload" in _cutover
assert "runtime/empiieros-shell-gui" in _cutover
assert "units_to_restart" in _cutover
assert "--on-active=3s" in _cutover
assert "invalid runtime response framing" in _cutover
assert "/usr/local/bin/empiieros-shell-gui" in Path("docs/plans/TZ-CAPSULE-OTA.md").read_text(
    encoding="utf-8"
)
assert "write_ota_baseline" in _cutover or "write_ota_baseline" in Path(
    "runtime/seat_shield_core/src/ota_observe.rs"
).read_text(encoding="utf-8")
assert "ota_not_staged" in Path("runtime/src/launcher_ota.rs").read_text(encoding="utf-8")
assert "ota_cutover::apply_current" in Path("runtime/src/launcher_ota.rs").read_text(
    encoding="utf-8"
)
assert Path("docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md").is_file()
_edge_tz = Path("docs/plans/TZ-EMPIERO-OS-OTA-EDGE.md").read_text(encoding="utf-8")
assert "tmpfs" in _edge_tz
assert "read_only" in _edge_tz
assert "Living CP artifact SK" in _edge_tz
assert "Host capsule bind" in _edge_tz and "forbidden" in _edge_tz
assert "/empiieros-ota/" in _edge_tz
assert "eliteew_empiieros_ota_upstream" in _edge_tz
assert "empiieros-ota-cp-ingest.py" in _edge_tz
assert "blob-ref" in _edge_tz
assert Path("scripts/empiieros-ota-cp-ingest.py").is_file()
_cp_ingest = Path("scripts/empiieros-ota-cp-ingest.py").read_text(encoding="utf-8")
assert "capsule_signature_required" in _cp_ingest
assert "podman" in _cp_ingest
assert "DEST_ROOT" in _cp_ingest
assert "openssl" in _cp_ingest
assert "path_escape" in _cp_ingest
assert "BLOB_HREF_PREFIX" in _cp_ingest
assert "def cmd_pack" in _cp_ingest
assert "def cmd_publish" in _cp_ingest
_ota_store = Path("runtime/src/ota_capsule_store.rs").read_text(encoding="utf-8")
assert "BLOB_HREF_PREFIX" in _ota_store
assert "verify_and_stage_with_blobs" in _ota_store
assert "parse_blob_href" in _ota_store
_https_vip = Path("runtime/src/https_vip.rs").read_text(encoding="utf-8")
assert 'OTA_ATTEST_PATH: &str = "/empiieros-ota/v1/attest"' in _https_vip
assert 'OTA_PREFIX: &str = "/empiieros-ota/"' in _https_vip
_launcher_ota = Path("runtime/src/launcher_ota.rs").read_text(encoding="utf-8")
assert "parse_attest_grant" in _launcher_ota
assert "ota_helper_required" in _launcher_ota
assert "execute_attest_and_fetch" in _launcher_ota
assert "HELPER_TIMEOUT_SECS: u64 = 900" in _launcher_ota
ota_unit = Path("runtime/helpers/empiieros-ota-fetch.service").read_text(encoding="utf-8")
assert "TimeoutStartSec=900" in ota_unit
assert "IPAddressDeny=any" in ota_unit
assert "IPAddressAllow=10.88.89.1" in ota_unit
assert "## Edge (E0" in Path("session/OTA-CAPSULE.md").read_text(encoding="utf-8")
assert "/empiieros-ota/" in Path("session/OTA-CAPSULE.md").read_text(encoding="utf-8")
assert "empiieros-ota-cp-ingest.py" in Path("session/OTA-CAPSULE.md").read_text(
    encoding="utf-8"
)
assert "blob-ref" in Path("session/OTA-CAPSULE.md").read_text(encoding="utf-8")
assert "portal_path_v1" in Path("session/OTA-CAPSULE.md").read_text(encoding="utf-8")
assert "no_capsule" in _edge_tz
assert "/v1/attest" in _edge_tz
assert "artifact_pk_missing" in _edge_tz or "artifact_pk_missing" in _launcher_ota
assert "plant_artifact_pk" in _ota_store
assert Path("scripts/empiieros-ota-plant-seat-pk.py").is_file()
_apply = Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "config/buildroot/external/board/eliteew-ts/rootfs_overlay/usr/local/bin/"
    "eliteew-ts-install-apply"
).read_text(encoding="utf-8")
assert "plant_seat_ota_pk" not in _apply
assert "require_seat_ota_pk" not in _apply
assert "plant_from_license" in _ota_store
assert Path("scripts/empiieros-ota-operator-key.sh").is_file()
assert Path("docs/plans/ELITEEW-TZ-OTA-PK-IN-AIR-LICENSE.md").is_file()
assert "OTA verify PK with the license" in Path("session/AIR-HANDSHAKE.md").read_text(
    encoding="utf-8"
)
assert "Living CP artifact SK" in _edge_tz
_stamp = Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "scripts/stamp-live-installer-media.sh"
).read_text(encoding="utf-8")
assert "plant_live_ota_pk" not in _stamp
assert "unplant_live_ota_keys" in _stamp
assert "plant_live_apply" in _stamp
assert "strip ESP OTA crutches" in Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "scripts/flash-live-installer-usb.sh"
).read_text(encoding="utf-8")
assert not Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "config/buildroot/external/board/eliteew-ts/rootfs_overlay/"
    "usr/local/share/eliteew-ts-installer/artifact.pk.hex"
).exists()
assert Path("scripts/empiieros-ota-e3-stage-measure.sh").is_file()
assert Path("scripts/empiieros-installer-media-unplant-ota.sh").is_file()
_plant_rt = Path("scripts/empiieros-installer-media-plant-runtime.sh").read_text(
    encoding="utf-8"
)
assert "share/eliteew-ts-installer/empiieros-runtime" in _plant_rt
assert "plant_from_persisted_inner" in _plant_rt
_ovl_apply = Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "config/buildroot/external/board/eliteew-ts/rootfs_overlay/"
    "usr/local/bin/eliteew-ts-install-apply"
).read_text(encoding="utf-8")
assert "extracted Runtime already IdP Date DPoP; keep SoT" in _ovl_apply
assert "planted OTA cutover ReadWritePaths" in _ovl_apply
assert "planted OTA fetch helper" in _ovl_apply
assert "plant_seat_ota_pk" not in _ovl_apply
assert "/opt/empiieros" in Path(
    os.environ["ELITEEW_TS_ROOT"] + "/"
    "config/buildroot/external/board/eliteew-ts/rootfs_overlay/"
    "etc/systemd/system/empiieros-runtime.service.d/ota-cutover.conf"
).read_text(encoding="utf-8")
assert "clearFeedback" in Path(
    "shell-ui/src/secure/components/SeatOtaSection.tsx"
).read_text(encoding="utf-8")
assert "artifact_pk_missing" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "no_capsule" in Path("shell-ui/src/secure/locale/en.ts").read_text(encoding="utf-8")
assert "otaChipArtifactPk" in Path("shell-ui/src/secure/components/SeatOtaSection.tsx").read_text(
    encoding="utf-8"
)
assert "ota_artifact_pk" in Path("runtime/src/launcher_ota.rs").read_text(encoding="utf-8")
assert "ota_baseline.json" in Path("session/SEAT-SHIELD.md").read_text(encoding="utf-8")
assert "### 9.3 OTA baseline" in Path("session/SEAT-SHIELD.md").read_text(encoding="utf-8")
assert "ota_capsule_staged" in Path("session/SEAT-SHIELD.md").read_text(encoding="utf-8")
assert "10.88.89.1" in Path("session/SEAT-SHIELD.md").read_text(encoding="utf-8")
assert Path("runtime/seat_shield_core/src/ota_observe.rs").is_file()
_ota_obs = Path("runtime/seat_shield_core/src/ota_observe.rs").read_text(encoding="utf-8")
assert 'OTA_PORTAL_VIP: &str = "10.88.89.1"' in _ota_obs
assert "observe_ota_baseline" in _ota_obs
assert "write_ota_baseline" in _ota_obs
assert "is_ota_portal_vip" in Path("runtime/seat_shield_core/src/cycle.rs").read_text(
    encoding="utf-8"
)
assert '"ota_baseline"' in Path("runtime/seat_shield_core/src/service.rs").read_text(
    encoding="utf-8"
)
assert "emit_ota_capsule_staged" in Path("runtime/src/ota_capsule_store.rs").read_text(
    encoding="utf-8"
)
assert "E4" in _edge_tz and "Measure green 2026-09-21" in _edge_tz
assert "ota_cutover" in _edge_tz
assert "launcher.ota_status" in Path("runtime/src/core.rs").read_text(encoding="utf-8")
assert "launcher.ota_request" in Path("runtime/src/core.rs").read_text(encoding="utf-8")
assert "launcher.ota_apply" in Path("runtime/src/core.rs").read_text(encoding="utf-8")
assert Path("runtime/src/bin/ota_fetch_helper.rs").is_file()
assert "spa_assets::sync_groups" in spa
assert "ASSETS_MANIFEST_PATH" in Path("runtime/src/spa_assets.rs").read_text(
    encoding="utf-8"
)
assert 'DEFERRED_GROUP: &str = "audio-guides"' in Path(
    "runtime/src/spa_assets.rs"
).read_text(encoding="utf-8")
assert '"index.html"' in Path("runtime/src/spa_assets.rs").read_text(encoding="utf-8")
assert Path("runtime/src/bin/spa_fetch_helper.rs").is_file()
assert "launcher.spa_sync" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "autoOpened" not in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "launcher.oidc_start" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "launcher.oidc_logout" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "foflineActivatorOidcLeave" in Path("shell-ui/src/secure/locale/ru.ts").read_text(
    encoding="utf-8"
)
assert "foflineActivatorOidcLeave" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "shell.auth_surface" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "from \"./foflineLauncherStage\"" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(
    encoding="utf-8"
)
assert "FoflineActivatorLab" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(
    encoding="utf-8"
)
assert "FoflineLauncherJourney" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(
    encoding="utf-8"
)
assert "foflineActivatorConsumeLocked" in Path(
    "shell-ui/src/desktop/foflineLauncherChrome.tsx"
).read_text(encoding="utf-8")
assert "FoflineLauncherJourney" in Path(
    "shell-ui/src/desktop/foflineLauncherChrome.tsx"
).read_text(encoding="utf-8")
assert "mayAutoBringUpPortal" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(
    encoding="utf-8"
)
assert "autoOpened" not in Path("shell-ui/src/desktop/foflineLauncherStage.ts").read_text(
    encoding="utf-8"
)
assert (
    'export const LAUNCHER_JOURNEY = [\n  "pin",\n  "consume",\n  "portal",\n'
    '  "spa",\n  "login",\n  "ready",\n] as const;'
    in Path("shell-ui/src/desktop/foflineLauncherStage.ts").read_text(encoding="utf-8")
)
assert "foflineHostTiles" in Path("shell-ui/src/desktop/foflineCatalog.ts").read_text(
    encoding="utf-8"
)
assert 'localKind !== "pack"' in Path("shell-ui/src/desktop/foflineCatalog.ts").read_text(
    encoding="utf-8"
)
assert "foflineHostTiles" in Path("shell-ui/src/desktop/FoflineDesktop.tsx").read_text(
    encoding="utf-8"
)
assert "visibleHostModules" in Path("shell-ui/src/desktop/FoflineDesktop.tsx").read_text(
    encoding="utf-8"
)
assert "FOFLINE_HOST_MODULES" in Path("shell-ui/src/desktop/foflineHostModules.ts").read_text(
    encoding="utf-8"
)
assert 'localKind === "pack"' not in Path("shell-ui/src/desktop/foflineHostModules.ts").read_text(
    encoding="utf-8"
)
assert Path("shell-ui/public/product-chrome/host-wallpaper.jpg").is_file()
assert Path("shell-ui/public/product-chrome/host-shield.png").is_file()
assert Path("shell-ui/public/product-chrome/host-system.png").is_file()
assert Path("shell-ui/public/product-chrome/host-network.png").is_file()
assert Path("shell-ui/public/product-chrome/host-diag.png").is_file()
assert Path("shell-ui/public/product-chrome/host-audio.png").is_file()
assert Path("shell-ui/public/product-chrome/host-events.png").is_file()
assert Path("shell-ui/public/product-chrome/host-browser.png").is_file()
assert Path("shell-ui/public/product-chrome/host-ota.png").is_file()
assert Path("shell-ui/public/product-chrome/tray-volume.png").is_file()
assert Path("shell-ui/public/product-chrome/tray-battery.png").is_file()
assert Path("shell-ui/public/product-chrome/tray-network.png").is_file()
assert "setSeatVolume" in Path("shell-ui/src/secure/components/taskbarTrayControl.ts").read_text(
    encoding="utf-8"
)
assert "wheelSeatPercent" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert Path("shell-ui/public/product-chrome/emperyworld-icon.svg").is_file()
assert "HOST_ICON_GRID = 110" in Path("shell-ui/src/desktop/foflineIconLayout.ts").read_text(
    encoding="utf-8"
)
assert "loadHostIconLayout" in Path("shell-ui/src/desktop/FoflineDesktop.tsx").read_text(
    encoding="utf-8"
)
assert "const [activatorOpen, setActivatorOpen] = useState(false)" in Path(
    "shell-ui/src/secure/SecureApp.tsx"
).read_text(encoding="utf-8")
assert "secure-taskbar-logo" in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(
    encoding="utf-8"
)
assert "taskbarRunning" in Path("shell-ui/src/secure/locale/en.ts").read_text(encoding="utf-8")
assert "taskbarRunning" in Path("shell-ui/src/secure/locale/ru.ts").read_text(encoding="utf-8")
assert "100dvh" in Path("shell-ui/src/secure/secure.css").read_text(encoding="utf-8")
assert "overflow-y: scroll" not in Path("shell-ui/src/secure/secure.css").read_text(
    encoding="utf-8"
)
assert "secure-app-window" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert "onPointerDown" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert "host-workspace-pos.v1" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert "WorkspaceFrame" in Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8")
assert "bottom: 1rem" in Path("shell-ui/src/desktop/fofline.css").read_text(encoding="utf-8")
assert "left: 8.4rem" not in Path("shell-ui/src/desktop/fofline.css").read_text(
    encoding="utf-8"
)
assert "8.4rem" not in Path("shell-ui/src/secure/secure.css").read_text(encoding="utf-8")
assert "TaskbarTray" in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(
    encoding="utf-8"
)
assert "system.power" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert "taskbarPowerMenu" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "taskbarPowerMenu" in Path("shell-ui/src/secure/locale/ru.ts").read_text(
    encoding="utf-8"
)
assert "Cloud-full Desktop is not mounted" not in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "foflineActivatorLabTools" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "foflineActivatorLabTools" in Path("shell-ui/src/secure/locale/ru.ts").read_text(
    encoding="utf-8"
)
assert "launcher_spa::edge_health" in core
assert '"launcher.edge_health"' in core
assert Path("runtime/src/https_vip.rs").read_text(encoding="utf-8").count('HEALTH_PATH: &str = "/api/health"') == 1
assert "launcher.edge_health" in Path("runtime/HEALTH-IPC.md").read_text(encoding="utf-8")
product = Path("runtime/src/launcher_product.rs").read_text(encoding="utf-8")
assert "fofline-spa://localhost/index.html?eliteewFoflineClient=1" in product
assert "remote_spa_forbidden" in product
assert "PACK_FILE_MODE: u32 = 0o644" in product
assert "STATE_TRAVERSE_MODE: u32 = 0o751" in product
assert "fn ensure_shell_traverse" in product
assert "ensure_secure_directory(&root, 0o751)" in Path("runtime/src/state_store.rs").read_text(
    encoding="utf-8"
)
assert "ensure_secure_directory(&root, 0o700)" not in Path("runtime/src/state_store.rs").read_text(
    encoding="utf-8"
)
spa_pack = Path("runtime/src/spa_pack.rs").read_text(encoding="utf-8")
assert "PACK_PARENT_MODE: u32 = 0o751" in spa_pack
assert "PACK_CURRENT_MODE: u32 = 0o755" in spa_pack
assert "SECRET_DIR_MODE: u32 = 0o700" in spa_pack
assert "launcher_product::ensure_shell_traverse" in Path("runtime/src/core.rs").read_text(
    encoding="utf-8"
)
assert "launcher_product::ensure_shell_traverse" in Path("runtime/src/launcher_spa.rs").read_text(
    encoding="utf-8"
)
assert "libc::chown" not in product
assert "file://" not in product
assert "https://app.eliteew.ru" not in product
assert "launcher_product::open" in core
assert "product.open" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "shell.product_surface" in Path("shell-ui/src/desktop/FoflineActivator.tsx").read_text(encoding="utf-8")
assert "fofline-spa" in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "PACK_ROOT" in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "shell.product_surface" in Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
assert "EMPIIEROS_FOFLINE_SPA_START" in Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
assert "token_ipc_forbidden" in Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
native_bridge = Path("runtime/shell-gui/native_bridge.c").read_text(encoding="utf-8")
assert "fofline product meet navigation kind=iframe" in native_bridge
assert "fofline product navigation denied code=" in native_bridge
assert "webkit_navigation_action_get_frame_name" in native_bridge
assert "blocked navigation outside trusted Shell origin" not in native_bridge
native_adv = native_bridge.replace("fofline product meet navigation kind=iframe", "")
assert "fofline product meet navigation kind=iframe" not in native_adv
chrome = Path("runtime/shell-gui/product_chrome.c").read_text(encoding="utf-8")
assert "Лаунчер" in chrome
assert "empiier_widget_show_deferred" in chrome
assert "empiieros-seat-panel" in chrome
assert "gnss_not_on_seat" in chrome
assert "seat.shield.status" in chrome
assert "audio.action not wired" in chrome
assert "always-on seat chrome" in chrome
assert "gtk_popover_set_modal" in chrome
assert "chrome_input_shape" in chrome
assert "movable no-peek" in chrome
assert "hud-chrome-pos.json" in chrome
assert "empiieros-hud-grip" in chrome
assert "CHROME_PIN_END" in chrome
assert "GTK_POS_LEFT" in chrome
assert "gtk_box_new(GTK_ORIENTATION_VERTICAL, 6)" in chrome
assert "chrome_peek_height" not in chrome
assert "chrome_apply_clip" not in chrome
assert "chrome_schedule_peek" not in chrome
assert "gtk_scrolled_window_new" not in chrome
assert "CHROME_PEEK_VISIBLE_DEN" not in chrome
assert "chrome_peek_margin" not in chrome
assert "chrome_pointer_inside" not in chrome
assert "hud-shield.png" in chrome
assert "hud-ota.png" in chrome
assert "HUD_FLY_OTA" in chrome
assert "hud_button_with_caption" in chrome
assert "PK приходит с Air" in chrome
assert "onOpenOta" in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(
    encoding="utf-8"
)
assert "onOpenOta" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert "host-ota.png" in Path("shell-ui/src/secure/components/TaskbarTray.tsx").read_text(
    encoding="utf-8"
)
assert "hud-volume.png" in chrome
assert "panel-mic.png" in chrome
assert "shield_chip" not in chrome
assert '"▾"' not in chrome
assert "GTK_STYLE_PROVIDER_PRIORITY_USER" in chrome
assert "margin-top: -48px" not in chrome
assert "schedule_collapse" not in chrome
assert "gtk_widget_set_margin_end" in chrome
assert "click-launcher.wav" in chrome
assert '"aplay"' in chrome
assert "chrome_hide_flyouts" in chrome
assert "chrome_toggle_flyout" in chrome
assert "Открыть лаунчер" in chrome
assert "Открыть EliteEW" in chrome
assert "product.open" in chrome
assert "seat chrome flyout kind=" in chrome
assert "HUD_FLY_VOLUME" in chrome
_play = chrome[chrome.find("static void play_click") : chrome.find("#define HUD_FLY_LAUNCHER")]
assert _play.find('"aplay"') < _play.find('"paplay"')
_vol = chrome[
    chrome.find("static void on_volume_clicked") : chrome.find(
        "static void on_shield_icon_clicked"
    )
]
assert "on_panel_clicked" not in _vol
assert "HUD_FLY_VOLUME" in _vol
_sh = chrome[
    chrome.find("static void on_shield_icon_clicked") : chrome.find(
        "static void on_observe_clicked"
    )
]
assert "open_host_workspace" not in _sh
assert "empiieros-host-workspace" in chrome
assert "empiieros-host-workspace" in Path("shell-ui/src/secure/SecureApp.tsx").read_text(
    encoding="utf-8"
)
assert "empiier_product_chrome_show(&bridge)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "gtk_overlay_add_overlay(GTK_OVERLAY(overlay), product_bar)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "gtk_widget_set_halign(product_bar, GTK_ALIGN_END)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "gtk_widget_set_halign(product_bar, GTK_ALIGN_CENTER)" not in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "gtk_widget_set_valign(product_bar, GTK_ALIGN_CENTER)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert Path("runtime/shell-gui/product-chrome/launcher.png").is_file()
assert Path("runtime/shell-gui/product-chrome/eliteew.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-panel.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-shield.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-shield-alert.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-ota.png").is_file()
assert Path("runtime/shell-gui/product-chrome/hud-volume.png").is_file()
assert Path("runtime/shell-gui/product-chrome/panel-mic.png").is_file()
assert Path("runtime/shell-gui/product-chrome/panel-cam.png").is_file()
assert Path("runtime/shell-gui/product-chrome/panel-gps.png").is_file()
assert Path("runtime/shell-gui/product-chrome/click-launcher.wav").is_file()
assert Path("runtime/shell-gui/product-chrome/click-eliteew.wav").is_file()

def _wav_peak(path: Path) -> int:
    import struct
    import wave

    with wave.open(str(path), "rb") as wav:
        raw = wav.readframes(wav.getnframes())
    samples = struct.unpack("<" + "h" * (len(raw) // 2), raw)
    return max(abs(sample) for sample in samples) if samples else 0

assert _wav_peak(Path("runtime/shell-gui/product-chrome/click-launcher.wav")) >= 8000
assert _wav_peak(Path("runtime/shell-gui/product-chrome/click-eliteew.wav")) >= 8000
assert Path("runtime/shell-gui/product-chrome/start_os.mp4").is_file()
assert Path("runtime/shell-gui/boot_splash.c").is_file()
assert "alsasink" in Path("runtime/shell-gui/boot_splash.c").read_text(encoding="utf-8")
assert "SPLASH_VOLUME 0.25" in Path("runtime/shell-gui/boot_splash.c").read_text(
    encoding="utf-8"
)
assert "empiier_boot_splash_attach" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "boot splash revealed hud" in Path("runtime/shell-gui/boot_splash.c").read_text(
    encoding="utf-8"
)
assert "empiier_product_chrome_show" in Path("runtime/shell-gui/boot_splash.c").read_text(
    encoding="utf-8"
)
assert "gtk_widget_show(splash->product_bar)" not in Path(
    "runtime/shell-gui/boot_splash.c"
).read_text(encoding="utf-8")
assert not Path("start_os.mp4").exists()
from hashlib import sha256 as _sha256

def _chrome_digest(name: str) -> str:
    return _sha256(
        Path("runtime/shell-gui/product-chrome").joinpath(name).read_bytes()
    ).hexdigest()

for _chrome_name in (
    "launcher.png",
    "eliteew.png",
    "hud-panel.png",
    "hud-shield.png",
    "hud-shield-alert.png",
    "hud-ota.png",
    "hud-volume.png",
    "panel-mic.png",
    "panel-cam.png",
    "panel-gps.png",
    "click-launcher.wav",
    "click-eliteew.wav",
):
    _pub = Path("shell-ui/public/product-chrome").joinpath(_chrome_name)
    assert _pub.is_file()
    assert _sha256(_pub.read_bytes()).hexdigest() == _chrome_digest(_chrome_name)
assert Path(".cursor/skills/comfyui-media/SKILL.md").is_file()
assert Path(".cursor/rules/comfyui-media.mdc").is_file()
assert "gtk_widget_set_no_show_all(widget, FALSE)" in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8")
assert "empiier_overlay_raise" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_overlay_park" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_widget_show_deferred" in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8")
assert "auth.eliteew.ru" in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert Path("runtime/shell-gui/auth_bounds.c").is_file()
assert Path("runtime/shell-gui/auth_bounds.h").is_file()
auth_bounds = Path("runtime/shell-gui/auth_bounds.c").read_text(encoding="utf-8")
assert "empiieros-auth-dimmer" in auth_bounds
assert "GTK_ALIGN_CENTER" in auth_bounds
assert "kind=card" in auth_bounds
assert "gtk_window_new" not in auth_bounds
assert "gtk_window_unfullscreen" not in auth_bounds
assert "kind=card" in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "runtime/shell-gui/auth_bounds.c" in Path(
    "scripts/package-shell-gui-rootfs.sh"
).read_text(encoding="utf-8")
auth_card_adv = Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8").replace(
    "kind=card", ""
)
assert "kind=card" not in auth_card_adv
assert "https://auth.eliteew.ru:443/" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert 'strstr(uri, ":443") == NULL' not in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "EmperyDesktopNativeOidc" in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "EmperyDesktopMeetStage" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "EmperyDesktopSeatBrowser" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "openSeatBrowser" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiierosSeatBrowser" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "token_forbidden" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_seat_browser_register_product_bridge" in Path(
    "runtime/shell-gui/seat_browser_surface.c"
).read_text(encoding="utf-8")
assert "fofline seat browser host request kind=open" in Path(
    "runtime/shell-gui/seat_browser_surface.c"
).read_text(encoding="utf-8")
assert "empiierosMeet" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "closeMeetStage" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "__empiierosMeetWait" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_meet_json_id" in Path("runtime/shell-gui/meet_bounds.c").read_text(
    encoding="utf-8"
)
assert "code=video_call_end" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
assert "empiier_fofline_video_call_end_path" in Path(
    "runtime/shell-gui/fofline_api_proxy.c"
).read_text(encoding="utf-8")
assert "setMeetStageBounds" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "meet_stage_bounds_required" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "join_url_denied" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "https://meet.eliteew.ru" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "charAt(23)" in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "#define MEET_HOST_LEN 15" in Path("runtime/shell-gui/meet_surface.c").read_text(
    encoding="utf-8"
)
assert "_Static_assert(sizeof(MEET_HOST) - 1 == MEET_HOST_LEN" in Path(
    "runtime/shell-gui/meet_surface.c"
).read_text(encoding="utf-8")
assert Path("runtime/shell-gui/meet_surface.c").is_file()
assert Path("runtime/shell-gui/meet_bounds.c").is_file()
meet_surface = Path("runtime/shell-gui/meet_surface.c").read_text(encoding="utf-8")
meet_bounds = Path("runtime/shell-gui/meet_bounds.c").read_text(encoding="utf-8")
assert "meet_hostmap_unavailable" in meet_surface
assert "fofline meet stage denied code=" in meet_surface
assert "fofline meet stage request kind=" in meet_surface
assert "meet_stage_bounds_required" in meet_surface
assert "meet_stage_bounds_invalid" in meet_bounds
assert "enter-fullscreen" in meet_surface
assert "fofline meet stage fullscreen denied" in meet_surface
secure_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "gtk_widget_set_halign(meet_box, GTK_ALIGN_FILL)" not in secure_main
assert "gtk_widget_set_halign(meet_box, GTK_ALIGN_START)" in secure_main
assert "jwt[n] != '#'" in meet_surface
assert "WEBKIT_IS_USER_MEDIA_PERMISSION_REQUEST" in meet_surface
assert "fofline product media permission allowed" in meet_surface
assert "empiier_native_bridge_meet_host_ok" in meet_surface
assert "web-process-terminated" in meet_surface
assert "fofline meet stage crashed reason=" in meet_surface
assert "empiieros-meet-stage-lost" in meet_surface
assert "meet_process_crashed" in meet_surface
assert "empiierosMeetDiag" in meet_surface
assert "dummyTracks" not in meet_surface
assert "ensureTracks" not in meet_surface
assert "addTrack" not in meet_surface
assert "webkit_web_view_terminate_web_process" in meet_surface
assert "fofline meet stage process recycled" in meet_surface
assert "addTransceiver" in meet_surface
assert "direction:'sendrecv'" in meet_surface
assert "lowerCodecs" in meet_surface
assert "wrapGet" in meet_surface
assert "localDescription" in meet_surface
assert "fofline meet new-window ignored" in meet_surface
assert "empiier_native_bridge_conceal_meet" in meet_surface
assert "empiier_overlay_park" in meet_surface
assert "empiier_overlay_raise" in meet_surface
assert "gtk_overlay_set_overlay_pass_through" in secure_main
assert "gdk_window_input_shape_combine_region" in Path(
    "runtime/shell-gui/native_bridge.c"
).read_text(encoding="utf-8")
meet_bounds_src = Path("runtime/shell-gui/meet_bounds.c").read_text(encoding="utf-8")
assert "empiier_native_bridge_conceal_meet" in meet_bounds_src
assert "empiier_native_bridge_hide_meet" not in meet_bounds_src
assert "video-calls" not in meet_surface
assert "hda-verb" not in meet_surface
meet_adv = meet_surface.replace("fofline meet stage crashed reason=", "")
assert "fofline meet stage crashed reason=" not in meet_adv
recycle_adv = meet_surface.replace("fofline meet stage process recycled", "")
assert "fofline meet stage process recycled" not in recycle_adv
bounds_adv = meet_surface.replace("meet_stage_bounds_required", "")
assert "meet_stage_bounds_required" not in bounds_adv
assert "launcher.oidc_start" in core
assert "launcher.oidc_logout" in core
assert "oidc_session_required" in Path("runtime/src/launcher_oidc.rs").read_text(encoding="utf-8")
assert "launcher_oidc::ensure_fresh_session" in Path("runtime/src/launcher_product.rs").read_text(
    encoding="utf-8"
)
assert "oidc_token" in Path("runtime/src/launcher_spa.rs").read_text(encoding="utf-8")
assert "oidc_par" in Path("runtime/src/launcher_spa.rs").read_text(encoding="utf-8")
assert "oidc_refresh" in Path("runtime/src/launcher_spa.rs").read_text(encoding="utf-8")
assert 'grant_type", "refresh_token"' in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "BAKED_PAR" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "eliteew_ticket" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "fn auth_code_ok" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
secure_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "static void finish_scheme_ok" in secure_main
assert "Access-Control-Allow-Origin" in secure_main
assert "webkit_uri_scheme_request_finish_with_response" in secure_main
fofline_fn = secure_main.split("static void serve_fofline_pack", 1)[1].split(
    "static void enable_webkit_sandbox", 1
)[0]
assert "finish_scheme_bytes" in fofline_fn
assert "static gboolean mime_is_stylesheet" in secure_main
assert "static gboolean scheme_request_is_cors" in secure_main
assert "static gboolean path_is_pack_module_css" in secure_main
assert "fofline boot" in secure_main
assert "Vite __vitePreload inserts" in secure_main
assert "mode=simple cors=%d pack=%d" in secure_main
assert 'register_uri_scheme_as_local(security_manager, "fofline-spa")' not in secure_main
assert 'register_uri_scheme_as_local(security_manager, "empiier")' in secure_main
assert "g_timeout_add(15000U, run_fofline_boot_probe" in secure_main
assert "g_timeout_add(45000U, run_fofline_boot_probe" in secure_main
assert "tx:tx.slice(0,72)" in secure_main
assert "scheme css finish" in secure_main
assert "__empiierBootErr" in secure_main
assert "__empiierBootSrc" in secure_main
assert "es:String(window.__empiierBootSrc||'')" in secure_main
assert "uri_is_fofline_edge" in secure_main
assert "fofline edge start" in secure_main
assert "fofline edge failed" in secure_main
assert "copy_json_reason" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
assert "reason=%s" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
assert "eliteew.foflineClient.v1" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "https://app.eliteew.ru'+u" not in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "u.slice(app.length)" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "empiier_fofline_api_proxy" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "fofline api proxy" in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
proxy = Path("runtime/shell-gui/fofline_api_proxy.c").read_text(encoding="utf-8")
assert "path_is_content_download" in proxy
assert "content=octet-stream" in proxy
assert "sniff_audio_type" in proxy
assert "media_range" in proxy
assert "api_proxy_partial_content" in proxy
assert "soup_message_headers_remove(headers, \"Range\")" in proxy
assert "Access-Control-Allow-Origin" in proxy
assert "Access-Control-Allow-Credentials" in proxy
assert "fofline-spa://localhost" in proxy
assert "__empiierosAudioBlob" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "__empiierosNeedLoad" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "MAX_CONTENT_RESPONSE" in proxy
assert proxy.index("webkit_uri_scheme_response_set_http_headers") < proxy.index(
    "webkit_uri_scheme_response_set_content_type"
)
assert '"Origin"' in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(
    encoding="utf-8"
)
assert (
    'soup_message_headers_replace(headers, "X-SSO-Replace-Confirm", "1")'
    in Path("runtime/shell-gui/fofline_api_proxy.c").read_text(encoding="utf-8")
)
assert (
    "eliteew.auth.sessionReplaceConfirmedUntil.v1"
    in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
)
assert "/api/v1/auth/sso/callback" not in Path(
    "runtime/src/launcher_oidc.rs"
).read_text(encoding="utf-8")
assert "XMLHttpRequest.prototype.open" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "function absApi" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "decodeURIComponent(escape(bin))" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "eliteew.keycloak.foflineNative.oidcTokens.v1" in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "refreshToken:bootstrap.token" in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "function hostSsoRefresh" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "/api/v1/auth/sso/refresh" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "fofline-oidc-bridge" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert "eliteew.mobileEmbeddedClient.v1" not in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "empiier_oidc_clear_product" in Path("runtime/shell-gui/native_bridge.c").read_text(
    encoding="utf-8"
)
assert "webkit_uri_scheme_request_get_http_headers" in secure_main
assert 'g_strcmp0(mode, "cors") == 0' in secure_main
auth_surface = Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "b.token||b.accessToken" in auth_surface
assert "hostOidc('refresh')" in auth_surface
assert "empiierosOidc" in auth_surface
assert r'\"force\":true' in auth_surface
assert "fn refresh_now" in Path("runtime/src/launcher_oidc.rs").read_text(
    encoding="utf-8"
)
assert (
    'refreshSessionJson:function(){return Promise.resolve(JSON.stringify({ok:false'
    not in auth_surface
)
assert "refreshSessionJson:function(){return hostOidc('refresh');}" in auth_surface
assert "matches!(item, '-' | '_' | '.')" in Path("runtime/src/oidc_urls.rs").read_text(
    encoding="utf-8"
)
assert "choose_dpop_iat_from_http_date" in Path("runtime/src/oidc_dpop_iat.rs").read_text(
    encoding="utf-8"
)
assert "issuer_http_date" in Path("runtime/src/https_auth.rs").read_text(encoding="utf-8")
assert "dpop_iat" in Path("runtime/src/launcher_oidc.rs").read_text(encoding="utf-8")
assert "oidc_ticket" in Path("runtime/src/launcher_oidc.rs").read_text(encoding="utf-8")
assert "signing.sk.hex" in Path("runtime/src/oidc_ticket.rs").read_text(encoding="utf-8")
assert "CHALLENGE_PATH" in Path("runtime/src/https_vip.rs").read_text(encoding="utf-8")
assert "192.168.8.10" not in Path("shell-ui/src/secure/components/NetworkPanel.tsx").read_text(
    encoding="utf-8"
)
assert "192.168.8.9" not in Path("shell-ui/src/secure/components/NetworkPanel.tsx").read_text(
    encoding="utf-8"
)
_shell_sources = [
    p
    for p in Path("shell-ui/src").rglob("*")
    if p.suffix in {".ts", ".tsx"} and not p.name.endswith(".check.ts")
]
assert Path("shell-ui/src/secure/components/NetworkPanel.tsx") in _shell_sources
for _src in _shell_sources:
    _text = _src.read_text(encoding="utf-8")
    assert "192.168.8.10" not in _text, _src
    assert "192.168.8.9" not in _text, _src
assert "SeatShield" not in Path("shell-ui/src/desktop/foflineCatalog.ts").read_text(encoding="utf-8")
assert '"shield"' in Path("shell-ui/src/secure/components/Taskbar.tsx").read_text(encoding="utf-8")
assert "SeatShieldCard" in Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8")
assert "SeatBrowserCard" in Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8")
assert Path("shell-ui/src/secure/components/SeatOtaSection.tsx").is_file()
assert "launcher.ota_status" in Path("shell-ui/src/secure/components/SeatOtaSection.tsx").read_text(
    encoding="utf-8"
)
assert "APPLY_OTA" in Path("shell-ui/src/session/seatOta.ts").read_text(encoding="utf-8")
assert "SeatOtaSection" not in Path("shell-ui/src/secure/components/SystemPanel.tsx").read_text(
    encoding="utf-8"
)
_ota_panel = Path("shell-ui/src/secure/components/OtaPanel.tsx").read_text(encoding="utf-8")
assert "SeatOtaSection" in _ota_panel
assert "HostAppShell" not in _ota_panel
assert "hostAppTabOta" not in _ota_panel
_ota_section = Path("shell-ui/src/secure/components/SeatOtaSection.tsx").read_text(
    encoding="utf-8"
)
assert "HostAppFact" in _ota_section
assert "otaPrereq" in _ota_section
assert "otaLead" not in _ota_section
assert "otaApplyPlantNote" not in _ota_section
assert "OtaPanel" in Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8")
assert '"ota"' in Path("shell-ui/src/desktop/foflineHostModules.ts").read_text(encoding="utf-8")
assert "otaTitle" in Path("shell-ui/src/secure/locale/en.ts").read_text(encoding="utf-8")
assert "otaTitle" in Path("shell-ui/src/secure/locale/ru.ts").read_text(encoding="utf-8")
assert "otaPrereq" in Path("shell-ui/src/secure/locale/en.ts").read_text(encoding="utf-8")
assert "otaLead" not in Path("shell-ui/src/secure/locale/en.ts").read_text(encoding="utf-8")
assert "otaLead" not in Path("shell-ui/src/secure/locale/ru.ts").read_text(encoding="utf-8")
assert "foflineActivatorLabSpaNote" in Path("shell-ui/src/secure/locale/en.ts").read_text(
    encoding="utf-8"
)
assert "foflineActivatorLabSpaNote" in Path(
    "shell-ui/src/desktop/foflineLauncherChrome.tsx"
).read_text(encoding="utf-8")
# Path-stage spa CTA still refuses re-sync when ready; Lab may re-fetch.
assert 'status?.spa_state === "ready"' in Path(
    "shell-ui/src/desktop/FoflineActivator.tsx"
).read_text(encoding="utf-8")
assert Path("shell-ui/src/secure/design/HostAppShell.tsx").is_file()
host_shell = Path("shell-ui/src/secure/design/HostAppShell.tsx").read_text(encoding="utf-8")
assert "export function HostAppShell" in host_shell
assert "host-app-root" in host_shell
assert "host-app-tiles" in host_shell
assert "tabs.length > 1" in host_shell
_shield_card = Path("shell-ui/src/secure/components/SeatShieldCard.tsx").read_text(
    encoding="utf-8"
)
assert "HostAppShell" in _shield_card
assert "SectionCard" not in _shield_card
assert "seatShieldOverviewLead" not in _shield_card
assert "host-app-tiles" in Path("shell-ui/src/secure/secure.css").read_text(encoding="utf-8")
secure_css = Path("shell-ui/src/secure/secure.css").read_text(encoding="utf-8")
assert ".host-app-root" in secure_css
assert ".seat-shield-root,\n.host-app-root" in secure_css or ".seat-shield-root,\r\n.host-app-root" in secure_css or (
    ".seat-shield-root," in secure_css and ".host-app-root" in secure_css
)
assert "host-app-window-body" in Path("shell-ui/src/secure/components/WorkspaceFrame.tsx").read_text(
    encoding="utf-8"
)
assert 'id: "browser"' in Path("shell-ui/src/desktop/foflineHostModules.ts").read_text(
    encoding="utf-8"
)
assert "Seat-local file is SoT" in Path("runtime/seat_browser_core/src/service.rs").read_text(
    encoding="utf-8"
)
browser_adv = Path("shell-ui/src/secure/SecureApp.tsx").read_text(encoding="utf-8").replace(
    "SeatBrowserCard", ""
)
assert "SeatBrowserCard" not in browser_adv
secure_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "webkit_settings_set_enable_webrtc(browser_settings" in secure_main
assert "webkit_settings_set_enable_media_stream(browser_settings, TRUE)" in secure_main
seat_browser_src = Path("runtime/shell-gui/seat_browser_surface.c").read_text(encoding="utf-8")
# Tabs attach owns the WebViews. Permission SoT is the overlay, not secure_main.
assert "empiier_native_bridge_seat_browser_permission" in seat_browser_src
assert "empiier_native_bridge_seat_browser_permission" in Path(
    "runtime/shell-gui/native_bridge.h"
).read_text(encoding="utf-8")
assert "empiier_native_bridge_seat_browser_permission" not in secure_main
assert "seat browser media permission allowed" in seat_browser_src
assert "empiier_seat_browser_chrome_build" in seat_browser_src
assert "empiier_native_bridge_seat_browser_command" in seat_browser_src
assert '"host_action": "reload"' in Path("runtime/seat_browser_core/src/service.rs").read_text(
    encoding="utf-8"
)
browser_svc = Path("runtime/seat_browser_core/src/service.rs").read_text(encoding="utf-8")
assert "reload/history/media host ops not wired yet" not in browser_svc
assert "media host ops not wired yet" not in browser_svc
assert "fn op_media_set" in browser_svc
assert '"seat.browser.download.begin"' in browser_svc
assert '"seat.browser.tab.open"' in browser_svc
assert "seat.browser.reload" in Path(
    "shell-ui/src/secure/components/SeatBrowserCard.tsx"
).read_text(encoding="utf-8")
webrtc_adv = secure_main.replace("webkit_settings_set_enable_webrtc(browser_settings", "")
assert "webkit_settings_set_enable_webrtc(browser_settings" not in webrtc_adv
assert "build_par_authorization_url" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "empery-fofline" in Path("runtime/src/oidc_urls.rs").read_text(encoding="utf-8")
assert "request_uri=urn%3Aietf%3Aparams%3Aoauth%3Arequest_uri%3A" in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "oidc auth back-to-app ignored origin=app.eliteew.ru" in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "oidc auth form blocked reason=empty_hidden_or_incomplete" in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "Мы сожалеем" in Path("runtime/shell-gui/auth_surface.c").read_text(encoding="utf-8")
assert "oidc auth rebuild start" in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert 'strstr(response, "refresh")' not in Path("runtime/shell-gui/auth_surface.c").read_text(
    encoding="utf-8"
)
assert 'strstr(response, "\\"refresh_token\\"")' in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert 'strstr(response, "\\"access_token\\"")' in Path(
    "runtime/shell-gui/auth_surface.c"
).read_text(encoding="utf-8")
assert "refresh token stayed" not in Path("runtime/src/launcher_oidc.rs").read_text(
    encoding="utf-8"
)
assert "WEBKIT_COOKIE_POLICY_ACCEPT_ALWAYS" in Path("runtime/shell-gui/secure_main.c").read_text(
    encoding="utf-8"
)
assert "submit-form" in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "Отмена" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "webkit_web_context_set_sandbox_enabled(auth_context, FALSE)" in Path(
    "runtime/shell-gui/secure_main.c"
).read_text(encoding="utf-8")
assert "on_auth_load_failed" in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
packager = Path("scripts/package-shell-gui-rootfs.sh").read_text(encoding="utf-8")
assert "--bin empiieros-portal-up" in packager
assert "--bin empiieros-spa-fetch" in packager
assert "--bin empiieros-ota-fetch" in packager
assert "usr/local/sbin/empiieros-ota-fetch" in packager
assert "empiieros-ota-fetch.service" in packager
assert "runtime/src/ota_cutover.rs" in packager
assert "ota-cutover.conf" in packager
assert "ota_artifact_pk" in packager
assert "--on-active=3s" in packager
assert "ota_reload_shell" in packager
assert "usr/local/sbin/empiieros-wg" in packager
assert "WITH_WGQUICK=no" in packager
assert "launcher.edge_health" in packager
assert "first_boot_pin.json" in pin
assert "DEFAULT_TECH_PIN" not in pin
assert "123456" not in pin
assert Path("session/FIRST-BOOT-PIN.md").is_file()
assert Path("session/SEAT-LAUNCHER.md").is_file()
assert "LAUNCHER_JOURNEY" in Path("session/SEAT-LAUNCHER.md").read_text(encoding="utf-8")
assert "autoOpened" in Path("session/SEAT-LAUNCHER.md").read_text(encoding="utf-8")
assert "SEAT-LAUNCHER.md" in Path("docs/foundation/PROJECT-TREE.md").read_text(
    encoding="utf-8"
)
face = Path("session/SEAT-FACE.md").read_text(encoding="utf-8")
assert "empiier://shell/index.html" in face
assert "movable" in face
assert "floating panels" in face
assert "Six icons" in face
assert "licensed tray" in face
assert "LAUNCHER_JOURNEY" in face
assert "SEAT-FACE.md" in Path("docs/foundation/PROJECT-TREE.md").read_text(
    encoding="utf-8"
)
gui = Path("session/SHELL-GUI.md").read_text(encoding="utf-8")
assert "secure_main.c" in gui
assert "empiier://shell/index.html" in gui
assert "GET /v0/substrate/shell-gui/status" not in gui
audio = Path("session/AUDIO-HARDWARE.md").read_text(encoding="utf-8")
assert "audio.control" in audio
assert "pipewire-pulse" not in audio
assert "set_master_volume" not in audio
assert "CARGO_TARGET_DIR" in packager
assert "--bin empiieros-air-enroll" in packager
assert "--bin seat-shield-fw" in packager
assert "--bin seat-shield-watch" in packager
assert "--bin seat-shield-probe" in packager
assert "runtime/seat_shield_probe/Cargo.toml" in packager
assert "relocation-model=pic" in packager
assert "link-arg=-Wl,-z,relro,-z,now" in packager
assert "link-arg=-Wl,-z,noexecstack" in packager
assert "x86_64-unknown-linux-musl" not in packager
assert "usr/libexec/empiieros-seat-shield-fw" in packager
assert "usr/libexec/empiieros-seat-shield-watch" in packager
assert "usr/libexec/empiieros-seat-shield-probe" in packager
assert "empiieros-seat-shield-nftables.service" in packager
assert "multi-user.target.wants/empiieros-seat-shield-nftables.service" in packager
assert "runtime/helpers/seat-shield/seat_shield.nft" in packager
assert "runtime/src/seat_shield.rs" in packager
assert "launcher.pin_issue" in packager
assert "empiieros-first-boot-pin-v1|" in packager
assert "runtime/src/launcher_pin.rs" in packager
assert 'bash "$SUBSTRATE_ROOT/scripts/buildroot-build.sh" --resume' in packager
network_control = Path("runtime/src/network_control.rs").read_text(encoding="utf-8")
assert "fn json_ipv4_address" in network_control
assert "networkd_verification_accepts_systemd_octet_addresses" in network_control
assert "fn exclude_name_from_wireless_match" in network_control
assert "fn claim_filename" in network_control
assert "fn claim_plan" in network_control
assert "20-wireless.network" in network_control
wifi_control = Path("runtime/src/wifi_control.rs").read_text(encoding="utf-8")
assert "scan_ssid=1" in wifi_control
assert "reset-failed" in wifi_control
assert "associated but IPv4 address not observed" in wifi_control
assert "kick_dhcp" in wifi_control
secure_main = Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "shell scroll observed" not in secure_main
assert "window.scrollBy({" not in secure_main
assert "on_window_scroll_event" not in secure_main
assert "on_scroll_event" not in secure_main
assert 'g_signal_connect(view_widget, "scroll-event"' not in secure_main
assert "GDK_SCROLL_MASK" not in secure_main
assert "Do not connect scroll-event" in secure_main
assert "installViewportLockedWheelScroll" in Path(
    "shell-ui/src/secure/SecureApp.tsx"
).read_text(encoding="utf-8")
assert "elementsFromPoint" in Path(
    "shell-ui/src/secure/wheelScrollTarget.ts"
).read_text(encoding="utf-8")
assert "webkit_settings_set_enable_media_stream(settings, TRUE)" in secure_main
assert "delete-event" in secure_main
assert "GDK_KEY_F4" in secure_main
assert "kiosk close ignored (Alt+F4)" in secure_main
assert "kiosk close ignored (delete-event)" in secure_main
kiosk = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
    "/config/buildroot/external/board/eliteew-ts/rootfs_overlay"
    "/usr/local/bin/eliteew-ts-kiosk"
).read_text(encoding="utf-8")
assert 'command="true"' in kiosk
assert "cooling 15s then retry" in kiosk
assert "EmpiierOS face must not die on Alt+F4" in kiosk
assert "LABWC_UPDATE_ACTIVATION_ENV=0" in kiosk
assert "NO_AT_BRIDGE=1" in kiosk
assert "AT_SPI_BUS=unix:path=/tmp/empiieros-no-at-spi" in kiosk
assert "kiosk-session-bus.conf" in kiosk
assert "--config-file=" in kiosk
assert "<standard_session_servicedirs />" not in kiosk
assert 'pkill -f ${bin}' in kiosk
payload_kiosk = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
    "/payload-templates/usr-local-bin/eliteew-ts-kiosk"
).read_text(encoding="utf-8")
assert payload_kiosk == kiosk
unit = Path(os.environ["ELITEEW_TS_ROOT"] + "/config/buildroot/external/board/eliteew-ts/rootfs_overlay/etc/systemd/system/empiieros-runtime.service").read_text(encoding="utf-8")
assert "BindReadOnlyPaths=/proc/1/mounts:/run/empiieros/host-mounts" in unit
assert "CAP_SYS_PTRACE" not in unit
assert "StateDirectoryMode=0751" in unit
assert "StateDirectoryMode=0700" not in unit
# LinkornEW is a separate face. The FoFline image packager and entry stay on shell-ui.
_linkorn = Path("shell-linkorn/src/LinkornDesktop.tsx").read_text(encoding="utf-8")
assert "LinkornEW" in Path("shell-linkorn/src/locale.ts").read_text(encoding="utf-8")
assert "FoflineActivator" not in _linkorn
assert "launcher.air_activate" not in _linkorn
assert "product.open" not in _linkorn
assert "LinkornEW" not in Path("shell-ui/src/main.tsx").read_text(encoding="utf-8")
assert "shell-linkorn" not in Path("shell-ui/src/main.tsx").read_text(encoding="utf-8")
assert "shell-linkorn" not in Path("scripts/package-shell-gui-rootfs.sh").read_text(
    encoding="utf-8"
)
_linkorn_md = Path("session/LINKORNEW.md").read_text(encoding="utf-8")
assert "linkornew" in _linkorn_md
assert "Latitude 5511" in _linkorn_md
assert "192.168.1.99/24" in _linkorn_md
assert "i915" in _linkorn_md
assert "libnss3" in _linkorn_md
assert "libcups" in _linkorn_md
assert Path("runtime/linkornew-seat/CHROMIUM.md").read_text(encoding="utf-8").count("libnss3") >= 1
assert "libcups.so.2" in Path("runtime/linkornew-seat/CHROMIUM.md").read_text(encoding="utf-8")
assert "linkornew" in Path("docs/foundation/SUBSTRATE-KINSHIP.md").read_text(encoding="utf-8")
_linkorn_rs = Path("runtime/src/linkorn.rs").read_text(encoding="utf-8")
assert "linkorn.console" in _linkorn_rs
assert "Command::new" not in _linkorn_rs
assert "linkornew is a separate runtime profile" in Path("runtime/src/core.rs").read_text(
    encoding="utf-8"
)
assert "linkorn_ops_stay_unregistered_on_the_fofline_runtime" in Path(
    "runtime/src/core.rs"
).read_text(encoding="utf-8")
assert "host_argv_allowed" in _linkorn_rs
assert "Command::new" not in _linkorn_rs
_host = Path("runtime/src/bin/linkornew_host.rs").read_text(encoding="utf-8")
assert "host_plan" in _host
assert "/var/lib/empiieros-runtime/linkorn/chrome/host.last" in _host
assert "sh -c" not in _host
assert "/bin/sh" not in _host
_cargo = Path("runtime/Cargo.toml").read_text(encoding="utf-8")
assert 'name = "linkornew-host"' in _cargo
assert 'required-features = ["linkornew"]' in _cargo
_packager = Path("scripts/package-shell-gui-rootfs.sh").read_text(encoding="utf-8")
assert "linkornew-host" not in _packager
assert "--features linkornew" not in _packager
assert "linkornew-seat" not in _packager
_seat_c = Path("runtime/linkornew-seat/linkornew-seat.c").read_text(encoding="utf-8")
assert 'g_strcmp0(action, "console")' in _seat_c
assert 'g_strcmp0(action, "browser")' in _seat_c
assert 'g_strcmp0(action, "command-post")' in _seat_c
assert "/usr/lib/linkornew/linkornew-host" in _seat_c
assert "linkornew://launch/" in _seat_c
assert "gtk_window_fullscreen" in _seat_c
assert "/bin/sh" not in _seat_c
assert "system(" not in _seat_c
_desk_js = Path("runtime/linkornew-seat/desk/desk.js").read_text(encoding="utf-8")
assert "keydown" in _desk_js
assert "linkornew://launch/" in _desk_js
assert 'notice.textContent = "start "' in _desk_js
assert "product.open" not in _desk_js
assert "charset=utf-8" in _seat_c
assert "/dev/ttyS0" not in _seat_c
assert "host running" in _seat_c
assert "waiting for program" in _seat_c
assert "/var/lib/empiieros-runtime/linkorn/chrome/host.last" in _seat_c
assert "LinkornEW" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
assert "linkornew-host" not in Path("runtime/shell-gui/secure_main.c").read_text(encoding="utf-8")
_desk = Path("shell-linkorn/src/LinkornDesktop.tsx").read_text(encoding="utf-8")
assert "requestLinkorn" in _desk
assert "product.open" not in _desk
assert "FoflineActivator" not in _desk
_session = Path("runtime/linkornew-seat/linkornew-session").read_text(encoding="utf-8")
assert "/dev/tty1" in _session
assert "refusing" in _session
assert "LABWC_UPDATE_ACTIVATION_ENV=0" in _session
assert "LIBSEAT_BACKEND=seatd" in _session
assert "WLR_NO_HARDWARE_CURSORS=1" in _session
assert "WLR_RENDERER=pixman" in _session
assert "sys_vendor" in _session
_bp = Path("runtime/linkornew-seat/bash_profile").read_text(encoding="utf-8")
assert "linkornew-session" in _bp
assert "empiieros-shell-gui" not in _bp
assert "eliteew-ts-kiosk" not in _bp
assert "empiieros-shell-gui" not in _session
assert "eliteew-ts-kiosk" not in _session
assert "agetty" not in _session
assert "unmask" not in _session
_autostart = Path("runtime/linkornew-seat/config/labwc/autostart").read_text(encoding="utf-8")
assert "linkornew-seat" in _autostart
assert "empiieros-shell-gui" not in _autostart
for _tty in range(1, 7):
    _mask = Path(f"runtime/linkornew-seat/systemd/getty@tty{_tty}.service")
    assert _mask.is_symlink() and _mask.readlink() == Path("/dev/null")
assert "Conflicts=getty@tty1.service" in Path(
    "runtime/linkornew-seat/systemd/linkornew-session.service"
).read_text(encoding="utf-8")
assert "200/CHDIR" not in Path(
    "runtime/linkornew-seat/systemd/linkornew-session.service"
).read_text(encoding="utf-8")
assert "WorkingDirectory=/" in Path(
    "runtime/linkornew-seat/systemd/linkornew-session.service"
).read_text(encoding="utf-8")
assert "linkornew-session" not in _packager
_console_c = Path("runtime/linkornew-seat/console.c").read_text(encoding="utf-8")
assert "--title=LinkornEW" in _console_c
assert "agetty" not in _console_c
assert "system(" not in _console_c
assert "/usr/bin/foot" not in _linkorn_rs
assert "console.c" not in _packager
_plant = Path("runtime/linkornew-seat/plant-chromium.sh").read_text(encoding="utf-8")
_pin = "98307979de788ecf6d728038865b3947e2bd89bcacea78128dd17a2d394736c8"
assert _pin in _plant
assert "about_credits.html" in Path("runtime/linkornew-seat/CHROMIUM.md").read_text(encoding="utf-8")
assert "refusing install: chromium license dump is incomplete" in _plant
assert "about_credits.html" in _plant
assert "credits_html" in _plant
assert "lab-overlay" in _plant
assert "GDK_BACKEND=wayland" in _plant
assert "dns-over-https-mode=off" in _plant
assert "chrome.bin" in _plant
assert "chrome_100_percent.pak" in _plant
assert "icudtl.dat" in _plant
assert "copy_host_glibc_bubble" not in _plant
assert "/lib64/libc.so.6" not in _plant
assert "EliteEW-x86_64.AppImage" not in _plant
assert "refusing official out/" in _plant
assert "station-chromium" not in _packager
assert "plant-chromium.sh" not in _packager
_cp = Path("runtime/linkornew-seat/command-post")
for _name in ("cp-controller.service", "cp-sentinel.service", "cp-command-deck.service"):
    _text = (_cp / _name).read_text(encoding="utf-8")
    assert "0.0.0.0" not in _text
    assert "TOKEN" not in _text
    assert "install.sh" not in _text
    assert "seat_shield.nft" not in _text
_deck = (_cp / "cp-command-deck.service").read_text(encoding="utf-8")
assert "http://127.0.0.1:4191" in _deck
assert "cp-deck-stub" in _deck
assert "INADDR_LOOPBACK" in (_cp / "cp-deck-stub.c").read_text(encoding="utf-8")
assert "0.0.0.0" not in (_cp / "cp-deck-stub.c").read_text(encoding="utf-8")
assert "cp-command-deck" not in _packager
assert "cp-deck-stub" not in _packager
_install = Path("runtime/linkornew-seat/install-linkornew.sh").read_text(encoding="utf-8")
assert "resolved-linkornew.conf" in _install
assert "10-linkornew-wired.network" in _install
assert "192.168.1.99/24" in Path("runtime/linkornew-seat/systemd/10-linkornew-wired.network").read_text(
    encoding="utf-8"
)
assert "Gateway=192.168.1.1" in Path("runtime/linkornew-seat/systemd/10-linkornew-wired.network").read_text(
    encoding="utf-8"
)
assert "DNS=192.168.1.1" in Path("runtime/linkornew-seat/systemd/10-linkornew-wired.network").read_text(
    encoding="utf-8"
)
assert "DNS=192.168.1.1" in Path("runtime/linkornew-seat/systemd/resolved-linkornew.conf").read_text(
    encoding="utf-8"
)
assert "192.168.8.10" not in Path("runtime/linkornew-seat/systemd/10-linkornew-wired.network").read_text(
    encoding="utf-8"
)
assert "8.8.8.8" not in _install
assert "8.8.8.8" not in Path("runtime/linkornew-seat/systemd/resolved-linkornew.conf").read_text(
    encoding="utf-8"
)
assert "1.1.1.1" not in Path("runtime/linkornew-seat/systemd/resolved-linkornew.conf").read_text(
    encoding="utf-8"
)
assert "seat-profile" in _install
assert "linkornew" in _install
assert "wipefs" not in _install
assert "EWAE1" not in _install
assert "eliteew-ts-install-apply" not in _install
assert ".ecap" not in _install
assert "bash_profile" in _install
assert "linkornew-session.service" in _install
assert "getty@tty${n}.service" in _install
assert "linkornew-host" in _install
assert "cp-deck-stub" in _install
assert "install-linkornew.sh" not in _packager
assert "linkornew" not in Path("scripts/empiieros-installer-media-plant-installer-gui.sh").read_text(
    encoding="utf-8"
)
_link_pack = Path("scripts/package-linkornew.sh").read_text(encoding="utf-8")
assert "refusing dual profile" in _link_pack
assert "package-shell-gui-rootfs.sh" not in _link_pack
assert "--profile linkornew" in _link_pack
assert "linkornew" not in Path("scripts/package-shell-gui-rootfs.sh").read_text(encoding="utf-8")
_assemble = Path("runtime/linkornew-seat/assemble-linkornew-rootfs.sh").read_text(encoding="utf-8")
assert "machine-id" in _assemble
assert "install-linkornew.sh" in _assemble
assert "dbus helper is not setuid" in _assemble
assert "--from-tar" in _assemble
assert "--overlay" in _assemble
assert "plant-chromium.sh" in _assemble
assert "lab-overlay" in _assemble
assert "SUDO_USER" in _assemble
assert "guest-libs-prefix" in _assemble
assert "libnss3.so" in _assemble
assert "libcups.so.2" in _assemble
_guest_libs = Path("runtime/linkornew-seat/build-guest-nss-cups.sh").read_text(encoding="utf-8")
assert "ftp.mozilla.org" in _guest_libs
assert "OpenPrinting" in _guest_libs
assert "nss-3.101-with-nspr-4.35" in _guest_libs
assert "--with-dnssd=no" in _guest_libs
assert "readlink -f" in _guest_libs
assert "refusing host blob" in _guest_libs
assert "/usr/lib64" in _guest_libs
assert "GLIBC" in _guest_libs
assert "MPL" in Path("runtime/linkornew-seat/CHROMIUM.md").read_text(encoding="utf-8")
assert "chown -R" not in _assemble
assert "chmod 0755 \"$root\"" in _assemble
assert "tar -C \"$stage\" -cf - ." not in _assemble
_flash = Path("scripts/flash-linkornew-usb.sh").read_text(encoding="utf-8")
assert "refusing nvme" in _flash
assert "dbus helper is not setuid" in _flash
assert "--numeric-owner" in _flash
assert "-L LINKORNEW" in _flash
assert "root=PARTUUID=" in _flash
assert "root=LABEL=" not in _flash
assert ",256MiB,U," in _flash
assert ",U,LINKEFI" not in _flash
assert "package-shell-gui-rootfs.sh" not in _flash
_qemu = Path("scripts/qemu-linkornew.sh").read_text(encoding="utf-8")
assert "refusing to overlay official out/" in _qemu
assert "fuse2fs" in _qemu
assert "ttyS0" in _qemu
assert "usb-tablet" in _qemu
assert "xres=1280" in _qemu
assert "yres=720" in _qemu
assert "virtio-vga" in _qemu
assert "plant-chromium.sh" in _qemu
assert "lab-overlay" in _qemu
assert "-m 12288" in _qemu
assert "host_cpus=$(nproc)" in _qemu
assert 'smp=$((host_cpus / 2))' in _qemu
assert "package-shell-gui-rootfs.sh" not in _qemu
assert "qemu-empiieros-smoke.sh" not in _qemu
assert "/dev/sda" not in _qemu

# Seat Shield H6 / PrivateTmp split (TZ-SEAT-SHIELD-H6-PRIVATETMP.md):
# Runtime keeps a private tmp jail; SystemCallFilter applies only after +SECCOMP install.
assert "PrivateTmp=true" in unit
assert "PrivateTmp=false" not in unit
assert "SystemCallFilter=~@debug" in unit
_ts_overlay = Path(
    os.environ["ELITEEW_TS_ROOT"] + ""
    "/config/buildroot/external/board/eliteew-ts/rootfs_overlay"
)
assert (_ts_overlay / "usr/local/bin/empiieros-state-unlock-agent").is_file()
assert (_ts_overlay / "etc/systemd/system/empiieros-state-unlock-agent.path").is_file()
assert (_ts_overlay / "etc/systemd/system/empiieros-state-unlock-agent.service").is_file()
_console_mask = _ts_overlay / "etc/systemd/system/systemd-ask-password-console.path"
assert _console_mask.is_symlink() and _console_mask.readlink() == Path("/dev/null")
domains = Path("runtime/src/observed_domains.rs").read_text(encoding="utf-8")
assert "host_proc_mounts" in domains
assert "/run/empiieros/host-mounts" in domains
