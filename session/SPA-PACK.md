# EmpiierOS — FoFline SPA pack bootstrap

**Status:** Living contract. Production path: `runtime/src/launcher_spa.rs`.
**Plane:** `session/` + Runtime helper. **Not:** Tauri SOCKS. **Not:** `product.open`.
**Not:** capsule-gated OTA ([`OTA-CAPSULE.md`](OTA-CAPSULE.md)).
**Glass / content-audio leftovers:** [`SEAT-FACE.md`](SEAT-FACE.md).

## Purpose

Fetch the **first** FoFline workspace pack after `allow_fofline` and a live
portal (journey stage `spa`, **before** OIDC). This is **bootstrap pack**,
not capsule OTA. Subsequent OS/FoFline update capsules use `launcher.ota_*`
after license + hardware + live EliteEW login + portal VIP attest.
Manifest is `https://app.eliteew.ru/fofline-updates/spa-latest.json`.
Artifact is `EliteEW_FoFline_SPA-<app_build_id>.tar.zst` (~100 MiB).

## Invariants

1. **OS plane.** Runtime IPC `launcher.spa_sync` starts
   `empiieros-spa-fetch.service`. Runtime keeps `IPAddressDeny=any`.
2. **VIP only.** Helper `IPAddressDeny=any` + `IPAddressAllow=10.88.89.1`.
   HTTPS connects to `10.88.89.1:443` with SNI `app.eliteew.ru`. No SOCKS,
   no boringtun, no `/desktop-client-updates/`. Substrate prelicense output
   must allow TCP 80/443 to `10.88.89.1` only — not the public edge VIP.
   ICMP + WG UDP alone is not enough; a default-deny table drops the SYN.
3. **Verify before promote.** sha256 then Tauri/FoFline minisign. Empty
   signature is `signature_required`. Audio packs (`include_audio=true`)
   are `audio_pack_forbidden`. Optional groups use a second channel
   `spa-assets-latest.json`. Each group is
   `EliteEW_FoFline_SPA_GROUP-<app_build_id>-<id>.tar.zst`. Merge is
   sparse into `current/` and never replaces `index.html` /
   `pack-manifest.json`. Hash-skip when archive sha256 + `files.json`
   still match. `audio-guides` is deferred (1.5 GiB). User/generated
   media is not a group — it stays authenticated `/api/v1/content`.
   The native `fofline-spa` API proxy returns XHR download bodies as
   `application/octet-stream`. CORS is the `fofline-spa://localhost`
   origin plus `Allow-Credentials` (not `*`). `Range` is stripped.
   Media clients that sent `Range` get a sniffed audio MIME so
   `canplay` can fire. A native user script (`__empiierosAudioBlob`)
   rewrites remaining `<audio src>` content URLs to a typed `blob:`
   and calls `load()`.
   This is not OS A/B OTA, not capsule-gated `launcher.ota_*`, and not
   the deferred `audio-guides` group.
   Channel `app_build_id` must match the local core or groups are
   skipped (update core first).
4. **Safe extract.** No `..`, absolute paths, or links. Caps match living
   core pack (512 MiB compressed, 1 GiB decompressed).
5. **Store.** `{state}/fofline-spa/` is traverse-only `0751`; `current/` is
   `0755`; pack files are `0644`; `state.json` stays `0600`. `staging/` and
   the download spool stay `0700`. Ready requires `index.html` +
   `pack-manifest.json`. The kiosk must be able to walk `current/` at GUI
   start, not only after `product.open`.
6. **IPC flags only.** `spa_state` / `spa_code` / `spa_app_build_id`. Signature
   and archive never enter the response.
7. **Local open.** After unlock + `allow_fofline` + `ready` + native OIDC,
   `product.open` republishes the same traverse modes (no `chown`; Runtime
   stays `CAP_NET_ADMIN` only) and returns the typed
   `fofline-spa://localhost` start URL. The scheme is secure + CORS, not
   local. Module scripts finish with CORS headers. Every stylesheet uses
   simple `finish()` so Vite `__vitePreload` `<link>` tags fire `load`.
   Remote SPA stays denied.

## IPC

`launcher.spa_sync` — after unlock + `allow_fofline` + `channel_up`.
Core first, then optional groups. IPC also returns `spa_assets_ready` /
`spa_assets_total` / `spa_assets_code`.
`launcher.edge_health` — after unlock + `allow_fofline` + portal up: helper
GET `/api/health` to VIP `10.88.89.1` (SNI `app.eliteew.ru`). Runtime keeps
`IPAddressDeny=any`. Response is a body prefix only.
`launcher.status` observes pack and group flags.

## Live lab (2026-09-18, not an official image)

Cause of the first fetch fail: substrate `eliteew_ts_prelicense` output
dropped TCP 443 (ICMP and TCP/53 to the VIP still passed). Lock: overlay
`prelicense.nft` accepts TCP 80/443 only to `10.88.89.1`. Public edge VIP
443 stays denied.

- Helper `empiieros-spa-fetch` `c0704308…` finished 2026-09-18 03:28:43 +10
  in 14s: `spa_state=ready` `spa_app_build_id=1786238708853-jn25ybw0v0`.
- Store `{state}/fofline-spa/current/` has `index.html` + `pack-manifest.json`
  (`include_audio=false`, `kind=fofline-spa`). State and `fofline-spa/` are
  `0751`; `current/` is `0755`; files are `0644` after Runtime start / pack
  ready, not only after `product.open`. PIN and `last_launch.json` stay `0600`.
- Replay: `FoFline workspace already current`.
- Runtime `IPAddressDeny=any` stayed. Helper allow is VIP-only.
- `eno1` stayed `192.168.8.10/30`. Default is not `wg-portal`.
- Live `product.open` `applied` for the local pack. Official `out/` still
  lacks the helper.
- Live `launcher.edge_health` `applied` `{"ok":true}` via VIP `/api/health`
  (helper `e20c5169…`). Runtime `IPAddressDeny=any` stayed.
- Official overlay `c78e2d11…` ships the same SPA helper plus `empiieros-wg`.
- Optional groups 2026-09-18 14:04–14:08 +10: helper `5859fdb9…`
  (`20260918T040334Z-spa-helper-7404`) + Runtime `42ca26b9…`
  (`20260918T040343Z-runtime-25398`). First `spa_sync` after core ready
  merged 7/8 catalog groups in ~136s (`assets_partial`; `audio-guides`
  deferred). Disk now has `desktop-wallpapers/15.webp` and
  `fonts/elite-quote/cormorant-garamond-cyrillic-400.woff2` (`0644`,
  dirs `0755`). `assets-state.json` stays `0600`. Replay hash-skip
  2.8s, same 7/8. User media stays `/api/v1/content`. Runtime
  `IPAddressDeny=any` stayed. `eno1` stayed `192.168.8.10/30`.
  Official `out/` unchanged.
- Official SPA pack staged 2026-09-18 15:07–15:11 +10 on command-center:
  `app_build_id=1789443659753-rovukoqfw9` (same web bake
  `eliteew-frontend:prod-baked-20260915-1340`). Core
  `EliteEW_FoFline_SPA-1789443659753-rovukoqfw9.tar.zst` `15M`
  sha256 `5384ff3f3f2b…` contains `EmperyDesktopNativeOidc`. Eight
  signed groups include elite-quote in `o-proekte`. Public
  `spa-latest.json` is public `1789443659753-rovukoqfw9` after
  rsync to `.111` without dest prune. Lab `spa_sync` 15:20–15:22
  +10 promoted `current/` to that id (`assets_partial` 7/8).
  Official EmpiierOS `out/` unchanged.
- Public core superseded 2026-09-18 18:37Z:
  `app_build_id=1789720073125-5fjlwh96mc` sha256 `60be0fc8…` (web bake
  `eliteew-frontend:prod-baked-20260918-1759`). Lab helper sync
  19:09:41–19:10:05 +10 installed that id. Groups stayed on the old
  id (`assets_core_mismatch` 0/8). Packed client maps opaque
  `fofline-spa:` to `https://app.eliteew.ru`. Official `out/` unchanged.
- Public core superseded 2026-09-18 11:28Z:
  `app_build_id=1789730765446-d2ufr39pnb` sha256 `c34c224e…`
  (notes: EmpiierOS Meet stage close on hangup; dist-build
  `2026-09-18T11:27:25Z`). Lab helper sync 21:49:53–21:50:16 +10
  installed that id. Groups stayed on `rovukoqfw9`
  (`assets_core_mismatch` 0/8). Packed `VideoCallApp-IsuY-m0b.js`
  calls `closeMeetStage()` with no URL. Official `out/` unchanged.
- Public core superseded 2026-09-18 16:51Z:
  `app_build_id=1789750041479-4jxqj4w8lj` sha256 `9296bc5a…`
  (notes: FoFline call stays in the VideoCall window; end on Meet
  death; dist-build `2026-09-18T16:50:26Z`). Lab helper sync
  02:58:26–02:58:39 +10 installed that id. Groups stayed on
  `rovukoqfw9` (`assets_core_mismatch` 0/8). Packed
  `VideoCallApp-BhyD_kAW.js` has no `openMeetStage` / no
  `openEmpiierosMeetStage`, mounts `iframe`, listens
  `empiieros-meet-stage-lost` + `pagehide`. Keepalive
  `POST /api/v1/video-calls/{id}/end` (Bearer access, `keepalive:!0`)
  is in `DesktopSocketNotificationsBridge-DItrm048.js`. Official
  `out/` unchanged. This pack's nested Meet iframe is a WebKitGTK
  dead end on EmpiierOS (product process crash 2026-09-19 03:34 +10).
  Next pack must use OS `openMeetStage(url,bounds)` /
  `setMeetStageBounds` — see
  `docs/plans/ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md`.
- Public core superseded 2026-09-19 05:15Z:
  `app_build_id=1789794743865-pza6u2zqzf` sha256 `b5c30279…`
  (notes: FoFline Meet is a clipped sibling stage; end on Meet death;
  dist-build `2026-09-19T05:13:50Z`). Lab helper sync
  15:33:20–15:33:38 +10 installed that id. Groups stayed on
  `rovukoqfw9` (`assets_core_mismatch` 0/8). Packed
  `VideoCallApp-bs-BnhEY.js` sha256 `e3251158…` has
  `setMeetStageBounds`, `openMeetStage(url,bounds)`,
  `meet_stage_bounds_required`, `empiieros-meet-stage-lost`,
  `pagehide`. Chrome iframe JSX remains in the chunk; EmpiierOS
  host hook is the default call path. Keepalive
  `POST /api/v1/video-calls/{id}/end` is in
  `DesktopSocketNotificationsBridge-uMaYm0aX.js`. Official `out/`
  unchanged.   Product WebKit still holds the previous JS until poet
  reopens FoFline.
- Public core superseded 2026-09-19 17:05Z:
  `app_build_id=1789837366816-w7k51h8btf` sha256
  `d3f6f6676b099af2cdb318a86b38adeb129bb921a1501ea7d7b5aa3a913da16a`
  (notes: FoFline on `foflineNative` plays via `decodeAudioData` +
  `AudioBufferSourceNode`, not media URL; dist-build
  `2026-09-19T17:05:57Z`). Lab helper sync 03:12 +10 installed that
  id (`spa_state=ready`, `include_audio=false`,
  `assets_core_mismatch` 0/8). Packed `contentApi-Bstp_TrW.js`
  sha256 `11d0e5ad…` has `decodeAudioData` / `createBufferSource`
  and skips `Range: bytes=0-0` when `Ro()` (`foflineNative` /
  `fofline-spa:`). Ticket / `<audio src>` remain for Chrome/Tauri.
  Official `out/` unchanged. Product WebKit still holds
  `pza6u2zqzf` until poet reopens FoFline. Play measure is then:
  `/download` 200, no `blob:fofline-spa:` cancel, no media
  `download-by-ticket`, analog PCM not stuck `0/1` after Stop.
- Public core reported 2026-09-20 18:02 +10: `app_build_id=1789890833725-7y746j4lvw`
  sha256 `da97f2877868ed806ece375b9643f6d43ad3e330c57938347f9ea24546ede7d8`
  (`include_audio=false`). Pack opens `EmperyDesktopSeatBrowser` on
  EmpiierOS (https only; no Tauri `ebrowser_open`). stack0 / edge
  `1789891909119-45tkayntro` is live (`prod-baked-20260920-1811`);
  spa-latest stayed this id. Lab already synced. Not MATCH.
