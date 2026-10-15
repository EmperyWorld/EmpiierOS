# Seat Shield floor helper protocol

**Binary (prep):** `seat-shield-fw`
**Future EmpiierOS name:** `empiieros-seat-shield-fw` (absolute path under `/usr/libexec/` or similar)
**Caller:** Runtime `HelperActuator` via `command_runner` (env cleared, argv only)

## Invariants

1. Helper path must be **absolute**. Production path is Runtime
`command_runner` (env cleared). The optional oneshot unit is not a
listening daemon (`Type=oneshot`, no `Listen*`). It may carry
`CAP_NET_ADMIN` + `NoNewPrivileges`; it must **not** set `IPAddressDeny`
(nft is netlink).
2. No shell, no `sh -c`, no interpolated nft scripts from Shell.
3. `--state-dir` must be absolute (required for every backend — protocol stability).
4. Peer IPs must parse as IP addresses.
5. One JSON object on stdout; exit `0` on success, non-zero on failure.
6. Backends share the **same** argv action contract; only global flags differ.

## Global flags

```text
--state-dir ABS
[--backend state|nft]          # default: state
[--nft-bin ABS]                # default: /usr/sbin/nft (nft backend)
[--nft-family inet|ip|ip6|…]   # default: inet
[--nft-table NAME]             # default: seat_shield
[--nft-set NAME]               # IPv4 set; default: blocked4
[--nft-set6 NAME]              # IPv6 set; default: blocked6
```

Runtime may pin an allowlisted **prefix** of these flags when invoking the helper
(`--backend`, `--nft-bin`, `--nft-family`, `--nft-table`, `--nft-set`, `--nft-set6` only).

## Commands

```text
HELPER [global…] --state-dir ABS block --ip IP [--reason TEXT]
HELPER [global…] --state-dir ABS unblock --ip IP
HELPER [global…] --state-dir ABS list
```

### Backend: `state` (prep)

Durable `fw_blocked.json` under `--state-dir`. Not metal.

### Backend: `nft` (metal-shaped)

Absolute `nft` binary; **only** these argv templates:

```text
nft add element FAMILY TABLE SET { IP }
nft delete element FAMILY TABLE SET { IP }
nft -j list set FAMILY TABLE SET
# fallback when nft was built without JSON:
nft list set FAMILY TABLE SET
```

`SET` is chosen by peer address family: IPv4 → `--nft-set`, IPv6 → `--nft-set6`.
`list` merges both sets. JSON is preferred; text parse is required on this
floor (`nftables` without libjansson).

No shell. Idempotent add (already-present → ok). Missing delete → `removed:false`.

**Floor prerequisite (metal):** table/sets must already exist
(`inet seat_shield` + `blocked4` + `blocked6`). Helper does **not** create tables.

Prep paste candidates: `floor/seat-shield/` (`seat_shield.nft` + systemd units).
Evidence: `./scripts/seat-shield-evidence.sh` (`netns_metal=match` via `unshare -rn`).

## Success examples

```json
{"ok":true,"action":"block","peer_ip":"198.51.100.20","backend":"nft"}
{"ok":true,"action":"block","peer_ip":"2001:db8::20","backend":"nft"}
{"ok":true,"action":"unblock","peer_ip":"198.51.100.20","removed":true,"backend":"state"}
{"ok":true,"action":"list","peers":["198.51.100.20","2001:db8::20"],"backend":"nft"}
```

## Failure example

```json
{"ok":false,"error":"invalid IP: nope"}
```
