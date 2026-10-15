# EmpiierOS — Portal hostname map

**Status:** Living contract. Production path: `runtime/src/portal_hostmap.rs`.
**Plane:** `session/` + portal helper. **Not:** Tauri SOCKS. **Not:** punching
Runtime `IPAddressDeny`.
**Canons:** [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md) §2.

## Purpose

After the OS-plane portal is up, the kiosk resolver must send
`app.eliteew.ru` and `auth.eliteew.ru` to VIP `10.88.89.1` so the local
FoFline pack can reach API/OIDC. Public TCP 443 stays denied.

## Invariants

1. **Exact names.** Only `app.eliteew.ru`, `auth.eliteew.ru`, and
   `meet.eliteew.ru`. No wildcards. Meet HTTPS uses the same VIP SNI path
   as Tauri SOCKS. WebRTC UDP to `93.77.165.185:13400` is not this map.
2. **Exact VIP.** `10.88.89.1` only. The helper writes a marked block at the
   top of `/etc/hosts` (`0644`, regular root-owned file, no symlink).
3. **Portal first.** `launcher.hostmap_apply` requires unlock + `wg-portal`
   up. `launcher.channel_up` applies the same block after the iface is up.
4. **OIDC needs the map.** `launcher.channel_up` refuses success unless
   `portal_hostmap=applied` (repairs once if an older helper skipped the
   write). `launcher.oidc_start` also fails closed with
   `portal_hostmap_required` when the map is missing. PAR can still reach
   the VIP by IP without DNS; the auth WebView cannot, and paints a white
   Keycloak card.
5. **Runtime stays closed.** The helper has `ReadWritePaths=/etc/hosts`.
   Runtime keeps `IPAddressDeny=any` and does not write `/etc/hosts`.
6. **Air residual.** A localhost-only `/run/empiieros/hosts-air` is bind-mounted
   over `/etc/hosts` in `empiieros-air-enroll.service` when present. If NSS
   asks systemd-resolved first, air redelivery may still see the VIP map.
7. **Not a VPN.** Default route stays on `eno1`. DNS for other names is
   unchanged.

## IPC

`launcher.hostmap_apply` — after unlock + portal up.
`launcher.status` reports `portal_hostmap=applied|missing`.

## Live lab (2026-09-18, not an official image)

- Runtime `eb06b81f…` (`20260917T180522Z-runtime-5485`), portal helper
  `e20a8f83…`, air helper `1f751aa4…`.
- `hostmap-1` `applied`. `portal_hostmap=applied`.
- `/etc/hosts` `0644` starts with the marked VIP line.
- `resolvectl query` `app.eliteew.ru` / `auth.eliteew.ru` → `10.88.89.1`
  (synthetic from hosts). NSS is `resolve` first; resolved still reads hosts.
- TCP `10.88.89.1:443` open. Public `93.77.165.185:443` failed (prelicense).
- `eno1` `192.168.8.10/30`. Default is not `wg-portal`.
- Runtime `IPAddressDeny=any`. Seat has no `curl`/`openssl`; HTTPS body was
  not re-fetched this turn (SPA OTA already used VIP:443 + SNI).
