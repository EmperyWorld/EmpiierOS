use crate::command_runner;
use crate::network_control::ip_path;
use crate::observed_domains::{
    host_proc_mounts, parse_root_mount, probe_audio, probe_display, probe_input, probe_locale,
    probe_services, probe_storage, probe_usb, probe_wifi, AudioData, DisplayData, DomainRecord,
    InputData, LocaleData, ServicesData, StorageData, UsbData, WifiData,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
pub struct SnapshotProvenance {
    pub collector: &'static str,
    pub boot_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SnapshotFreshness {
    pub age_ms: u64,
    pub stale: bool,
    pub limit_ms: u64,
    pub reason: &'static str,
    pub provenance: &'static str,
}

pub const SNAPSHOT_STALE_LIMIT_MS: u64 = 8_000;

#[derive(Debug, Serialize)]
pub struct SystemSnapshot {
    pub sequence: u64,
    pub observed_at_epoch: u64,
    pub freshness: SnapshotFreshness,
    pub provenance: SnapshotProvenance,
    pub runtime: RuntimeStatus,
    pub host: HostStatus,
    pub network: NetworkStatus,
    pub power: PowerStatus,
    pub security: SecurityPosture,
    pub services: DomainRecord<ServicesData>,
    pub wifi: DomainRecord<WifiData>,
    pub display: DomainRecord<DisplayData>,
    pub input: DomainRecord<InputData>,
    pub audio: DomainRecord<AudioData>,
    pub storage: DomainRecord<StorageData>,
    pub usb: DomainRecord<UsbData>,
    pub locale: DomainRecord<LocaleData>,
    pub identity: SeatIdentity,
    pub watchdog: WatchdogStatus,
    pub capabilities: Vec<CapabilityStatus>,
    pub overall_status: String,
}

#[derive(Debug, Serialize)]
pub struct RuntimeStatus {
    pub status: &'static str,
    pub version: &'static str,
    pub transport: &'static str,
    pub uptime_seconds: u64,
    pub start_count: u64,
}

#[derive(Debug, Serialize)]
pub struct HostStatus {
    pub hostname: Option<String>,
    pub kernel_release: Option<String>,
    pub boot_id: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_count: Option<u32>,
    pub mem_total_kb: Option<u64>,
    pub mem_available_kb: Option<u64>,
    pub swap_total_kb: Option<u64>,
    pub swap_free_kb: Option<u64>,
    pub loadavg_1: Option<String>,
    pub loadavg_5: Option<String>,
    pub loadavg_15: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NetworkStatus {
    pub source: &'static str,
    pub interfaces: Vec<NetworkInterface>,
    pub default_route: Option<DefaultRoute>,
    pub dns_servers: Vec<String>,
    pub dns_source: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub kind: String,
    pub operstate: String,
    pub carrier: Option<bool>,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub speed_mbps: Option<u32>,
    pub duplex: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DefaultRoute {
    pub interface: Option<String>,
    pub gateway: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PowerSupply {
    pub name: String,
    pub kind: String,
    pub online: Option<bool>,
    pub capacity: Option<u8>,
    pub state: Option<String>,
    pub charge_now_uah: Option<u64>,
    pub charge_full_uah: Option<u64>,
    pub charge_full_design_uah: Option<u64>,
    pub voltage_now_uv: Option<u64>,
    pub current_now_ua: Option<u64>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub technology: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ThermalZone {
    pub name: String,
    pub kind: String,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct PowerStatus {
    pub source: &'static str,
    pub ac_online: Option<bool>,
    pub battery_present: bool,
    pub battery_percent: Option<u8>,
    pub battery_state: Option<String>,
    pub highest_temperature_c: Option<f32>,
    pub supplies: Vec<PowerSupply>,
    pub thermal_zones: Vec<ThermalZone>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SecurityPosture {
    pub source: &'static str,
    pub secure_boot: Option<bool>,
    pub dm_verity_root: bool,
    pub root_read_only: Option<bool>,
    pub kernel_lockdown: Option<String>,
    pub status: &'static str,
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SeatIdentity {
    pub source: &'static str,
    pub kiosk_user_observed: bool,
    pub maint_user_observed: bool,
    pub ssh_allow_users: Vec<String>,
    pub password_authentication: Option<bool>,
    pub permit_root_login: Option<bool>,
    pub pubkey_authentication: Option<bool>,
    pub kbd_interactive_authentication: Option<bool>,
    pub authentication_methods: Option<String>,
    pub kiosk_authorized_keys_present: bool,
    pub maint_authorized_keys_present: bool,
    pub temporary_sudoers_present: bool,
}

#[derive(Debug, Serialize)]
pub struct WatchdogDevice {
    pub name: String,
    pub identity: Option<String>,
    pub state: Option<String>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct WatchdogStatus {
    pub source: &'static str,
    pub present: bool,
    pub devices: Vec<WatchdogDevice>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CapabilityStatus {
    pub id: &'static str,
    pub availability: &'static str,
    pub source: &'static str,
    pub reason: &'static str,
}

pub fn snapshot(started: Instant, state_root: Option<&Path>) -> SystemSnapshot {
    let network = probe_network();
    let power = probe_power();
    let security = probe_security();
    let boot_id = read_trimmed("/proc/sys/kernel/random/boot_id");
    let capabilities = capability_statuses(&network, state_root);
    let overall_status = if security.status == "verified"
        && network.status == "connected"
        && network.error.is_none()
    {
        "verified"
    } else {
        "degraded"
    };

    SystemSnapshot {
        sequence: 0,
        observed_at_epoch: now_epoch(),
        freshness: SnapshotFreshness {
            age_ms: 0,
            stale: false,
            limit_ms: SNAPSHOT_STALE_LIMIT_MS,
            reason: "observed_now",
            provenance: "runtime-system-probe",
        },
        provenance: SnapshotProvenance {
            collector: "runtime-system-probe",
            boot_id: boot_id.clone(),
        },
        runtime: RuntimeStatus {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            transport: "unix-peercred-v1",
            uptime_seconds: started.elapsed().as_secs(),
            start_count: 0,
        },
        host: HostStatus {
            hostname: read_trimmed("/etc/hostname"),
            kernel_release: read_trimmed("/proc/sys/kernel/osrelease"),
            boot_id,
            cpu_model: cpu_model(),
            cpu_count: cpu_count(),
            mem_total_kb: meminfo_kb("MemTotal"),
            mem_available_kb: meminfo_kb("MemAvailable"),
            swap_total_kb: meminfo_kb("SwapTotal"),
            swap_free_kb: meminfo_kb("SwapFree"),
            loadavg_1: loadavg_field(0),
            loadavg_5: loadavg_field(1),
            loadavg_15: loadavg_field(2),
        },
        network,
        power,
        security,
        services: probe_services(),
        wifi: probe_wifi(),
        display: probe_display(),
        input: probe_input(),
        audio: probe_audio(),
        storage: probe_storage(),
        usb: probe_usb(),
        locale: probe_locale(),
        identity: probe_identity(),
        watchdog: probe_watchdog(),
        capabilities,
        overall_status: overall_status.to_string(),
    }
}

pub fn probe_network() -> NetworkStatus {
    let net_root = Path::new("/sys/class/net");
    if !net_root.is_dir() {
        return NetworkStatus {
            source: "linux-sysfs+iproute2",
            interfaces: Vec::new(),
            default_route: None,
            dns_servers: Vec::new(),
            dns_source: "unavailable".to_string(),
            status: "unavailable".to_string(),
            error: Some("/sys/class/net is unavailable".to_string()),
        };
    }

    let (address_map, address_error) = ip_address_map("-4", "inet");
    let (address6_map, address6_error) = ip_address_map("-6", "inet6");
    let default_route = default_route();
    let mut interfaces = Vec::new();
    if let Ok(entries) = fs::read_dir(net_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let kind = classify_link_kind(
                &name,
                path.join("wireless").exists() || path.join("phy80211").exists(),
                read_trimmed(path.join("type")).as_deref(),
            );
            interfaces.push(NetworkInterface {
                operstate: read_trimmed(path.join("operstate"))
                    .unwrap_or_else(|| "unknown".to_string()),
                carrier: read_trimmed(path.join("carrier")).map(|value| value == "1"),
                mac_address: read_trimmed(path.join("address")),
                ipv4_addresses: address_map.get(&name).cloned().unwrap_or_default(),
                ipv6_addresses: address6_map.get(&name).cloned().unwrap_or_default(),
                speed_mbps: read_trimmed(path.join("speed")).and_then(|value| parse_link_speed(&value)),
                duplex: read_trimmed(path.join("duplex")).and_then(|value| parse_duplex(&value)),
                name,
                kind: kind.to_string(),
            });
        }
    }
    interfaces.sort_by(|left, right| left.name.cmp(&right.name));

    let status = classify_network(&interfaces, default_route.as_ref());

    NetworkStatus {
        source: "linux-sysfs+iproute2",
        interfaces,
        default_route,
        dns_servers: resolv_conf_dns(),
        dns_source: classify_dns_source(),
        status: status.to_string(),
        error: address_error.or(address6_error),
    }
}

pub(crate) fn classify_link_kind(
    name: &str,
    is_wireless: bool,
    arphrd_type: Option<&str>,
) -> &'static str {
    if name == "lo" {
        return "loopback";
    }
    if is_wireless {
        return "wifi";
    }
    match arphrd_type {
        Some("1") => "ethernet",
        Some(_) => "other",
        None if looks_virtual_link(name) => "other",
        None => "ethernet",
    }
}

fn parse_link_speed(raw: &str) -> Option<u32> {
    let value = raw.trim();
    if value == "-1" || value.eq_ignore_ascii_case("unknown") {
        return None;
    }
    let speed = value.parse::<u32>().ok()?;
    (1..=400_000).contains(&speed).then_some(speed)
}

fn sanitize_ip_cidr(local: &str, prefix: u64, ipv6: bool) -> Option<String> {
    let max_prefix = if ipv6 { 128 } else { 32 };
    if prefix > max_prefix || local.is_empty() || local.len() > 64 {
        return None;
    }
    let allowed = if ipv6 {
        local
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == ':' || ch == '.')
    } else {
        local
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '.')
    };
    allowed.then(|| format!("{local}/{prefix}"))
}

fn parse_duplex(raw: &str) -> Option<String> {
    matches!(raw.trim(), "full" | "half").then(|| raw.trim().to_string())
}

fn looks_virtual_link(name: &str) -> bool {
    name.starts_with("sit")
        || name.starts_with("tun")
        || name.starts_with("tap")
        || name.starts_with("dummy")
        || name.starts_with("veth")
        || name.starts_with("virbr")
        || name.starts_with("docker")
        || name.starts_with("br-")
        || name.starts_with("wg")
        || matches!(
            name,
            "ip6tnl0" | "ip6gre0" | "gre0" | "gretap0" | "erspan0" | "teql0"
        )
}

/// Product taskbar chip. Same order as the shell tray: associated Wi-Fi, then a
/// carrier-up ethernet. No addresses, names, or SSIDs.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct NetworkLinkChip {
    pub ok: bool,
    pub kind: Option<String>,
    pub active: bool,
}

fn link_physically_up(iface: &NetworkInterface) -> bool {
    iface.operstate.eq_ignore_ascii_case("up") || iface.carrier == Some(true)
}

pub fn network_link_chip(network: &NetworkStatus, wifi_associated: bool) -> NetworkLinkChip {
    let ethernet = network
        .interfaces
        .iter()
        .find(|iface| iface.kind == "ethernet");
    let wifi = network
        .interfaces
        .iter()
        .find(|iface| iface.kind == "wifi");
    if wifi_associated {
        return NetworkLinkChip {
            ok: true,
            kind: Some("wifi".to_string()),
            active: true,
        };
    }
    if ethernet.is_some_and(link_physically_up) {
        return NetworkLinkChip {
            ok: true,
            kind: Some("ethernet".to_string()),
            active: true,
        };
    }
    if wifi.is_some_and(link_physically_up) {
        return NetworkLinkChip {
            ok: true,
            kind: Some("wifi".to_string()),
            active: true,
        };
    }
    if ethernet.is_some() {
        return NetworkLinkChip {
            ok: true,
            kind: Some("ethernet".to_string()),
            active: false,
        };
    }
    if wifi.is_some() {
        return NetworkLinkChip {
            ok: true,
            kind: Some("wifi".to_string()),
            active: false,
        };
    }
    NetworkLinkChip {
        ok: false,
        kind: None,
        active: false,
    }
}

fn classify_network(
    interfaces: &[NetworkInterface],
    default_route: Option<&DefaultRoute>,
) -> &'static str {
    let routed_interface = default_route.and_then(|route| route.interface.as_deref());
    if routed_interface.is_some_and(|name| {
        interfaces
            .iter()
            .any(|iface| iface.name == name && !iface.ipv4_addresses.is_empty())
    }) {
        "connected"
    } else if interfaces
        .iter()
        .any(|iface| iface.kind != "loopback" && !iface.ipv4_addresses.is_empty())
    {
        "local"
    } else if interfaces.iter().any(|iface| iface.kind != "loopback") {
        "disconnected"
    } else {
        "unavailable"
    }
}

fn ip_address_map(
    family_flag: &'static str,
    family: &'static str,
) -> (HashMap<String, Vec<String>>, Option<String>) {
    let Some(ip) = ip_path() else {
        return (
            HashMap::new(),
            Some("iproute2 executable is unavailable".to_string()),
        );
    };
    let result = match command_runner::run(
        ip,
        &["-j", family_flag, "address", "show"],
        Duration::from_secs(2),
    ) {
        Ok(result) if result.success => result,
        Ok(result) => {
            return (
                HashMap::new(),
                Some(format!(
                    "ip address failed: {}",
                    result.stderr.trim().replace(['\r', '\n'], " ")
                )),
            );
        }
        Err(error) => return (HashMap::new(), Some(error)),
    };

    let Ok(rows) = serde_json::from_str::<Vec<Value>>(&result.stdout) else {
        return (
            HashMap::new(),
            Some("ip address returned invalid JSON".to_string()),
        );
    };
    let mut map = HashMap::new();
    for row in rows {
        let Some(name) = row.get("ifname").and_then(Value::as_str) else {
            continue;
        };
        let addresses = row
            .get("addr_info")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|item| item.get("family").and_then(Value::as_str) == Some(family))
            .filter_map(|item| {
                let local = item.get("local").and_then(Value::as_str)?;
                let prefix = item.get("prefixlen").and_then(Value::as_u64)?;
                sanitize_ip_cidr(local, prefix, family == "inet6")
            })
            .collect::<Vec<_>>();
        map.insert(name.to_string(), addresses);
    }
    (map, None)
}

fn default_route() -> Option<DefaultRoute> {
    let ip = ip_path()?;
    let result = command_runner::run(
        ip,
        &["-j", "-4", "route", "show", "default"],
        Duration::from_secs(2),
    )
    .ok()?;
    if !result.success {
        return None;
    }
    let rows = serde_json::from_str::<Vec<Value>>(&result.stdout).ok()?;
    let row = rows.first()?;
    Some(DefaultRoute {
        interface: row.get("dev").and_then(Value::as_str).map(str::to_string),
        gateway: row
            .get("gateway")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn classify_dns_source() -> String {
    classify_dns_source_from_link(fs::read_link("/etc/resolv.conf").ok().as_deref())
}

pub(crate) fn classify_dns_source_from_link(target: Option<&Path>) -> String {
    match target.and_then(Path::to_str) {
        Some(path) if resolv_target_kind(path) == Some("uplink") => {
            "systemd-resolved-uplink".to_string()
        }
        Some(path) if resolv_target_kind(path) == Some("stub") => {
            "systemd-resolved-stub".to_string()
        }
        Some(_) => "unrecognized".to_string(),
        None if Path::new("/etc/resolv.conf").is_file() => "static-resolv-conf".to_string(),
        None => "unavailable".to_string(),
    }
}

fn resolv_target_kind(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().trim_start_matches("./");
    let leaf = normalized.rsplit('/').next().unwrap_or(normalized);
    if (normalized.ends_with("run/systemd/resolve/resolv.conf")
        || normalized == "../run/systemd/resolve/resolv.conf")
        && leaf == "resolv.conf"
    {
        return Some("uplink");
    }
    if normalized.ends_with("run/systemd/resolve/stub-resolv.conf")
        || normalized == "../run/systemd/resolve/stub-resolv.conf"
        || leaf == "stub-resolv.conf"
    {
        return Some("stub");
    }
    None
}

fn resolv_conf_dns() -> Vec<String> {
    fs::read_to_string("/etc/resolv.conf")
        .ok()
        .into_iter()
        .flat_map(|contents| parse_resolv_nameservers(&contents))
        .collect()
}

pub(crate) fn parse_resolv_nameservers(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|line| line.trim().strip_prefix("nameserver "))
        .filter_map(|value| sanitize_nameserver(value.trim()))
        .take(8)
        .collect()
}

pub(crate) fn sanitize_nameserver(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > 64 || value.contains('%') {
        return None;
    }
    let v4 = value.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && value.contains('.');
    let v6 = value.chars().all(|ch| ch.is_ascii_hexdigit() || ch == ':' || ch == '.')
        && value.contains(':');
    (v4 || v6).then(|| value.to_string())
}

pub fn probe_power() -> PowerStatus {
    let supplies = probe_power_supplies();
    let mut ac_online = None;
    let mut battery_present = false;
    let mut battery_percent = None;
    let mut battery_state = None;
    for supply in &supplies {
        if supply.kind == "mains" && ac_online.is_none() {
            ac_online = supply.online;
        } else if supply.kind == "battery" && !battery_present {
            battery_present = true;
            battery_percent = supply.capacity;
            battery_state = supply.state.clone();
        }
    }

    let thermal_zones = probe_thermal_zones();
    let highest_temperature_c = thermal_zones
        .iter()
        .filter_map(|zone| zone.temperature_c)
        .max_by(f32::total_cmp);

    let status = if !supplies.is_empty() || !thermal_zones.is_empty() {
        "observed"
    } else {
        "unavailable"
    };
    PowerStatus {
        source: "linux-sysfs",
        ac_online,
        battery_present,
        battery_percent,
        battery_state,
        highest_temperature_c,
        supplies,
        thermal_zones,
        status: status.to_string(),
    }
}

fn probe_power_supplies() -> Vec<PowerSupply> {
    let mut supplies = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/power_supply") else {
        return supplies;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_sysfs_ident(&name) {
            continue;
        }
        let kind = read_trimmed(entry.path().join("type"))
            .unwrap_or_default()
            .to_ascii_lowercase();
        if kind.is_empty() {
            continue;
        }
        let path = entry.path();
        supplies.push(PowerSupply {
            name,
            kind,
            online: read_trimmed(path.join("online")).map(|value| value == "1"),
            capacity: read_trimmed(path.join("capacity"))
                .and_then(|value| value.parse::<u8>().ok())
                .map(|value| value.min(100)),
            state: read_trimmed(path.join("status")).map(|value| value.to_ascii_lowercase()),
            charge_now_uah: read_sysfs_u64(&path.join("charge_now"), 100_000_000),
            charge_full_uah: read_sysfs_u64(&path.join("charge_full"), 100_000_000),
            charge_full_design_uah: read_sysfs_u64(&path.join("charge_full_design"), 100_000_000),
            voltage_now_uv: read_sysfs_u64(&path.join("voltage_now"), 30_000_000),
            current_now_ua: read_sysfs_u64(&path.join("current_now"), 10_000_000),
            manufacturer: read_trimmed(path.join("manufacturer")).and_then(|value| sanitize_supply_text(&value)),
            model: read_trimmed(path.join("model_name")).and_then(|value| sanitize_supply_text(&value)),
            technology: read_trimmed(path.join("technology")).and_then(|value| sanitize_supply_text(&value)),
        });
    }
    supplies.sort_by(|left, right| left.name.cmp(&right.name));
    supplies
}

fn probe_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/thermal") else {
        return zones;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_thermal_zone_name(&name) {
            continue;
        }
        let kind = read_trimmed(entry.path().join("type")).unwrap_or_else(|| "unknown".to_string());
        zones.push(ThermalZone {
            name,
            kind,
            temperature_c: read_trimmed(entry.path().join("temp")).and_then(|value| parse_milli_celsius(&value)),
        });
    }
    zones.sort_by(|left, right| left.name.cmp(&right.name));
    zones
}

fn is_thermal_zone_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("thermal_zone") else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_sysfs_ident(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn parse_milli_celsius(value: &str) -> Option<f32> {
    let raw = value.trim().parse::<f32>().ok()?;
    let celsius = if raw.abs() > 1000.0 { raw / 1000.0 } else { raw };
    (celsius.is_finite() && (-40.0..=200.0).contains(&celsius)).then_some(celsius)
}

fn probe_identity() -> SeatIdentity {
    let (settings, source) = probe_sshd_settings();
    SeatIdentity {
        source,
        kiosk_user_observed: passwd_has_user("eliteew-ts"),
        maint_user_observed: passwd_has_user("empiieros-maint"),
        ssh_allow_users: settings.allow_users,
        password_authentication: settings.password_authentication,
        permit_root_login: settings.permit_root_login,
        pubkey_authentication: settings.pubkey_authentication,
        kbd_interactive_authentication: settings.kbd_interactive_authentication,
        authentication_methods: settings.authentication_methods,
        kiosk_authorized_keys_present: authorized_keys_present(Path::new("/home/eliteew-ts")),
        maint_authorized_keys_present: authorized_keys_present(Path::new("/home/empiieros-maint")),
        temporary_sudoers_present: regular_file_exists(Path::new(
            "/etc/sudoers.d/99-temporary-maintenance",
        )),
    }
}

fn probe_watchdog() -> WatchdogStatus {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/watchdog") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_watchdog_name(&name) {
                continue;
            }
            devices.push(WatchdogDevice {
                name,
                identity: read_trimmed(entry.path().join("identity")),
                state: read_trimmed(entry.path().join("state")),
                timeout_seconds: read_trimmed(entry.path().join("timeout"))
                    .and_then(|value| value.parse().ok()),
            });
        }
    }
    devices.sort_by(|left, right| left.name.cmp(&right.name));
    WatchdogStatus {
        source: "linux-sysfs",
        present: !devices.is_empty(),
        status: if devices.is_empty() {
            "unavailable".to_string()
        } else {
            "observed".to_string()
        },
        devices,
    }
}

fn is_watchdog_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("watchdog") else {
        return false;
    };
    rest.is_empty() || rest.bytes().all(|byte| byte.is_ascii_digit())
}

fn passwd_has_user(name: &str) -> bool {
    fs::read_to_string("/etc/passwd")
        .ok()
        .is_some_and(|text| {
            text.lines().any(|line| line.split_once(':').is_some_and(|(user, _)| user == name))
        })
}

fn regular_file_exists(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .ok()
        .is_some_and(|meta| meta.file_type().is_file())
}

fn authorized_keys_present(home: &Path) -> bool {
    let ssh_dir = home.join(".ssh");
    match fs::symlink_metadata(&ssh_dir) {
        Ok(meta) if meta.file_type().is_dir() => {
            regular_file_exists(&ssh_dir.join("authorized_keys"))
        }
        _ => false,
    }
}

struct SshdSettings {
    allow_users: Vec<String>,
    password_authentication: Option<bool>,
    permit_root_login: Option<bool>,
    pubkey_authentication: Option<bool>,
    kbd_interactive_authentication: Option<bool>,
    authentication_methods: Option<String>,
}

fn probe_sshd_settings() -> (SshdSettings, &'static str) {
    if let Some(text) = sshd_effective_text() {
        return (parse_sshd_settings(&text), "linux-passwd+sshd-T");
    }
    (
        parse_sshd_settings(&read_sshd_config_text()),
        "linux-passwd+sshd",
    )
}

fn sshd_effective_text() -> Option<String> {
    let result = crate::command_runner::run(
        "/usr/sbin/sshd",
        &["-T"],
        Duration::from_secs(2),
    )
    .ok()?;
    if !result.success || result.stdout.trim().is_empty() {
        return None;
    }
    Some(result.stdout)
}

fn read_sshd_config_text() -> String {
    let mut texts = Vec::new();
    if regular_file_exists(Path::new("/etc/ssh/sshd_config")) {
        if let Ok(text) = fs::read_to_string("/etc/ssh/sshd_config") {
            texts.push(text);
        }
    }
    if let Ok(entries) = fs::read_dir("/etc/ssh/sshd_config.d") {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "conf") && regular_file_exists(path)
            })
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(text) = fs::read_to_string(path) {
                texts.push(text);
            }
        }
    }
    texts.join("\n")
}

fn parse_sshd_settings(text: &str) -> SshdSettings {
    let mut allow_users = Vec::new();
    let mut password_authentication = None;
    let mut permit_root_login = None;
    let mut pubkey_authentication = None;
    let mut kbd_interactive_authentication = None;
    let mut authentication_methods = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        match key.to_ascii_lowercase().as_str() {
            "allowusers" => {
                allow_users = value
                    .split_whitespace()
                    .filter(|name| is_ssh_user(name))
                    .map(str::to_string)
                    .collect();
            }
            "passwordauthentication" => password_authentication = parse_ssh_bool(value),
            "pubkeyauthentication" => pubkey_authentication = parse_ssh_bool(value),
            "kbdinteractiveauthentication" => {
                kbd_interactive_authentication = parse_ssh_bool(value)
            }
            "authenticationmethods" => authentication_methods = parse_ssh_auth_methods(value),
            "permitrootlogin" => {
                permit_root_login = match value.split_whitespace().next().unwrap_or("") {
                    "yes" => Some(true),
                    "no" | "prohibit-password" | "without-password" | "forced-commands-only" => {
                        Some(false)
                    }
                    _ => None,
                };
            }
            _ => {}
        }
    }
    SshdSettings {
        allow_users,
        password_authentication,
        permit_root_login,
        pubkey_authentication,
        kbd_interactive_authentication,
        authentication_methods,
    }
}

fn is_ssh_user(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some('a'..='z') | Some('_'))
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
}

fn parse_ssh_bool(value: &str) -> Option<bool> {
    match value.split_whitespace().next()? {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

fn parse_ssh_auth_methods(value: &str) -> Option<String> {
    let mut methods = Vec::new();
    for token in value.split(|ch: char| ch == ',' || ch.is_whitespace()) {
        if token.is_empty() {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        if !matches!(
            lower.as_str(),
            "publickey"
                | "password"
                | "keyboard-interactive"
                | "hostbased"
                | "gssapi-with-mic"
                | "any"
        ) {
            return None;
        }
        if !methods.contains(&lower) {
            methods.push(lower);
        }
    }
    (methods.len() <= 4).then_some(methods.join(",")).filter(|text| !text.is_empty())
}

pub fn probe_security() -> SecurityPosture {
    let secure_boot = secure_boot_state();
    let dm_verity_root = dm_verity_root();
    let root_read_only = root_mount_read_only();
    let kernel_lockdown = read_trimmed("/sys/kernel/security/lockdown");
    let mut reasons = Vec::new();

    if secure_boot != Some(true) {
        reasons.push(match secure_boot {
            Some(false) => "secure_boot_disabled".to_string(),
            None => "secure_boot_unavailable".to_string(),
            Some(true) => unreachable!(),
        });
    }
    if !dm_verity_root {
        reasons.push("dm_verity_not_observed".to_string());
    }
    if root_read_only != Some(true) {
        reasons.push("root_not_verified_read_only".to_string());
    }
    if kernel_lockdown
        .as_deref()
        .is_none_or(|value| !value.contains("[integrity]") && !value.contains("[confidentiality]"))
    {
        reasons.push("kernel_lockdown_inactive_or_unavailable".to_string());
    }

    SecurityPosture {
        source: "linux-kernel",
        secure_boot,
        dm_verity_root,
        root_read_only,
        kernel_lockdown,
        status: if reasons.is_empty() {
            "verified"
        } else {
            "degraded"
        },
        reasons,
    }
}

fn secure_boot_state() -> Option<bool> {
    let path =
        "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";
    let bytes = fs::read(path).ok()?;
    bytes.get(4).copied().map(|value| value == 1)
}

fn dm_verity_root() -> bool {
    let Some(device) = root_device_major_minor() else {
        return false;
    };
    read_trimmed(Path::new("/sys/dev/block").join(device).join("dm/uuid"))
        .is_some_and(|uuid| uuid.to_ascii_uppercase().starts_with("CRYPT-VERITY")
            || uuid.to_ascii_uppercase().starts_with("VERITY"))
}

fn root_device_major_minor() -> Option<String> {
    let mountinfo = fs::read_to_string("/proc/self/mountinfo").ok()?;
    mountinfo.lines().find_map(|line| {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        (fields.get(4) == Some(&"/"))
            .then(|| fields.get(2).map(|value| (*value).to_string()))
            .flatten()
    })
}

fn root_mount_read_only() -> Option<bool> {
    parse_root_mount(&host_proc_mounts()?).1
}

fn gate(row: crate::capability_gate::Row) -> CapabilityStatus {
    CapabilityStatus {
        id: row.id,
        availability: row.availability,
        source: row.source,
        reason: row.reason,
    }
}

fn capability_statuses(network: &NetworkStatus, state_root: Option<&Path>) -> Vec<CapabilityStatus> {
    let wifi_present = network.interfaces.iter().any(|iface| iface.kind == "wifi");
    let networkctl = Path::new("/usr/bin/networkctl").is_file()
        && ip_path().is_some()
        && Path::new("/etc/systemd/network").is_dir();
    let wifi_tools = Path::new("/usr/sbin/wpa_supplicant").is_file()
        && Path::new("/usr/sbin/iw").is_file()
        || Path::new("/usr/bin/iw").is_file()
            && Path::new("/usr/sbin/wpa_supplicant").is_file();
    vec![
        CapabilityStatus {
            id: "network.configure",
            availability: if networkctl { "available" } else { "unavailable" },
            source: "systemd-networkd",
            reason: if networkctl {
                "networkctl_observed_mutations_verified"
            } else {
                "networkctl_not_installed"
            },
        },
        CapabilityStatus {
            id: "network.wifi",
            availability: if wifi_present && wifi_tools {
                "available"
            } else {
                "unavailable"
            },
            source: "iw+wpa_supplicant",
            reason: if !wifi_present {
                "wifi_interface_not_observed"
            } else if !wifi_tools {
                "wifi_tools_not_installed"
            } else {
                "wifi_interface_and_tools_observed"
            },
        },
        {
            let helper = Path::new("/usr/local/sbin/empiieros-wifi-radio").is_file()
                && Path::new("/etc/systemd/system/empiieros-wifi-radio.service").is_file();
            let radio = Path::new("/sys/class/rfkill")
                .read_dir()
                .ok()
                .map(|entries| {
                    entries.filter_map(Result::ok).any(|entry| {
                        fs::read_to_string(entry.path().join("type"))
                            .is_ok_and(|value| value.trim() == "wlan")
                    })
                })
                .unwrap_or(false);
            CapabilityStatus {
                id: "network.wifi.radio",
                availability: if helper && radio {
                    "available"
                } else {
                    "unavailable"
                },
                source: "sysfs-rfkill",
                reason: if !helper {
                    "wifi_radio_helper_not_installed"
                } else if !radio {
                    "wifi_radio_not_observed"
                } else {
                    "allowlisted_wifi_radio_helper_observed"
                },
            }
        },
        {
            let power_units = Path::new("/etc/systemd/system/empiieros-power-reboot.service")
                .is_file()
                && Path::new("/etc/systemd/system/empiieros-power-poweroff.service").is_file();
            CapabilityStatus {
                id: "system.power",
                availability: if power_units { "available" } else { "unavailable" },
                source: "systemd-isolate",
                reason: if power_units {
                    "allowlisted_power_units_observed"
                } else {
                    "power_units_not_installed"
                },
            }
        },
        unavailable("security.integrity", "not_implemented"),
        unavailable("operator.auth", "not_implemented"),
        unavailable("update.apply", "not_implemented"),
        unavailable("factory.reset", "not_implemented"),
        gate(crate::capability_gate::product_open(state_root)),
        CapabilityStatus {
            id: "seat.browser",
            availability: "available",
            source: "seat-browser-overlay",
            reason: "empiieros_seat_browser_overlay",
        },
        gate(crate::capability_gate::launcher_status()),
        gate(crate::capability_gate::capsule_import(state_root)),
        gate(crate::capability_gate::air_activate(state_root)),
        gate(crate::capability_gate::pin_issue()),
        gate(crate::capability_gate::pin_unlock(state_root)),
        gate(crate::capability_gate::channel_up(state_root)),
        gate(crate::capability_gate::hostmap_apply(state_root)),
        gate(crate::capability_gate::spa_sync(state_root)),
        gate(crate::capability_gate::edge_health(state_root)),
        gate(crate::capability_gate::ota_status(state_root)),
        gate(crate::capability_gate::ota_request(state_root)),
        gate(crate::capability_gate::ota_apply(state_root)),
        gate(crate::capability_gate::oidc_start(state_root)),
        gate(crate::capability_gate::oidc_logout(state_root)),
        {
            let helper = Path::new("/usr/local/sbin/empiieros-audio-volume").is_file()
                && Path::new("/etc/systemd/system/empiieros-audio-volume.service").is_file();
            let card = Path::new("/proc/asound/card0").is_dir();
            CapabilityStatus {
                id: "audio.control",
                availability: if helper && card {
                    "available"
                } else {
                    "unavailable"
                },
                source: "alsa-mixer",
                reason: if !helper {
                    "audio_helper_not_installed"
                } else if !card {
                    "audio_card_not_observed"
                } else {
                    "allowlisted_audio_helper_observed"
                },
            }
        },
        {
            let helper = Path::new("/usr/local/sbin/empiieros-display-brightness").is_file()
                && Path::new("/etc/systemd/system/empiieros-display-brightness.service").is_file();
            let backlight = Path::new("/sys/class/backlight")
                .read_dir()
                .ok()
                .map(|entries| entries.filter_map(Result::ok).count())
                .unwrap_or(0)
                > 0;
            CapabilityStatus {
                id: "display.control",
                availability: if helper && backlight {
                    "available"
                } else {
                    "unavailable"
                },
                source: "sysfs-backlight",
                reason: if !helper {
                    "backlight_helper_not_installed"
                } else if !backlight {
                    "backlight_not_observed"
                } else {
                    "allowlisted_backlight_helper_observed"
                },
            }
        },
        gate(crate::capability_gate::time_sync_row()),
        unavailable("offline.mount", "not_implemented"),
        {
            let present = Path::new("/sys/class/watchdog")
                .read_dir()
                .ok()
                .is_some_and(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .any(|entry| is_watchdog_name(&entry.file_name().to_string_lossy()))
                });
            unavailable(
                "hardware.watchdog",
                if present {
                    "watchdog_mutation_not_implemented"
                } else {
                    "watchdog_not_observed"
                },
            )
        },
    ]
}

fn unavailable(id: &'static str, reason: &'static str) -> CapabilityStatus {
    CapabilityStatus {
        id,
        availability: "unavailable",
        source: "secure-reset-ledger",
        reason,
    }
}

fn cpu_model() -> Option<String> {
    parse_cpu_model(&fs::read_to_string("/proc/cpuinfo").ok()?)
}

fn cpu_count() -> Option<u32> {
    parse_cpu_count(&fs::read_to_string("/proc/cpuinfo").ok()?)
}

fn meminfo_kb(key: &str) -> Option<u64> {
    parse_meminfo_kb(&fs::read_to_string("/proc/meminfo").ok()?, key)
}

fn loadavg_field(index: usize) -> Option<String> {
    parse_loadavg_field(&fs::read_to_string("/proc/loadavg").ok()?, index)
}

fn parse_cpu_model(raw: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        let value = line.strip_prefix("model name")?;
        let value = value.trim().trim_start_matches(':').trim();
        if value.is_empty() || value.len() > 96 {
            return None;
        }
        value
            .chars()
            .all(|ch| ch.is_ascii_graphic() || ch == ' ')
            .then(|| value.to_string())
    })
}

fn parse_cpu_count(raw: &str) -> Option<u32> {
    let count = raw
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count() as u32;
    (count > 0).then_some(count)
}

fn parse_meminfo_kb(raw: &str, key: &str) -> Option<u64> {
    raw.lines().find_map(|line| {
        let (name, rest) = line.split_once(':')?;
        (name == key).then(|| rest.trim().split_whitespace().next()?.parse().ok())?
    })
}

pub(crate) fn parse_loadavg_field(raw: &str, index: usize) -> Option<String> {
    let value = raw.split_whitespace().nth(index)?.trim();
    if value.is_empty() || value.len() > 16 {
        return None;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.')
        .then(|| value.to_string())
}

fn read_sysfs_u64(path: &Path, max: u64) -> Option<u64> {
    read_trimmed(path).and_then(|value| parse_sysfs_u64(&value, max))
}

fn parse_sysfs_u64(raw: &str, max: u64) -> Option<u64> {
    let value = raw.trim().parse::<u64>().ok()?;
    (value <= max).then_some(value)
}

fn sanitize_supply_text(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > 64 {
        return None;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_graphic() || ch == ' ')
        .then(|| value.to_string())
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_proc_parsers_keep_observed_numbers() {
        assert_eq!(
            parse_cpu_model("processor\t: 0\nmodel name\t: Intel(R) Core(TM) i7-10850H CPU @ 2.70GHz\n"),
            Some("Intel(R) Core(TM) i7-10850H CPU @ 2.70GHz".to_string())
        );
        assert_eq!(parse_cpu_count("processor\t: 0\nprocessor\t: 1\n"), Some(2));
        assert_eq!(
            parse_meminfo_kb("MemTotal:       32445560 kB\nMemAvailable:   31790284 kB\n", "MemTotal"),
            Some(32445560)
        );
        assert_eq!(parse_loadavg_field("0.22 0.15 0.08 1/239 3976\n", 0).as_deref(), Some("0.22"));
        assert_eq!(parse_loadavg_field("0.22 0.15 0.08 1/239 3976\n", 1).as_deref(), Some("0.15"));
        assert_eq!(parse_loadavg_field("0.22 0.15 0.08 1/239 3976\n", 2).as_deref(), Some("0.08"));
        assert!(parse_loadavg_field("bad", 0).is_none());
        assert_eq!(
            parse_meminfo_kb("SwapTotal:             0 kB\nSwapFree:              0 kB\n", "SwapTotal"),
            Some(0)
        );
        assert_eq!(
            parse_meminfo_kb("SwapFree:              0 kB\n", "SwapFree"),
            Some(0)
        );
        assert_eq!(
            classify_dns_source_from_link(Some(Path::new(
                "../run/systemd/resolve/resolv.conf"
            ))),
            "systemd-resolved-uplink"
        );
        assert_eq!(
            classify_dns_source_from_link(Some(Path::new(
                "/run/systemd/resolve/stub-resolv.conf"
            ))),
            "systemd-resolved-stub"
        );
        assert_eq!(
            parse_resolv_nameservers(
                "nameserver 192.168.8.9\nnameserver 1.1.1.1\nnameserver 8.8.8.8\n"
            ),
            vec!["192.168.8.9", "1.1.1.1", "8.8.8.8"]
        );
        assert!(sanitize_nameserver("192.168.8.9;id").is_none());
        assert!(sanitize_nameserver("fe80::1%eno1").is_none());
    }

    #[test]
    fn seat_browser_capability_is_advertised() {
        let snap = snapshot(Instant::now(), None);
        let cap = snap
            .capabilities
            .iter()
            .find(|item| item.id == "seat.browser")
            .expect("seat.browser row");
        assert_eq!(cap.availability, "available");
        assert_eq!(cap.source, "seat-browser-overlay");
        assert_eq!(cap.reason, "empiieros_seat_browser_overlay");
        assert!(
            !snap.capabilities.iter().any(|item| item.id == "seat.browser.*"),
            "advertise the family id, not a glob"
        );
    }

    #[test]
    fn product_open_and_clock_follow_real_gates() {
        let snap = snapshot(Instant::now(), None);
        let product = snap
            .capabilities
            .iter()
            .find(|item| item.id == "product.open")
            .expect("product.open row");
        assert_eq!(product.availability, "unavailable");
        assert_eq!(product.reason, "pin_not_issued");
        let clock = snap
            .capabilities
            .iter()
            .find(|item| item.id == "time.sync")
            .expect("time.sync row");
        assert_eq!(clock.source, "operator-wall-clock");
        assert!(!clock.reason.contains("timesyncd"));
        let ota = snap
            .capabilities
            .iter()
            .find(|item| item.id == "launcher.ota_request")
            .expect("launcher.ota_request row");
        assert_eq!(ota.availability, "unavailable");
        assert_eq!(ota.reason, "pin_not_issued");
    }

    #[test]
    fn unavailable_capability_is_never_green() {
        let capability = unavailable("example", "not implemented");
        assert_eq!(capability.availability, "unavailable");
        assert_ne!(capability.availability, "available");
        let product = unavailable("product.remote", "not_implemented");
        assert_eq!(product.id, "product.remote");
        assert_eq!(product.availability, "unavailable");
        assert_eq!(product.source, "secure-reset-ledger");
    }

    #[test]
    fn security_probe_has_no_verified_default() {
        let posture = probe_security();
        if posture.status == "verified" {
            assert_eq!(posture.secure_boot, Some(true));
            assert!(posture.dm_verity_root);
            assert_eq!(posture.root_read_only, Some(true));
            assert!(posture.reasons.is_empty());
        } else {
            assert!(!posture.reasons.is_empty());
        }
    }

    #[test]
    fn sit_and_unknown_arphrd_are_not_ethernet() {
        assert_eq!(classify_link_kind("sit0", false, Some("776")), "other");
        assert_eq!(classify_link_kind("eno1", false, Some("1")), "ethernet");
        assert_eq!(classify_link_kind("wlp58s0", true, Some("1")), "wifi");
        assert_eq!(classify_link_kind("lo", false, Some("772")), "loopback");
        assert_eq!(classify_link_kind("sit0", false, None), "other");
    }

    #[test]
    fn sshd_and_watchdog_parsers_are_strict() {
        let settings = parse_sshd_settings(
            "PermitRootLogin no\nPasswordAuthentication no\nPubkeyAuthentication yes\nKbdInteractiveAuthentication no\nAuthenticationMethods publickey\nAllowUsers eliteew-ts\n# AllowUsers root\n",
        );
        assert_eq!(settings.allow_users, vec!["eliteew-ts".to_string()]);
        assert_eq!(settings.password_authentication, Some(false));
        assert_eq!(settings.permit_root_login, Some(false));
        assert_eq!(settings.pubkey_authentication, Some(true));
        assert_eq!(settings.kbd_interactive_authentication, Some(false));
        assert_eq!(settings.authentication_methods.as_deref(), Some("publickey"));
        let effective = parse_sshd_settings(
            "permitrootlogin without-password\npasswordauthentication yes\nallowusers eliteew-ts\n",
        );
        assert_eq!(effective.password_authentication, Some(true));
        assert_eq!(effective.permit_root_login, Some(false));
        assert!(effective.authentication_methods.is_none());
        let locked = parse_sshd_settings(
            "permitrootlogin no\npasswordauthentication no\nauthenticationmethods publickey,password\nallowusers eliteew-ts\n",
        );
        assert_eq!(locked.password_authentication, Some(false));
        assert_eq!(locked.authentication_methods.as_deref(), Some("publickey,password"));
        assert!(parse_ssh_auth_methods("publickey;id").is_none());
        assert!(parse_ssh_auth_methods("publickey /etc/shadow").is_none());
        assert!(is_watchdog_name("watchdog0"));
        assert!(is_watchdog_name("watchdog"));
        assert!(!is_watchdog_name("timeleft"));
        assert!(!is_ssh_user("root;id"));
        assert!(is_ssh_user("empiieros-maint"));
    }

    #[test]
    fn thermal_zone_parser_skips_cooling_devices_and_scales_milli_c() {
        assert!(is_thermal_zone_name("thermal_zone0"));
        assert!(is_thermal_zone_name("thermal_zone11"));
        assert!(!is_thermal_zone_name("cooling_device0"));
        assert!(!is_thermal_zone_name("thermal_zone"));
        assert!(!is_thermal_zone_name("thermal_zoneX"));
        assert_eq!(parse_milli_celsius("25000"), Some(25.0));
        assert_eq!(parse_milli_celsius("44000"), Some(44.0));
        assert_eq!(parse_milli_celsius("43"), Some(43.0));
        assert_eq!(parse_milli_celsius("201000"), None);
        assert_eq!(parse_milli_celsius("-41000"), None);
        assert!(is_sysfs_ident("BAT0"));
        assert!(is_sysfs_ident("AC"));
        assert!(!is_sysfs_ident("../BAT0"));
        assert_eq!(parse_sysfs_u64("2381000", 100_000_000), Some(2_381_000));
        assert_eq!(parse_sysfs_u64("100000001", 100_000_000), None);
        assert_eq!(parse_sysfs_u64("-1", 100_000_000), None);
        assert_eq!(sanitize_supply_text("DELL 3PCVM0A").as_deref(), Some("DELL 3PCVM0A"));
        assert!(sanitize_supply_text("bad\nname").is_none());
    }

    #[test]
    fn ethernet_sysfs_speed_and_duplex_are_bounded() {
        assert_eq!(parse_link_speed("1000"), Some(1000));
        assert_eq!(parse_link_speed("2500"), Some(2500));
        assert_eq!(parse_link_speed("-1"), None);
        assert_eq!(parse_link_speed("unknown"), None);
        assert_eq!(parse_link_speed("0"), None);
        assert_eq!(parse_duplex("full").as_deref(), Some("full"));
        assert_eq!(parse_duplex("half").as_deref(), Some("half"));
        assert!(parse_duplex("auto").is_none());
        assert_eq!(
            sanitize_ip_cidr("fe80::ce48:3aff:fe73:ec75", 64, true).as_deref(),
            Some("fe80::ce48:3aff:fe73:ec75/64")
        );
        assert!(sanitize_ip_cidr("fe80::1;id", 64, true).is_none());
        assert!(sanitize_ip_cidr("fe80::1", 129, true).is_none());
        assert_eq!(
            sanitize_ip_cidr("192.168.8.10", 30, false).as_deref(),
            Some("192.168.8.10/30")
        );
    }

    fn chip_iface(name: &str, kind: &str, operstate: &str, carrier: Option<bool>) -> NetworkInterface {
        NetworkInterface {
            name: name.to_string(),
            kind: kind.to_string(),
            operstate: operstate.to_string(),
            carrier,
            mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
            ipv4_addresses: vec!["192.168.8.10/30".to_string()],
            ipv6_addresses: vec![],
            speed_mbps: None,
            duplex: None,
        }
    }

    #[test]
    fn link_chip_prefers_associated_wifi_over_up_ethernet() {
        let network = NetworkStatus {
            source: "test",
            interfaces: vec![
                chip_iface("eno1", "ethernet", "up", Some(true)),
                chip_iface("wlp58s0", "wifi", "up", Some(true)),
            ],
            default_route: None,
            dns_servers: vec!["8.8.8.8".to_string()],
            dns_source: "test".to_string(),
            status: "connected".to_string(),
            error: None,
        };
        let chip = network_link_chip(&network, true);
        assert_eq!(chip.kind.as_deref(), Some("wifi"));
        assert!(chip.active);
        let json = serde_json::to_string(&chip).expect("chip");
        assert!(!json.contains("192.168.8.10"));
        assert!(!json.contains("aa:bb"));
        assert!(!json.contains("8.8.8.8"));
    }

    #[test]
    fn link_chip_uses_up_ethernet_when_wifi_is_not_associated() {
        let network = NetworkStatus {
            source: "test",
            interfaces: vec![
                chip_iface("eno1", "ethernet", "up", Some(true)),
                chip_iface("wlp58s0", "wifi", "down", Some(false)),
            ],
            default_route: None,
            dns_servers: vec![],
            dns_source: "test".to_string(),
            status: "local".to_string(),
            error: None,
        };
        let chip = network_link_chip(&network, false);
        assert_eq!(chip.kind.as_deref(), Some("ethernet"));
        assert!(chip.active);
    }

    #[test]
    fn link_chip_names_ethernet_inactive_when_nothing_is_up() {
        let network = NetworkStatus {
            source: "test",
            interfaces: vec![chip_iface("eno1", "ethernet", "down", Some(false))],
            default_route: None,
            dns_servers: vec![],
            dns_source: "test".to_string(),
            status: "disconnected".to_string(),
            error: None,
        };
        let chip = network_link_chip(&network, false);
        assert_eq!(chip.kind.as_deref(), Some("ethernet"));
        assert!(!chip.active);
    }

    #[test]
    fn network_requires_a_routed_ipv4_interface_to_report_connected() {
        let interfaces = vec![NetworkInterface {
            name: "eth0".to_string(),
            kind: "ethernet".to_string(),
            operstate: "up".to_string(),
            carrier: Some(true),
            mac_address: None,
            ipv4_addresses: vec!["192.168.8.10/24".to_string()],
            ipv6_addresses: vec!["fe80::1/64".to_string()],
            speed_mbps: Some(1000),
            duplex: Some("full".to_string()),
        }];
        assert_eq!(classify_network(&interfaces, None), "local");
        assert_eq!(
            classify_network(
                &interfaces,
                Some(&DefaultRoute {
                    interface: Some("eth0".to_string()),
                    gateway: Some("192.168.8.1".to_string()),
                })
            ),
            "connected"
        );
        assert_eq!(
            classify_network(
                &interfaces,
                Some(&DefaultRoute {
                    interface: Some("wlan0".to_string()),
                    gateway: None,
                })
            ),
            "local"
        );
    }
}
