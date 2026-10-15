# Network Manager & Connection Wizard Contract (P80)

**Role:** Session plane contract for first-party interactive network configuration, Wi-Fi SSID scanning, static/DHCP IP, and DNS setup.
**Plan companion:** historical wave in `trash/`. Living status: [`../docs/plans/MATURITY-LEDGER.md`](../docs/plans/MATURITY-LEDGER.md).
**Parent contract:** [`CONTRACT.md`](CONTRACT.md)

---

## 1. Purpose

Provides the OS Shell with a first-party, bounded network control plane for discovering network interfaces, scanning Wi-Fi access points, configuring static/DHCP IPv4 connections, tuning DNS servers, and toggling air-gap offline mode without spawning bloated external desktop network tools (`nm-connection-editor`, `gnome-control-center` network panel) or executing unconfined shell scripts.

---

## 2. Invariants

1. **Singular Network Floor:** All network status queries and configuration actions are managed directly through the first-party EliteEW Runtime bridge (`runtime/src/network_manager.rs`). Never spawn unconfined network management applets or execute raw shell networking scripts (`netplan`, `ip link`, `iptables`).
2. **Interactive Bounded Wi-Fi Discovery:** Scanning for wireless networks returns typed access points with signal strength percentages, BSSIDs, and security profiles (`wpa3`, `wpa2`, `open`). Credentials must be strictly validated (SSID: 1..=32 chars, PSK: 8..=63 chars).
3. **Safe IPv4 Addressing & DNS Validation:** All IP addresses and DNS resolvers are validated against IPv4 syntax before being applied. Loopback interface (`lo`) is protected and can never be disabled or reconfigured. Host IP cannot be `0.0.0.0` or `255.255.255.255`. Prefix length must be between 1 and 32. Up to 3 DNS servers may be specified. Settings forms must not pre-fill the lab polygon `192.168.8.10/30` / `192.168.8.9`. `network.wifi.connect` waits for association first, then kicks DHCP; a late lease must not roll back a proven PSK.
4. **Air-Gap Offline Isolation Mode:** A dedicated offline mode flag immediately disables external interfaces, preventing network egress and ingress, while preserving internal loopback (`127.0.0.1`) IPC connectivity.
5. **Auditable Logging:** All connection attempts, IP reconfigurations, DNS changes, and offline mode toggles are appended to `{state_dir}/network_manager/network_audit.json` (`0o600`).
6. **Persistence & Clean State:** Saved network profiles and active preferences are stored in `{state_dir}/network_manager/profiles.json` (`0o600`).
7. **Factory Reset Clean:** The `{state_dir}/network_manager/` directory is completely removed during seat local factory reset (`seat.local_factory_reset`).

---

## 3. Storage & Layout

```text
{state_dir}/network_manager/
  ├── profiles.json       # 0o600 saved Wi-Fi profiles and network interface configurations
  └── network_audit.json  # 0o600 audit trail of network configuration actions
```

---

## 4. Non-Goals

- Spawning external desktop network applets (`nm-connection-editor`, `gnome-control-center`, `wicd`, `connman-gtk`).
- Executing raw shell scripts or invoking unrestricted subcommands (`netplan apply`, `ip link`, `iptables`).
- Exposing raw system D-Bus sockets directly to the webview or JavaScript bridge.
