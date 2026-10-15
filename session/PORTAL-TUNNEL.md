# EmpiierOS — FoFline portal tunnel (OS plane)

**Status:** Living contract. Production path: `runtime/src/launcher_portal.rs`.
**Plane:** `session/` + Runtime helper. **Not:** Tauri boringtun. **Not:** `wg-quick`.
**Not:** punching Runtime `IPAddressDeny`.

## Purpose

Bring up the split-tunnel portal (`wg-portal` → VIP `10.88.89.1/32`) from a
`channel_valid` Capsule conf. The datapath is in-tree Linux WireGuard built
from the factory kernel sources. Userspace `wg` is built from
`wireguard-tools` sources as `/usr/local/sbin/empiieros-wg`.

## Invariants

1. **OS plane only.** Runtime IPC `launcher.channel_up` starts allowlisted
   `empiieros-portal-up.service`. Runtime keeps `IPAddressDeny=any`.
2. **From source.** Kernel module `CONFIG_WIREGUARD=m` plus crypto/udp_tunnel
   deps. No distro kmod. No userspace boringtun/smoltcp.
3. **No wg-quick.** Helper calls `ip` + `empiieros-wg set`. Hook keys stay
   forbidden by `validate_portal_conf`.
4. **Split-tunnel.** `AllowedIPs` exactly `10.88.89.1/32`. Default route via
   `wg-portal` is `portal_default_route_forbidden`. Some non-portal iface
   (`wlp*`, `en*`, …) must keep a global IPv4 after bring-up. Do not pin
   lab `eno1` or the `/30`. Missing uplink is `portal_broke_uplink`
   (historical `portal_broke_eno1` was that false fail on Wi-Fi seats).
5. **No `/etc/wireguard` plant.** Conf stays `{state}/wireguard/wg-portal.conf`
   (`0600`) so substrate `eliteew-ts-tunnel` / `wg-quick@` does not auto-start.
6. **Observe honesty.** `channel_up` is `/sys/class/net/wg-portal/operstate`,
   not a stored flag. `channel_code` is `valid_up` when valid and up.
7. **Secrets.** PrivateKey never enters IPC or helper result.
8. **Host-map.** After the iface is up, bring-up must leave
   `portal_hostmap=applied` (`app`/`auth`/`meet.eliteew.ru` →
   `10.88.89.1` in `/etc/hosts`). Runtime repairs once if an older
   helper skipped the write; still missing is `portal_hostmap_required`.
   See [`PORTAL-HOSTMAP.md`](PORTAL-HOSTMAP.md) and
   [`SEAT-FOFLINE-CANONS.md`](SEAT-FOFLINE-CANONS.md) §2.
9. **Prelicense egress.** Substrate `eliteew_ts_prelicense` output must accept
   TCP 80/443 to `10.88.89.1` after the handshake. ICMP-only is not HTTPS.
   Public edge VIP `443` stays denied. Meet ICE is UDP `13400` only to
   `93.77.165.185` (not a general UDP punch).

## IPC

`launcher.channel_up` — after unlock + `channel_valid`. Success flags only.

## Live lab (2026-09-18, not an official image)

- Kernel module from factory linux-6.12.20; userspace `empiieros-wg` from
  wireguard-tools 1.0.20210914 (`WITH_WGQUICK=no`).
- Mux: `20260917T165634Z-wg-src-5223`, portal helper `20260917T165639Z-portal-helper-7752`,
  Runtime `20260917T165640Z-runtime-31460`.
- Observed: `channel_code=valid_up`, `wg-portal` UP MTU 1380, VIP ping 1/1,
  `eno1` `192.168.8.10/30`, no default via `wg-portal`, `/etc/wireguard` empty,
  Runtime `IPAddressDeny=any`, `product.open` unavailable.
- Official `out/` still lacks the module and portal helper.
