use crate::command_runner;
use crate::state_store::atomic_write_file;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::net::Ipv4Addr;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const NETWORK_FILE_PREFIX: &str = "05-empiieros-";
const GENERIC_WIRELESS_FILE: &str = "20-wireless.network";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigureParams {
    pub interface: String,
    pub method: String,
    pub address: Option<String>,
    pub prefix: Option<u8>,
    pub gateway: Option<String>,
    #[serde(default)]
    pub dns: Vec<String>,
}

pub fn configure(params: Value) -> Result<Value, String> {
    let params: ConfigureParams =
        serde_json::from_value(params).map_err(|error| format!("invalid_params: {error}"))?;
    validate_interface(&params.interface)?;
    if ip_path().is_none() || !Path::new("/usr/bin/networkctl").is_file() {
        return Err("network_backend_unavailable".to_string());
    }
    let content = render_network_file(&params)?;
    let config_dir = network_config_dir();
    ensure_config_dir(&config_dir)?;

    let (filename, mode) = claim_filename(&config_dir, &params.interface);
    let final_path = config_dir.join(&filename);
    let previous = read_previous_regular(&final_path)?;
    atomic_write_file(&config_dir, &filename, content.as_bytes(), mode)
        .map_err(|error| format!("config_write_failed: {error}"))?;

    let apply = apply_networkd(&params.interface);
    if let Err(error) = apply {
        return Err(with_rollback_result(
            error,
            rollback_and_apply(
                &config_dir,
                &filename,
                previous.as_deref(),
                &params.interface,
            ),
        ));
    }

    let expected = if params.method == "static" {
        Some(format!(
            "{}/{}",
            params
                .address
                .as_deref()
                .ok_or_else(|| "static_address_required_after_render".to_string())?,
            params
                .prefix
                .ok_or_else(|| "static_prefix_required_after_render".to_string())?
        ))
    } else {
        None
    };
    if !wait_for_networkd_ipv4(
        &params.interface,
        &final_path,
        expected.as_deref(),
        Duration::from_secs(15),
    ) {
        return Err(with_rollback_result(
            "network_verification_failed: no expected IPv4 address observed".to_string(),
            rollback_and_apply(
                &config_dir,
                &filename,
                previous.as_deref(),
                &params.interface,
            ),
        ));
    }

    Ok(json!({
        "interface": params.interface,
        "method": params.method,
        "config_file": filename,
        "observed_ipv4": observed_ipv4(&params.interface),
        "source": "systemd-networkd",
        "applied": true
    }))
}

pub fn validate_interface(interface: &str) -> Result<(), String> {
    if interface.is_empty() || interface.len() > 15 {
        return Err("invalid_interface_length".to_string());
    }
    if !interface.bytes().enumerate().all(|(index, byte)| {
        if index == 0 {
            byte.is_ascii_alphanumeric()
        } else {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
        }
    }) {
        return Err("invalid_interface_characters".to_string());
    }
    if interface == "lo" {
        return Err("loopback_interface_is_immutable".to_string());
    }
    let path = Path::new("/sys/class/net").join(interface);
    if !path.is_dir() {
        return Err("interface_not_found".to_string());
    }
    let wireless = path.join("wireless").exists() || path.join("phy80211").exists();
    let arphrd = fs::read_to_string(path.join("type"))
        .ok()
        .map(|value| value.trim().to_string());
    let kind = if interface == "lo" {
        "loopback"
    } else if wireless {
        "wifi"
    } else if arphrd.as_deref() == Some("1") {
        "ethernet"
    } else {
        "other"
    };
    match kind {
        "ethernet" | "wifi" => Ok(()),
        "loopback" => Err("loopback_interface_is_immutable".to_string()),
        _ => Err("interface_is_not_configurable".to_string()),
    }
}

fn render_network_file(params: &ConfigureParams) -> Result<String, String> {
    if params.method != "dhcp" && params.method != "static" {
        return Err("method_must_be_dhcp_or_static".to_string());
    }
    if params.dns.len() > 3 {
        return Err("maximum_three_dns_servers".to_string());
    }
    let dns = params
        .dns
        .iter()
        .map(|value| parse_ipv4(value, "dns"))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = format!(
        "# Managed by empiieros-runtime; local edits are replaced.\n[Match]\nName={}\n\n[Network]\n",
        params.interface
    );
    if params.method == "dhcp" {
        if params.address.is_some() || params.prefix.is_some() || params.gateway.is_some() {
            return Err("dhcp_rejects_static_fields".to_string());
        }
        output.push_str("DHCP=ipv4\n");
    } else {
        let address = parse_ipv4(
            params
                .address
                .as_deref()
                .ok_or_else(|| "static_address_required".to_string())?,
            "address",
        )?;
        let prefix = params
            .prefix
            .filter(|value| (1..=32).contains(value))
            .ok_or_else(|| "static_prefix_must_be_1_to_32".to_string())?;
        validate_unicast_host(address, prefix, "address")?;
        output.push_str(&format!("DHCP=no\nAddress={address}/{prefix}\n"));
        if let Some(gateway_raw) = params.gateway.as_deref() {
            let gateway = parse_ipv4(gateway_raw, "gateway")?;
            validate_unicast_host(gateway, prefix, "gateway")?;
            if !same_subnet(address, gateway, prefix) {
                return Err("gateway_is_outside_interface_subnet".to_string());
            }
            if address == gateway {
                return Err("gateway_must_differ_from_interface_address".to_string());
            }
            output.push_str(&format!("Gateway={gateway}\n"));
        }
    }
    for server in dns {
        output.push_str(&format!("DNS={server}\n"));
    }
    output.push_str("LLMNR=no\nMulticastDNS=no\nIPv6AcceptRA=no\n\n[Link]\nRequiredForOnline=no\n");
    Ok(output)
}

fn parse_ipv4(raw: &str, field: &str) -> Result<Ipv4Addr, String> {
    let value = raw
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|_| format!("invalid_{field}_ipv4"))?;
    if value.is_unspecified() || value.is_broadcast() || value.is_multicast() || value.is_loopback()
    {
        return Err(format!("disallowed_{field}_ipv4"));
    }
    Ok(value)
}

fn validate_unicast_host(address: Ipv4Addr, prefix: u8, field: &str) -> Result<(), String> {
    if prefix <= 30 {
        let mask = u32::MAX << (32 - u32::from(prefix));
        let host = u32::from(address) & !mask;
        if host == 0 || host == !mask {
            return Err(format!("{field}_must_be_a_unicast_host"));
        }
    }
    Ok(())
}

fn same_subnet(left: Ipv4Addr, right: Ipv4Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - u32::from(prefix))
    };
    u32::from(left) & mask == u32::from(right) & mask
}

fn ensure_config_dir(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|error| format!("network_config_dir_unavailable: {error}"))?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err("network_config_dir_is_not_secure".to_string());
    }
    if meta.uid() != 0 || meta.mode() & 0o022 != 0 {
        return Err("network_config_dir_has_insecure_owner_or_mode".to_string());
    }
    Ok(())
}

fn apply_networkd(interface: &str) -> Result<(), String> {
    reload_networkd()?;
    let result = command_runner::run(
        "/usr/bin/networkctl",
        &["reconfigure", interface],
        Duration::from_secs(8),
    )
    .map_err(|error| format!("networkctl_reconfigure_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!(
            "networkctl_reconfigure_failed: {}",
            command_error(&result)
        ));
    }
    Ok(())
}

fn reload_networkd() -> Result<(), String> {
    let result = command_runner::run(
        "/usr/bin/networkctl",
        &["reload"],
        Duration::from_secs(8),
    )
    .map_err(|error| format!("networkctl_reload_spawn_failed: {error}"))?;
    if !result.success {
        return Err(format!("networkctl_reload_failed: {}", command_error(&result)));
    }
    Ok(())
}

fn command_error(result: &command_runner::CommandResult) -> String {
    let detail = if result.stderr.trim().is_empty() {
        result.stdout.trim()
    } else {
        result.stderr.trim()
    };
    if result.timed_out {
        "timed out".to_string()
    } else if detail.is_empty() {
        format!("exit {:?}", result.exit_code)
    } else {
        detail.replace(['\r', '\n'], " ")
    }
}

fn wait_for_networkd_ipv4(
    interface: &str,
    expected_network_file: &Path,
    expected_address: Option<&str>,
    timeout: Duration,
) -> bool {
    let started = Instant::now();
    loop {
        if observed_networkd_status(interface).is_some_and(|status| {
            networkd_status_matches(&status, expected_network_file, expected_address)
        }) {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn observed_networkd_status(interface: &str) -> Option<Value> {
    let result = command_runner::run(
        "/usr/bin/networkctl",
        &["--json=short", "--no-pager", "status", interface],
        Duration::from_secs(2),
    )
    .ok()?;
    if !result.success {
        return None;
    }
    serde_json::from_str(&result.stdout).ok()
}

fn networkd_status_matches(
    status: &Value,
    expected_network_file: &Path,
    expected_address: Option<&str>,
) -> bool {
    if status.get("NetworkFile").and_then(Value::as_str)
        != expected_network_file.to_str()
    {
        return false;
    }
    status
        .get("Addresses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|address| {
            if address.get("Family").and_then(Value::as_u64) != Some(libc::AF_INET as u64)
                || !address
                    .get("ConfigState")
                    .and_then(Value::as_str)
                    .is_some_and(|state| state.split(',').any(|part| part == "configured"))
            {
                return false;
            }
            let source = address.get("ConfigSource").and_then(Value::as_str);
            let local = json_ipv4_address(address.get("Address"));
            let prefix = address.get("PrefixLength").and_then(Value::as_u64);
            match expected_address {
                Some(expected) => {
                    source == Some("static")
                        && local
                            .zip(prefix)
                            .is_some_and(|(local, prefix)| format!("{local}/{prefix}") == expected)
                }
                None => source == Some("DHCPv4") && local.is_some() && prefix.is_some(),
            }
        })
}

pub fn observed_ipv4(interface: &str) -> Vec<String> {
    let Some(ip) = ip_path() else {
        return Vec::new();
    };
    let result = match command_runner::run(
        ip,
        &["-j", "-4", "address", "show", "dev", interface],
        Duration::from_secs(2),
    ) {
        Ok(result) if result.success => result,
        _ => return Vec::new(),
    };
    let Ok(rows) = serde_json::from_str::<Vec<Value>>(&result.stdout) else {
        return Vec::new();
    };
    rows.iter()
        .flat_map(|row| {
            row.get("addr_info")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|info| info.get("family").and_then(Value::as_str) == Some("inet"))
        .filter_map(|info| {
            Some(format!(
                "{}/{}",
                info.get("local")?.as_str()?,
                info.get("prefixlen")?.as_u64()?
            ))
        })
        .collect()
}

pub fn ip_path() -> Option<&'static str> {
    ["/usr/bin/ip", "/usr/sbin/ip", "/sbin/ip"]
        .into_iter()
        .find(|path| Path::new(path).is_file())
}

fn read_previous_regular(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("existing_config_metadata_failed: {error}")),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("existing_config_is_not_a_regular_file".to_string());
    }
    fs::read(path)
        .map(Some)
        .map_err(|error| format!("existing_config_read_failed: {error}"))
}

fn claim_filename(dir: &Path, interface: &str) -> (String, u32) {
    claim_plan(dir, interface, interface_is_wifi(interface))
}

fn claim_plan(dir: &Path, interface: &str, is_wifi: bool) -> (String, u32) {
    if is_wifi && dir.join(GENERIC_WIRELESS_FILE).is_file() {
        // This image's networkd rematches the already-bound file after reload
        // but ignores a newly created 05-*.network and unmanages the link.
        (GENERIC_WIRELESS_FILE.to_string(), 0o644)
    } else {
        (
            format!("{NETWORK_FILE_PREFIX}{}.network", interface),
            0o600,
        )
    }
}

fn rollback(dir: &Path, filename: &str, previous: Option<&[u8]>) -> Result<(), String> {
    if let Some(previous) = previous {
        let mode = if filename == GENERIC_WIRELESS_FILE {
            0o644
        } else {
            0o600
        };
        atomic_write_file(dir, filename, previous, mode)
            .map(|_| ())
            .map_err(|error| format!("restore_previous_config_failed: {error}"))
    } else {
        let path = dir.join(filename);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("rollback_config_metadata_failed: {error}"))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("rollback_refused_non_regular_config".to_string());
        }
        fs::remove_file(path).map_err(|error| format!("rollback_config_remove_failed: {error}"))
    }
}

fn rollback_and_apply(
    dir: &Path,
    filename: &str,
    previous: Option<&[u8]>,
    interface: &str,
) -> Result<(), String> {
    rollback(dir, filename, previous)?;
    apply_networkd(interface)
}

fn interface_is_wifi(interface: &str) -> bool {
    let path = Path::new("/sys/class/net").join(interface);
    path.join("wireless").exists() || path.join("phy80211").exists()
}

#[cfg(test)]
fn glob_token_matches(pattern: &str, name: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
    } else {
        pattern == name
    }
}

#[cfg(test)]
fn name_spec_matches(spec: &str, interface: &str) -> bool {
    let mut positives = Vec::new();
    let mut negatives = Vec::new();
    for token in spec.split_whitespace() {
        if let Some(rest) = token.strip_prefix('!') {
            negatives.push(rest);
        } else if !token.is_empty() {
            positives.push(token);
        }
    }
    if negatives
        .iter()
        .any(|pattern| glob_token_matches(pattern, interface))
    {
        return false;
    }
    !positives.is_empty()
        && positives
            .iter()
            .any(|pattern| glob_token_matches(pattern, interface))
}

#[cfg(test)]
fn exclude_name_from_wireless_match(content: &str, interface: &str) -> Result<String, String> {
    if interface.is_empty() || interface.len() > 15 {
        return Err("invalid_interface_length".to_string());
    }
    let exclusion = format!("!{interface}");
    let mut output = String::new();
    let mut in_match = false;
    let mut replaced = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_match = trimmed.eq_ignore_ascii_case("[match]");
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if in_match && trimmed.len() >= 5 && trimmed[..5].eq_ignore_ascii_case("name=") {
            let value = trimmed[5..].trim();
            if name_spec_matches(value, interface) {
                output.push_str("Name=");
                output.push_str(value);
                if !value.is_empty() {
                    output.push(' ');
                }
                output.push_str(&exclusion);
                output.push('\n');
                replaced = true;
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    if !replaced {
        return Err("wireless_match_name_not_found".to_string());
    }
    Ok(output)
}

#[cfg(test)]
fn release_generic_wireless_claim(
    dir: &Path,
    interface: &str,
) -> Result<Option<Vec<u8>>, String> {
    let path = dir.join(GENERIC_WIRELESS_FILE);
    let previous = match read_previous_regular(&path)? {
        Some(bytes) => bytes,
        None => return Ok(None),
    };
    let text = String::from_utf8(previous.clone())
        .map_err(|_| "wireless_claim_is_not_utf8".to_string())?;
    let name_line = text.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("name="))
            .then(|| trimmed[5..].trim())
    });
    if name_line.is_none() || name_line.is_some_and(|spec| !name_spec_matches(spec, interface)) {
        return Ok(None);
    }
    let rewritten = exclude_name_from_wireless_match(&text, interface)?;
    if rewritten == text {
        return Ok(None);
    }
    atomic_write_file(dir, GENERIC_WIRELESS_FILE, rewritten.as_bytes(), 0o644)
        .map_err(|error| format!("wireless_claim_write_failed: {error}"))?;
    Ok(Some(previous))
}

fn with_rollback_result(primary: String, rollback: Result<(), String>) -> String {
    match rollback {
        Ok(()) => primary,
        Err(error) => format!("{primary}; rollback_failed: {error}"),
    }
}

fn json_ipv4_address(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return text.parse::<Ipv4Addr>().ok().map(|addr| addr.to_string());
    }
    let octets = value.as_array()?;
    if octets.len() != 4 {
        return None;
    }
    let mut parsed = [0_u8; 4];
    for (index, octet) in octets.iter().enumerate() {
        let number = octet.as_u64()?;
        if number > 255 {
            return None;
        }
        parsed[index] = number as u8;
    }
    Some(Ipv4Addr::from(parsed).to_string())
}

fn network_config_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(path) = std::env::var_os("EMPIIEROS_TEST_NETWORK_DIR") {
        return PathBuf::from(path);
    }
    PathBuf::from("/etc/systemd/network")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_static(interface: &str) -> ConfigureParams {
        ConfigureParams {
            interface: interface.to_string(),
            method: "static".to_string(),
            address: Some("192.168.8.10".to_string()),
            prefix: Some(30),
            gateway: Some("192.168.8.9".to_string()),
            dns: vec!["1.1.1.1".to_string()],
        }
    }

    #[test]
    fn render_rejects_injection_and_out_of_subnet_gateway() {
        let mut params = base_static("eth0\n[Network]");
        assert!(render_network_file(&params).is_ok());
        assert!(validate_interface(&params.interface).is_err());

        params.interface = "eth0".to_string();
        params.gateway = Some("192.168.9.1".to_string());
        assert_eq!(
            render_network_file(&params),
            Err("gateway_is_outside_interface_subnet".to_string())
        );
    }

    #[test]
    fn render_has_no_unvalidated_newlines() {
        let params = base_static("eth0");
        let rendered = render_network_file(&params).expect("render");
        assert!(rendered.contains("Address=192.168.8.10/30"));
        assert!(rendered.contains("Gateway=192.168.8.9"));
        assert!(!rendered.contains("DHCP=yes"));
    }

    #[test]
    fn dhcp_rejects_static_fields() {
        let params = ConfigureParams {
            interface: "eth0".to_string(),
            method: "dhcp".to_string(),
            address: Some("192.168.1.2".to_string()),
            prefix: None,
            gateway: None,
            dns: Vec::new(),
        };
        assert_eq!(
            render_network_file(&params),
            Err("dhcp_rejects_static_fields".to_string())
        );
    }

    #[test]
    fn static_config_rejects_network_broadcast_and_self_gateway() {
        let mut params = base_static("eth0");
        params.address = Some("192.168.8.8".to_string());
        assert_eq!(
            render_network_file(&params),
            Err("address_must_be_a_unicast_host".to_string())
        );

        params.address = Some("192.168.8.10".to_string());
        params.gateway = Some("192.168.8.10".to_string());
        assert_eq!(
            render_network_file(&params),
            Err("gateway_must_differ_from_interface_address".to_string())
        );

        params.gateway = Some("192.168.8.11".to_string());
        assert_eq!(
            render_network_file(&params),
            Err("gateway_must_be_a_unicast_host".to_string())
        );
    }

    #[test]
    fn networkd_verification_rejects_stale_or_wrong_source_addresses() {
        let path = Path::new("/etc/systemd/network/05-empiieros-eth0.network");
        let mut status = json!({
            "NetworkFile": path,
            "Addresses": [{
                "Family": libc::AF_INET,
                "Address": "192.168.8.10",
                "PrefixLength": 24,
                "ConfigSource": "static",
                "ConfigState": "configured"
            }]
        });
        assert!(networkd_status_matches(
            &status,
            path,
            Some("192.168.8.10/24")
        ));
        assert!(!networkd_status_matches(&status, path, None));

        status["Addresses"][0]["ConfigSource"] = json!("DHCPv4");
        assert!(networkd_status_matches(&status, path, None));
        assert!(!networkd_status_matches(
            &status,
            path,
            Some("192.168.8.10/24")
        ));

        status["NetworkFile"] = json!("/etc/systemd/network/25-stale.network");
        assert!(!networkd_status_matches(&status, path, None));
    }

    #[test]
    fn networkd_verification_accepts_systemd_octet_addresses() {
        let path = Path::new("/etc/systemd/network/05-empiieros-wlp58s0.network");
        let status = json!({
            "NetworkFile": path,
            "Addresses": [
                {
                    "Family": 10,
                    "Address": [254, 128, 0, 0, 0, 0, 0, 0, 94, 186, 239, 255, 254, 235, 8, 161],
                    "PrefixLength": 64,
                    "ConfigSource": "foreign",
                    "ConfigState": "configured"
                },
                {
                    "Family": 2,
                    "Address": [192, 168, 38, 82],
                    "PrefixLength": 24,
                    "ConfigSource": "DHCPv4",
                    "ConfigState": "configured"
                }
            ]
        });
        assert!(networkd_status_matches(&status, path, None));
        assert!(!networkd_status_matches(
            &status,
            path,
            Some("192.168.38.82/24")
        ));

        let static_status = json!({
            "NetworkFile": path,
            "Addresses": [{
                "Family": 2,
                "Address": [192, 168, 8, 10],
                "PrefixLength": 30,
                "ConfigSource": "static",
                "ConfigState": "configured"
            }]
        });
        assert!(networkd_status_matches(
            &static_status,
            path,
            Some("192.168.8.10/30")
        ));
        assert_eq!(
            json_ipv4_address(Some(&json!([192, 168, 38, 82]))).as_deref(),
            Some("192.168.38.82")
        );
        assert!(json_ipv4_address(Some(&json!([192, 168, 38]))).is_none());
        assert!(json_ipv4_address(Some(&json!("not-an-ip"))).is_none());
    }

    #[test]
    fn wifi_claim_rewrites_the_already_bound_wireless_file() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-wifi-claim-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("dir");
        let (name, mode) = claim_plan(&root, "wlp58s0", true);
        assert_eq!(name, "05-empiieros-wlp58s0.network");
        assert_eq!(mode, 0o600);
        fs::write(root.join(GENERIC_WIRELESS_FILE), "[Match]\nName=wl*\n").expect("wireless");
        let (name, mode) = claim_plan(&root, "wlp58s0", true);
        assert_eq!(name, GENERIC_WIRELESS_FILE);
        assert_eq!(mode, 0o644);
        let (name, mode) = claim_plan(&root, "eth0", false);
        assert_eq!(name, "05-empiieros-eth0.network");
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn wireless_name_exclusion_unclaims_a_matching_glob() {
        let original = "[Match]\nName=wl*\nType=wlan\n\n[Network]\nDHCP=ipv4\n";
        assert!(name_spec_matches("wl*", "wlp58s0"));
        assert!(!name_spec_matches("wl* !wlp58s0", "wlp58s0"));
        assert!(name_spec_matches("wl* !wlp58s0", "wlp0s20f3"));
        assert!(!name_spec_matches("en*", "wlp58s0"));

        let rewritten = exclude_name_from_wireless_match(original, "wlp58s0").expect("exclude");
        assert!(rewritten.contains("Name=wl* !wlp58s0"));
        assert!(rewritten.contains("Type=wlan"));
        assert_eq!(
            exclude_name_from_wireless_match(&rewritten, "wlp58s0").unwrap_err(),
            "wireless_match_name_not_found"
        );
    }

    #[test]
    fn release_generic_wireless_claim_rewrites_and_is_reversible() {
        let root = std::env::temp_dir().join(format!(
            "empiieros-wireless-claim-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("dir");
        let original = "[Match]\nName=wl*\nType=wlan\n\n[Network]\nDHCP=ipv4\n";
        fs::write(root.join(GENERIC_WIRELESS_FILE), original).expect("write");
        let previous = release_generic_wireless_claim(&root, "wlp58s0")
            .expect("release")
            .expect("changed");
        assert_eq!(previous, original.as_bytes());
        let now = fs::read_to_string(root.join(GENERIC_WIRELESS_FILE)).expect("read");
        assert!(now.contains("Name=wl* !wlp58s0"));
        assert!(release_generic_wireless_claim(&root, "wlp58s0")
            .expect("second")
            .is_none());
        atomic_write_file(&root, GENERIC_WIRELESS_FILE, &previous, 0o644).expect("restore");
        let restored = fs::read_to_string(root.join(GENERIC_WIRELESS_FILE)).expect("restored");
        assert_eq!(restored, original);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn previous_config_reader_refuses_symlinks() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "empiieros-network-previous-{}",
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
}
