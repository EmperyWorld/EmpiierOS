# EmpiierOS foundation status

**Updated:** 2026-09-22 (archive identity; metal still open)
**Verdict:** `NOT_READY`

On 2026-09-22 both `buildroot-output/images/rootfs.tar.gz` and
`trusted-substrate/out/rootfs.tar.gz` hash to `cf0b46e1…` and match the
stamp. `5979e9cf…` remains the 2026-09-20 ledger bake. It is not the
archive on disk. Rows below that cite `5979e9cf…` describe that bake.
They are not deleted. Metal acceptance is still open. A USB plant is
not this archive.

The 2026-09-16 audit invalidated earlier “Done/Wired” and aggregate wave-count
claims. Historical wave documents remain design and implementation records, not
acceptance evidence.

| Living authority | Role |
|------------------|------|
| [`plans/MATURITY-LEDGER.md`](plans/MATURITY-LEDGER.md) | Capability status |
| [`plans/METAL-LIVE-BASELINE.md`](plans/METAL-LIVE-BASELINE.md) | Durable live-seat snapshot (not readiness) |
| [`plans/SECURE-RESET-LIVE-DIARY.md`](plans/SECURE-RESET-LIVE-DIARY.md) | Chronological lab log (not readiness) |
| [`plans/PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md`](plans/PRODUCT-WAVE-FOFLINE-LICENSE-INSTALL.md) | Active scoped wave |
| [`plans/TZ-DISK-AT-REST.md`](plans/TZ-DISK-AT-REST.md) | LUKS2 state slot (PIN ≠ volume key) |
| [`../session/SEAT-FACE.md`](../session/SEAT-FACE.md) | Living glass map (source, not `out/`) |
| [`../session/OS-INSTALL.md`](../session/OS-INSTALL.md) | V2 install journey |
| [`../runtime/HEALTH-IPC.md`](../runtime/HEALTH-IPC.md) | Production Unix ops |
| [`../.cursor/orchestration/reports/deep_secure_reset_audit.md`](../.cursor/orchestration/reports/deep_secure_reset_audit.md) | Audit facts |

## Current gates (2026-09-21)

| Gate | Result | Honest limit |
|------|--------|--------------|
| `scripts/preflight-foundation.sh` | exit 0 | EmpiierOS source/hook/Shell/Runtime gate only |
| Official image `image_contract_ok` | `out/rootfs.ext2` `e48aab38…` (2026-09-20 LUKS floor bake), bzImage `69bc60b6…` | Overlay `670612ea…`; Runtime `79e06a7d…`; GUI `f2fe1900…`; Shell `index-DMc2IDZV.js`; fw `c0d89634…`; watch `3a93cc44…`; stamp `rootfs_sha256=5979e9cf…`. Archive has `/usr/sbin/cryptsetup`; kernel `CONFIG_DM_CRYPT=y`. Prior host bake `4104a67b…` superseded |
| `qemu-empiieros-smoke.sh` | **not rerun** this bake | Prior H1 FAIL (`WebKit sandbox enabled` absent) still stands. Do not treat as metal |
| `qemu-installed-system-smoke.sh` | not rerun | Now requires cryptsetup in the live image (LUKS2 p3). Prior exit 0 was a 2-partition plain disk |
| `ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh` | **FAIL** | Missing kinship readiness doc; not an image hash miss |
| Physical USB / fresh-metal install | USB written; LUKS metal walked; **not** MATCH | Stick `071C57D0F48A4A92` (`/dev/sda`, `TRAN=usb` `RM=1`). Official factory SoT still `5979e9cf…`. Live mux SoT `c9993f85…` (Runtime `f734ed76…` with deferred cutover restart `--on-active=3s` in SoT + share + live). Live apply `01fd70dd…` keeps extracted IdP-Date Runtime, plants OTA helper + `ota-cutover.conf` (no `plant_seat_ota_pk`). Seat-shell share `index-C33yKaFx.js`. No `artifact.pk.hex` on ESP or installer share. Official `out/` ships cryptsetup. Already-installed disks still have the silent SoT wifi unit until reinstall. QEMU not rerun |
| Installer proof-before-`ERASE` | baked ticket-only helper on the USB | Apply+stamp: canonical `install-proof.json`, no media `.ecap`. QEMU install smoke not rerun. LUKS metal walked; not MATCH |
| Unlock-only launcher Shell | baked + muxed | Official Shell `index-DMc2IDZV.js` / GUI `f2fe1900…`. Live mux is still the previous seat. `pin_issue` is `installer_issued_only`. Lab PIN already unlocked |
| Consume-after-unlock Runtime | baked + lab mux | Official Runtime `79e06a7d…`. Live snapshot mux still `48f1275e…`. Unlocked seat: import `path_not_allowed` |
| Inner-capsule persist | live | Port `62890` persisted `{state}/capsule_inner/` + sealed `current.ecap` `2469`. `entitlement_present=true` `allow_fofline=true`. |
| Portal channel validate | live | Split-tunnel conf still required. Live now also starts the OS-plane iface. |
| OS-plane portal (src WG) | baked + lab mux | Official image has `empiieros-wg` `bbdc4dcd…` and `wireguard.ko`. Live seat still uses muxed module `8b2379ff…`. `channel_code=valid_up`. `/etc/wireguard` empty. Factory still also ships `/usr/bin/wg-quick`. |
| FoFline SPA pack OTA | baked + lab mux | Official overlay ships `empiieros-spa-fetch` `e20c5169…`. Live `spa_state=ready` `1789890833725-7y746j4lvw` after `spa_sync` 18:12 +10. `assets_core_mismatch` 7/8 (`include_audio=false`). Public VIP 443 stayed denied. |
| Helper/Runtime pair | baked + live mux | Official Runtime `79e06a7d…` / watch `3a93cc44…` / fw `c0d89634…` / GUI `f2fe1900…`. Live Runtime `48f1275e…` / watch `8525c72b…` / fw `1a3856a2…`. Live systemd still `-SECCOMP`. |
| `product.open` | lab mux local only after OIDC | Live session present; 11:38:07 inject + `fofline-spa` serve. Official `out/` now bakes current source (`f229c221…`); QEMU not rerun. Quote fonts stay denied. |
| Portal host-map | baked + lab mux | Official portal helper unit writes `/etc/hosts`. Live `hostmap-1` `applied`. Public `93.77.165.185:443` failed. |
| Edge `/api/health` | baked + lab mux | Official Runtime has `launcher.edge_health`. Live `edge-health-1` `applied` `{"ok":true}` via VIP. |

EmpiierOS preflight is **not** the Trusted Substrate selftest. A local exit 0
does not replace image, QEMU, or metal acceptance.

## Kinship documentation gap

`trusted-substrate/scripts/selftest-installer.sh` still requires

`documentation/reference/solution-readiness/2026-09-08-trusted-substrate-mit-disk-installer.md`

That file is **absent** from the monorepo. EmpiierOS does not invent kinship
readiness documents to force the factory selftest green. This is factory
documentation debt. QEMU image and installer-disk smokes remain the image
**gate**; last run is 2026-09-18 H1 FAIL and was **not rerun** after the
2026-09-20 LUKS bake.

## Current secure-reset scope

- Authenticated Unix socket (`SO_PEERCRED` + root-owned proxyd). No HTTP caller role.
- Simulated and unsafe operations compiled out of the production Runtime path.
- Observed host, network, power, and security state without optimistic defaults.
- Atomic Ethernet/Wi-Fi changes with observed verification.
- One supervised boot lifecycle for Runtime, compositor, and Shell.
- Reproducible EmpiierOS overlay inside the Trusted Substrate factory.
- Scoped FoFline host: Capsule v3 file import, living air consume, local
  first-boot PIN unlock, OS-plane portal, SPA pack OTA, native OIDC card,
  typed local `product.open`, always-on HUD, Seat Browser overlay, Seat
  Shield host workspace, V2 disk install (LUKS2 2 GiB state; PIN ≠
  volume key), Wi-Fi `AF_INET` plant after extract, empty Settings IPs
  (lab `/30` refused). Installer plants the PIN. Remote SPA stays
  denied. Official `out/` `5979e9cf…` is behind the live apply/Wi-Fi
  plant. QEMU smoke is red on the WebKit sandbox string and was not
  rerun after the LUKS bake.

Write scope for the factory is the recorded secure-reset path
`powered_by_emperyworld/pack-organization/trusted-substrate/` only. Do not write
`connection-agent` or other kinship trees.

## Current lab seat (not metal acceptance)

Live `192.168.8.10/30` on `eno1` is the **polygon**. It is not a
product Settings default. Apply refuses those addresses; persist
writes `10-` static only after `STATIC_OK_FILE`.

OTA operator how-to: [`plans/OTA-OPERATOR-RUNBOOK.md`](plans/OTA-OPERATOR-RUNBOOK.md).
Current EmpiierOS overlay `7590c54c…` Runtime `2783fa8a…` has deferred
cutover restart + `ota_artifact_pk` + `ota-cutover.conf` + Shell
`index-BKgVdGT6.js`. That is **not** official `out/`. Prior overlay
`0e0e19d8…` / Runtime `6737388a…` superseded as the current link.

Official factory `out/` SoT is `5979e9cf…` (2026-09-20 LUKS floor:
`rootfs.ext2` `e48aab38…`, overlay `670612ea…`, Runtime `79e06a7d…`,
GUI `f2fe1900…`, Shell `index-DMc2IDZV.js`). Live-installer USB
`071C57D0F48A4A92` currently carries mux SoT `c9993f85…` (Runtime
`f734ed76…` in SoT + share + live; deferred cutover restart). Live
apply `01fd70dd…` matches overlay keep-SoT + helper plant. Seat-shell
`index-C33yKaFx.js`. Prior mux SoT `245efa91…` / Runtime `74a24895…`
/ apply `c6618d61…` / SoT `e150127f…` / `5843eea8…` / `f56dba1b…`
/ apply plant `8a9ec933…` / `d250a976…` /
host bake `4104a67b…` / slice `fa1fbc6b…` are superseded. Do not
quote them as current.

LUKS2 state + recovery-reveal + first-boot PIN (`fp_v2`) were walked
on metal. Post-install Wi-Fi failed when extract planted the official
SoT unit without `AF_INET`; the live USB now plants the unit. An
already-installed disk stays silent until reinstall. QEMU not rerun.
Runtime `IPAddressDeny=any` stayed. Seat Shield is a FoFline host
workspace. Quote fonts stay denied. Not MATCH.
Diary: [`plans/SECURE-RESET-LIVE-DIARY.md`](plans/SECURE-RESET-LIVE-DIARY.md).

## Agent kernel

Living inventory: **9 alwaysApply rules · 1 glob rule · 5 hook bindings · 13 skills · 5 agents**.
Language bridge is **disabled**. See
[`foundation/AGENT-KERNEL-SPEC.md`](foundation/AGENT-KERNEL-SPEC.md) and
[`foundation/LANGUAGE-BRIDGE.md`](foundation/LANGUAGE-BRIDGE.md).

## Promotion gate

```bash
npm ci --prefix shell-ui
bash scripts/preflight-foundation.sh
```

Image evidence, when tasked:

```bash
cd ../powered_by_emperyworld/pack-organization/trusted-substrate
bash scripts/qemu-empiieros-smoke.sh
bash scripts/qemu-installed-system-smoke.sh
```

`ELITEEW_TS_SECURE_RESET_QEMU=1 bash scripts/selftest-secure-reset.sh` stays red
until the missing kinship readiness document exists or the factory check is
replaced with a stricter correct invariant. Do not delete the check to obtain
green.

## Known exclusions

- No current `verified-real` capability claim.
- dm-verity, Secure Boot, measured boot, and A/B updates are not accepted.
- Operator authentication, entitlement, package update, and destructive
  maintenance remain disabled in production until redesigned. Typed local
  `product.open` after native OIDC is on the lab mux; official `out/`
  `5979e9cf…` bakes the Open source slice but QEMU did not walk Open.
  Remote SPA stays denied. Glass MATCH stays the operator's.
- Hardware acceptance requires a newly built image and a fresh installation on
  disposable media — never a silent reinstall of the open SSH lab.
- Official `out/` is behind the live apply/Wi-Fi `AF_INET` plant. A USB
  plant is not factory SoT.

## 2026-09-18 sanitation

Living SoT was split from the chronological lab diary. Stale “selftest passes”
and “36 Runtime tests” claims were removed from this file. `.cursor` inventory
was confirmed against the kernel spec. The empty `.cursor/commands/` directory
and the dead Gemma translator that imported the removed language-bridge skill
were soft-deleted. Historical P0–P92 `PRODUCT-WAVE-*` files were
soft-deleted to `trash/` on 2026-09-21. `docs/capture/` stays origin
archive.
