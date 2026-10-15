use crate::command_runner;
use crate::network_control::{ip_path, observed_ipv4, validate_interface};
use crate::state_store::atomic_write_file;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScanParams {
    interface: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectParams {
    interface: Option<String>,
    ssid: String,
    psk: Option<String>,
    #[serde(default)]
    allow_open: bool,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisconnectParams {
    interface: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AccessPoint {
    ssid: String,
    bssid: String,
    signal_dbm: Option<f32>,
    signal_percent: u8,
    frequency_mhz: Option<u32>,
    security: String,
}

pub fn scan(params: Value) -> Result<Value, String> {
    let params: ScanParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    let interface = resolve_wifi_interface(params.interface.as_deref())?;
    let iw = iw_path().ok_or_else(|| "wifi_backend_unavailable: iw is not installed".to_string())?;
    let result = command_runner::run(
        iw,
        &["dev", &interface, "scan"],
        Duration::from_secs(12),
    )
    .map_err(|error| format!("wifi_scan_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!("wifi_scan_failed: {}", command_error(&result)));
    }
    let access_points = parse_iw_scan(&result.stdout);
    Ok(json!({
        "interface": interface,
        "source": "iw",
        "observed": true,
        "access_points": access_points
    }))
}

pub fn connect(params: Value) -> Result<Value, String> {
    let params: ConnectParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    let interface = resolve_wifi_interface(params.interface.as_deref())?;
    validate_ssid(&params.ssid)?;
    let security = match params.psk.as_deref() {
        Some(psk) if !psk.is_empty() => {
            validate_psk(psk)?;
            "wpa-psk"
        }
        _ if params.allow_open => "open",
        _ => return Err("psk_required_unless_allow_open_is_true".to_string()),
    };
    if !Path::new("/usr/sbin/wpa_supplicant").is_file() {
        return Err("wifi_backend_unavailable: wpa_supplicant is not installed".to_string());
    }

    let config_dir = PathBuf::from("/etc/wpa_supplicant");
    let meta = fs::symlink_metadata(&config_dir)
        .map_err(|error| format!("wpa_config_dir_unavailable: {error}"))?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err("wpa_config_dir_is_not_secure".to_string());
    }
    if meta.uid() != 0 || meta.mode() & 0o022 != 0 {
        return Err("wpa_config_dir_has_insecure_owner_or_mode".to_string());
    }

    let country = validate_country(params.country.as_deref())?;
    let filename = format!("wpa_supplicant-{interface}.conf");
    let path = config_dir.join(&filename);
    let previous = read_previous_regular(&path)?;
    let config = render_config(
        &params.ssid,
        params.psk.as_deref().filter(|value| !value.is_empty()),
        &country,
    );
    atomic_write_file(&config_dir, &filename, config.as_bytes(), 0o600)
        .map_err(|error| format!("wpa_config_write_failed: {error}"))?;

    let unit = format!("empiieros-wifi@{interface}.service");
    if let Err(error) = restart_wifi_unit(&unit) {
        return Err(with_recovery(
            error,
            recover_previous_wifi(&config_dir, &filename, previous.as_deref(), &unit, &interface),
        ));
    }
    if let Err(error) = reconfigure_network(&interface) {
        return Err(with_recovery(
            error,
            recover_previous_wifi(&config_dir, &filename, previous.as_deref(), &unit, &interface),
        ));
    }

    if !wait_for_association(&interface, &params.ssid, Duration::from_secs(20)) {
        return Err(with_recovery(
            "wifi_verification_failed: association with requested SSID was not observed"
                .to_string(),
            recover_previous_wifi(&config_dir, &filename, previous.as_deref(), &unit, &interface),
        ));
    }
    // Association proved the PSK. Kick DHCP after carrier; do not roll
    // the profile back if the lease is merely late.
    if let Err(error) = kick_dhcp(&interface) {
        return Err(error);
    }
    if !wait_for_observed_ipv4(&interface, Duration::from_secs(20)) {
        return Err(
            "wifi_verification_failed: associated but IPv4 address not observed".to_string(),
        );
    }

    Ok(json!({
        "interface": interface,
        "ssid": params.ssid,
        "security": security,
        "associated": true,
        "observed_ipv4": observed_ipv4(&interface),
        "source": "wpa_supplicant+systemd-networkd"
    }))
}

pub fn disconnect(params: Value) -> Result<Value, String> {
    let params: DisconnectParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    let interface = resolve_wifi_interface(params.interface.as_deref())?;
    let unit = format!("empiieros-wifi@{interface}.service");
    stop_wifi_unit(&unit)?;
    reconfigure_network(&interface)?;

    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(5) {
        if !carrier_up(&interface) {
            return Ok(json!({
                "interface": interface,
                "associated": false,
                "source": "wpa_supplicant"
            }));
        }
        thread::sleep(Duration::from_millis(200));
    }
    Err("wifi_disconnect_verification_failed".to_string())
}

pub fn reconnect(params: Value) -> Result<Value, String> {
    let params: DisconnectParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    let interface = resolve_wifi_interface(params.interface.as_deref())?;
    if !Path::new("/usr/sbin/wpa_supplicant").is_file() {
        return Err("wifi_backend_unavailable: wpa_supplicant is not installed".to_string());
    }
    persist_config_is_usable(&persist_config_path(&interface))?;

    let unit = format!("empiieros-wifi@{interface}.service");
    restart_wifi_unit(&unit)?;
    reconfigure_network(&interface)?;

    if !wait_for_any_association(&interface, Duration::from_secs(20)) {
        return Err(
            "wifi_verification_failed: association with requested SSID was not observed"
                .to_string(),
        );
    }
    kick_dhcp(&interface)?;
    if !wait_for_observed_ipv4(&interface, Duration::from_secs(20)) {
        return Err(
            "wifi_verification_failed: associated but IPv4 address not observed".to_string(),
        );
    }

    Ok(json!({
        "interface": interface,
        "ssid": associated_ssid(&interface),
        "associated": true,
        "observed_ipv4": observed_ipv4(&interface),
        "source": "wpa_supplicant+persist"
    }))
}

fn persist_config_path(interface: &str) -> PathBuf {
    PathBuf::from("/etc/wpa_supplicant").join(format!("wpa_supplicant-{interface}.conf"))
}

fn persist_config_is_usable(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err("wifi_persist_missing: no wpa_supplicant config for this interface".to_string());
        }
        Err(error) => return Err(format!("wifi_persist_metadata_failed: {error}")),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("wifi_persist_is_not_a_regular_file".to_string());
    }
    if metadata.uid() != 0 || metadata.mode() & 0o7777 != 0o600 {
        return Err("wifi_persist_has_insecure_owner_or_mode".to_string());
    }
    Ok(())
}

fn resolve_wifi_interface(requested: Option<&str>) -> Result<String, String> {
    if let Some(interface) = requested {
        validate_interface(interface)?;
        if is_wifi_interface(interface) {
            return Ok(interface.to_string());
        }
        return Err("interface_is_not_wifi".to_string());
    }

    fs::read_dir("/sys/class/net")
        .map_err(|error| format!("network_sysfs_unavailable: {error}"))?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|interface| is_wifi_interface(interface))
        .ok_or_else(|| "wifi_interface_not_found".to_string())
}

fn is_wifi_interface(interface: &str) -> bool {
    let root = Path::new("/sys/class/net").join(interface);
    root.join("wireless").exists() || root.join("phy80211").exists()
}

fn validate_ssid(ssid: &str) -> Result<(), String> {
    let length = ssid.as_bytes().len();
    if length == 0 || length > 32 {
        return Err("ssid_length_must_be_1_to_32_bytes".to_string());
    }
    if ssid.chars().any(char::is_control) {
        return Err("ssid_contains_control_character".to_string());
    }
    Ok(())
}

fn validate_psk(psk: &str) -> Result<(), String> {
    if !(8..=63).contains(&psk.len()) {
        return Err("psk_length_must_be_8_to_63_ascii_characters".to_string());
    }
    if !psk.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
        return Err("psk_must_be_printable_ascii".to_string());
    }
    Ok(())
}

fn validate_country(country: Option<&str>) -> Result<String, String> {
    let country = country.unwrap_or("00").trim().to_ascii_uppercase();
    if country != "00"
        && (country.len() != 2 || !country.bytes().all(|byte| byte.is_ascii_uppercase()))
    {
        return Err("country_must_be_two_ascii_letters".to_string());
    }
    Ok(country)
}

fn render_config(ssid: &str, psk: Option<&str>, country: &str) -> String {
    // This image's wpa_supplicant rejects `ctrl_interface=` and `SAE`.
    // The supervised unit starts with `-c` and does not need a control socket.
    let mut config = format!(
        "update_config=0\ncountry={country}\n\nnetwork={{\n    ssid=\"{}\"\n    scan_ssid=1\n",
        escape_quoted(ssid)
    );
    if let Some(psk) = psk {
        config.push_str(&format!("    psk=\"{}\"\n", escape_quoted(psk)));
        config.push_str("    key_mgmt=WPA-PSK\n");
    } else {
        config.push_str("    key_mgmt=NONE\n");
    }
    config.push_str("}\n");
    config
}

fn escape_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn restart_wifi_unit(unit: &str) -> Result<(), String> {
    let _ = command_runner::run(
        "/usr/bin/systemctl",
        &["reset-failed", unit],
        Duration::from_secs(5),
    );
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["restart", unit],
        Duration::from_secs(10),
    )
    .map_err(|error| format!("wifi_service_spawn_failed: {error}"))?;
    if result.success {
        Ok(())
    } else {
        Err(format!("wifi_service_restart_failed: {}", command_error(&result)))
    }
}

fn stop_wifi_unit(unit: &str) -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/systemctl",
        &["stop", unit],
        Duration::from_secs(10),
    )
    .map_err(|error| format!("wifi_service_spawn_failed: {error}"))?;
    if result.success {
        Ok(())
    } else {
        Err(format!("wifi_service_stop_failed: {}", command_error(&result)))
    }
}

fn reconfigure_network(interface: &str) -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/networkctl",
        &["reconfigure", interface],
        Duration::from_secs(8),
    )
    .map_err(|error| format!("network_reconfigure_spawn_failed: {error}"))?;
    if result.success {
        Ok(())
    } else {
        Err(format!("network_reconfigure_failed: {}", command_error(&result)))
    }
}

fn wait_for_association(interface: &str, expected_ssid: &str, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if carrier_up(interface) && associated_ssid(interface).as_deref() == Some(expected_ssid) {
            return true;
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}

fn wait_for_any_association(interface: &str, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if carrier_up(interface)
            && associated_ssid(interface).is_some_and(|ssid| !ssid.is_empty())
        {
            return true;
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}

fn wait_for_observed_ipv4(interface: &str, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if !observed_ipv4(interface).is_empty() {
            return true;
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}

fn flush_ipv4(interface: &str) {
    let Some(ip) = ip_path() else {
        return;
    };
    let _ = command_runner::run(
        ip,
        &["-4", "addr", "flush", "dev", interface, "scope", "global"],
        Duration::from_secs(3),
    );
}

fn kick_dhcp(interface: &str) -> Result<(), String> {
    flush_ipv4(interface);
    reconfigure_network(interface)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WifiLink {
    pub ssid: String,
    pub bssid: Option<String>,
    pub frequency_mhz: Option<u32>,
    pub signal_dbm: Option<i16>,
    pub rx_mbps: Option<u32>,
    pub tx_mbps: Option<u32>,
}

pub(crate) fn associated_ssid(interface: &str) -> Option<String> {
    associated_link(interface).map(|link| link.ssid)
}

pub(crate) fn associated_link(interface: &str) -> Option<WifiLink> {
    let iw = iw_path()?;
    let result = command_runner::run(
        iw,
        &["dev", interface, "link"],
        Duration::from_secs(2),
    )
    .ok()?;
    if !result.success {
        return None;
    }
    parse_iw_link(&result.stdout)
}

fn parse_iw_link(output: &str) -> Option<WifiLink> {
    let mut ssid = None;
    let mut bssid = None;
    let mut frequency_mhz = None;
    let mut signal_dbm = None;
    let mut rx_mbps = None;
    let mut tx_mbps = None;
    for line in output.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("Connected to ") {
            bssid = rest
                .split_whitespace()
                .next()
                .and_then(parse_bssid);
        } else if let Some(rest) = line.strip_prefix("SSID: ") {
            if !rest.is_empty() && rest.len() <= 32 && !rest.contains('\n') {
                ssid = Some(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("freq: ") {
            frequency_mhz = parse_link_frequency(rest);
        } else if let Some(rest) = line.strip_prefix("signal: ") {
            signal_dbm = parse_link_signal(rest);
        } else if let Some(rest) = line.strip_prefix("rx bitrate: ") {
            rx_mbps = parse_link_bitrate(rest);
        } else if let Some(rest) = line.strip_prefix("tx bitrate: ") {
            tx_mbps = parse_link_bitrate(rest);
        }
    }
    Some(WifiLink {
        ssid: ssid?,
        bssid,
        frequency_mhz,
        signal_dbm,
        rx_mbps,
        tx_mbps,
    })
}

fn parse_bssid(raw: &str) -> Option<String> {
    let value = raw.trim();
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 6 {
        return None;
    }
    if parts.iter().all(|part| {
        part.len() == 2 && part.chars().all(|ch| ch.is_ascii_hexdigit())
    }) {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

fn parse_link_frequency(raw: &str) -> Option<u32> {
    let token = raw.split_whitespace().next()?;
    let value = if let Ok(integer) = token.parse::<u32>() {
        integer
    } else {
        let parsed = token.parse::<f32>().ok()?;
        if !parsed.is_finite() {
            return None;
        }
        parsed.round() as u32
    };
    (2400..=7125).contains(&value).then_some(value)
}

fn parse_link_signal(raw: &str) -> Option<i16> {
    let value = raw.split_whitespace().next()?.parse::<f32>().ok()?;
    let rounded = value.round() as i16;
    (-120..=0).contains(&rounded).then_some(rounded)
}

fn parse_link_bitrate(raw: &str) -> Option<u32> {
    let value = raw.split_whitespace().next()?.parse::<f32>().ok()?;
    if !value.is_finite() || value < 1.0 || value > 10_000.0 {
        return None;
    }
    Some(value.round() as u32)
}

fn carrier_up(interface: &str) -> bool {
    fs::read_to_string(Path::new("/sys/class/net").join(interface).join("carrier"))
        .is_ok_and(|value| value.trim() == "1")
}

fn iw_path() -> Option<&'static str> {
    ["/usr/sbin/iw", "/usr/bin/iw"]
        .into_iter()
        .find(|path| Path::new(path).is_file())
}

fn parse_iw_scan(output: &str) -> Vec<AccessPoint> {
    #[derive(Default)]
    struct Pending {
        ssid: Option<String>,
        bssid: String,
        signal: Option<f32>,
        frequency: Option<u32>,
        secured: bool,
        sae: bool,
    }

    fn finish(pending: &mut Pending, output: &mut Vec<AccessPoint>) {
        let Some(ssid) = pending.ssid.take() else {
            *pending = Pending::default();
            return;
        };
        if ssid.is_empty() {
            *pending = Pending::default();
            return;
        }
        let signal_percent = pending
            .signal
            .map(|dbm| ((dbm + 100.0) * 2.0).round().clamp(0.0, 100.0) as u8)
            .unwrap_or(0);
        output.push(AccessPoint {
            ssid,
            bssid: std::mem::take(&mut pending.bssid),
            signal_dbm: pending.signal,
            signal_percent,
            frequency_mhz: pending.frequency,
            security: if pending.sae {
                "wpa3".to_string()
            } else if pending.secured {
                "wpa2".to_string()
            } else {
                "open".to_string()
            },
        });
        *pending = Pending::default();
    }

    let mut access_points = Vec::new();
    let mut pending = Pending::default();
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("BSS ") {
            finish(&mut pending, &mut access_points);
            pending.bssid = rest
                .split([' ', '('])
                .next()
                .unwrap_or("")
                .to_string();
        } else if let Some(rest) = line.strip_prefix("SSID: ") {
            pending.ssid = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("signal: ") {
            pending.signal = rest
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<f32>().ok());
        } else if let Some(rest) = line.strip_prefix("freq: ") {
            pending.frequency = rest
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<u32>().ok());
        } else if line.starts_with("RSN:") || line.starts_with("WPA:") {
            pending.secured = true;
        } else if line.contains("Authentication suites: SAE") {
            pending.secured = true;
            pending.sae = true;
        }
    }
    finish(&mut pending, &mut access_points);
    access_points.sort_by(|left, right| right.signal_percent.cmp(&left.signal_percent));
    access_points.dedup_by(|left, right| left.ssid == right.ssid && left.bssid == right.bssid);
    access_points
}

fn command_error(result: &command_runner::CommandResult) -> String {
    if result.timed_out {
        return "timed out".to_string();
    }
    let detail = if result.stderr.trim().is_empty() {
        result.stdout.trim()
    } else {
        result.stderr.trim()
    };
    if detail.is_empty() {
        format!("exit {:?}", result.exit_code)
    } else {
        detail.replace(['\r', '\n'], " ")
    }
}

fn read_previous_regular(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("existing_wifi_config_metadata_failed: {error}")),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("existing_wifi_config_is_not_a_regular_file".to_string());
    }
    fs::read(path)
        .map(Some)
        .map_err(|error| format!("existing_wifi_config_read_failed: {error}"))
}

fn rollback(dir: &Path, filename: &str, previous: Option<&[u8]>) -> Result<(), String> {
    if let Some(previous) = previous {
        atomic_write_file(dir, filename, previous, 0o600)
            .map(|_| ())
            .map_err(|error| format!("restore_previous_wifi_config_failed: {error}"))
    } else {
        let path = dir.join(filename);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("rollback_wifi_config_metadata_failed: {error}"))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("rollback_refused_non_regular_wifi_config".to_string());
        }
        fs::remove_file(path).map_err(|error| format!("rollback_wifi_config_remove_failed: {error}"))
    }
}

fn recover_previous_wifi(
    dir: &Path,
    filename: &str,
    previous: Option<&[u8]>,
    unit: &str,
    interface: &str,
) -> Result<(), String> {
    rollback(dir, filename, previous)?;
    if previous.is_some() {
        restart_wifi_unit(unit)?;
    } else {
        stop_wifi_unit(unit)?;
    }
    reconfigure_network(interface)
}

fn with_recovery(primary: String, recovery: Result<(), String>) -> String {
    match recovery {
        Ok(()) => primary,
        Err(error) => format!("{primary}; recovery_failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iw_parser_returns_observed_values_only() {
        let fixture = r#"
BSS aa:bb:cc:dd:ee:ff(on wlan0)
        freq: 5180
        signal: -43.00 dBm
        SSID: FactoryNet
        RSN:
        * Authentication suites: SAE
BSS 11:22:33:44:55:66(on wlan0)
        freq: 2412
        signal: -78.00 dBm
        SSID: Guest
"#;
        let parsed = parse_iw_scan(fixture);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].ssid, "FactoryNet");
        assert_eq!(parsed[0].bssid, "aa:bb:cc:dd:ee:ff");
        assert_eq!(parsed[0].frequency_mhz, Some(5180));
        assert_eq!(parsed[0].security, "wpa3");
        assert_eq!(parsed[1].bssid, "11:22:33:44:55:66");
        assert_eq!(parsed[1].security, "open");
    }

    #[test]
    fn iw_link_parser_keeps_observed_association() {
        let fixture = r#"
Connected to 52:ff:20:aa:80:99 (on wlp58s0)
	SSID: EWKaska
	freq: 5180
	RX: 123 bytes (4 packets)
	TX: 456 bytes (2 packets)
	signal: -47 dBm
	rx bitrate: 866.7 MBit/s
	tx bitrate: 650.0 MBit/s
"#;
        let parsed = parse_iw_link(fixture).expect("link");
        assert_eq!(parsed.ssid, "EWKaska");
        assert_eq!(parsed.bssid.as_deref(), Some("52:ff:20:aa:80:99"));
        assert_eq!(parsed.frequency_mhz, Some(5180));
        assert_eq!(parsed.signal_dbm, Some(-47));
        assert_eq!(parsed.rx_mbps, Some(867));
        assert_eq!(parsed.tx_mbps, Some(650));
        assert!(parse_iw_link("Not connected.\n").is_none());
        assert!(parse_bssid("zz:zz:zz:zz:zz:zz").is_none());
        assert_eq!(parse_link_frequency("2412"), Some(2412));
        assert_eq!(parse_link_frequency("5260.0"), Some(5260));
        assert!(parse_link_frequency("12").is_none());
        assert_eq!(parse_link_signal("-47.00 dBm"), Some(-47));
        assert!(parse_link_signal("5 dBm").is_none());
    }

    #[test]
    fn config_escapes_quotes_and_country_validation_is_strict() {
        let config = render_config("Net\"One", Some("correct horse"), "US");
        assert!(config.contains("ssid=\"Net\\\"One\""));
        assert!(config.contains("psk=\"correct horse\""));
        assert!(config.contains("key_mgmt=WPA-PSK\n"));
        assert!(config.contains("scan_ssid=1\n"));
        assert!(!config.contains("ctrl_interface"));
        assert!(!config.contains("SAE"));
        assert_eq!(validate_country(Some("us")).expect("country"), "US");
        assert_eq!(validate_country(None).expect("world"), "00");
        assert!(validate_country(Some("0A")).is_err());
    }

    #[test]
    fn validation_rejects_short_psk_and_control_ssid() {
        assert!(validate_psk("short").is_err());
        assert!(validate_ssid("bad\nssid").is_err());
    }

    #[test]
    fn link_verification_requires_the_observed_ssid() {
        let linked = "Connected to aa:bb:cc:dd:ee:ff\n\tSSID: FactoryNet\n";
        assert_eq!(parse_iw_link(linked).map(|link| link.ssid).as_deref(), Some("FactoryNet"));
        assert_eq!(parse_iw_link("Not connected.\n"), None);
    }

    #[test]
    fn previous_wifi_config_reader_refuses_symlinks() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "empiieros-wifi-previous-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        let target = root.join("target");
        fs::write(&target, b"secret").expect("target");
        let link = root.join("config");
        symlink(&target, &link).expect("link");
        assert!(read_previous_regular(&link).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn persist_config_refuses_missing_and_world_readable_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let root = std::env::temp_dir().join(format!(
            "empiieros-wifi-persist-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        let missing = root.join("missing.conf");
        assert!(
            persist_config_is_usable(&missing)
                .expect_err("missing")
                .contains("wifi_persist_missing")
        );

        let target = root.join("target");
        fs::write(&target, b"psk=secret").expect("target");
        let link = root.join("link.conf");
        symlink(&target, &link).expect("link");
        assert_eq!(
            persist_config_is_usable(&link).expect_err("symlink"),
            "wifi_persist_is_not_a_regular_file"
        );

        let world = root.join("world.conf");
        fs::write(&world, b"psk=secret").expect("world");
        fs::set_permissions(&world, fs::Permissions::from_mode(0o644)).expect("mode");
        assert_eq!(
            persist_config_is_usable(&world).expect_err("world"),
            "wifi_persist_has_insecure_owner_or_mode"
        );
        let _ = fs::remove_dir_all(root);
    }
}
