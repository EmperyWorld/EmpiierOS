use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub struct DomainRecord<T> {
    pub availability: &'static str,
    pub source: &'static str,
    pub reason: Option<String>,
    pub data: T,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServiceRecord {
    pub name: String,
    pub observed: bool,
    pub state: String,
    pub result: String,
    pub ran: bool,
    pub nrestarts: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConnectorRecord {
    pub name: String,
    pub status: String,
    pub enabled: String,
    pub dpms: String,
    pub modes: Vec<String>,
    pub connector_id: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServicesData {
    pub units: Vec<ServiceRecord>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WifiAssociation {
    pub interface: String,
    pub ssid: String,
    pub bssid: Option<String>,
    pub frequency_mhz: Option<u32>,
    pub signal_dbm: Option<i16>,
    pub rx_mbps: Option<u32>,
    pub tx_mbps: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RadioRecord {
    pub name: String,
    pub kind: String,
    pub soft_blocked: bool,
    pub hard_blocked: bool,
    pub state: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct WifiData {
    pub interfaces: Vec<String>,
    pub associations: Vec<WifiAssociation>,
    pub radios: Vec<RadioRecord>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DisplayData {
    pub connectors: Vec<ConnectorRecord>,
    pub backlights: Vec<BacklightRecord>,
    pub driver: Option<String>,
    pub pci_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BacklightRecord {
    pub name: String,
    pub brightness: Option<u32>,
    pub max_brightness: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct InputDevice {
    pub node: String,
    pub name: String,
    pub has_key: bool,
    pub has_rel: bool,
    pub has_abs: bool,
    pub has_rel_wheel: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct InputData {
    pub event_nodes: u32,
    pub devices: Vec<InputDevice>,
    pub rel_wheel_observed: bool,
    pub uinput_observed: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AudioCard {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AudioData {
    pub cards: u32,
    pub devices: Vec<AudioCard>,
    pub playback_name: Option<String>,
    pub observed_percent: Option<u8>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BlockDevice {
    pub name: String,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StorageData {
    pub root_fstype: Option<String>,
    pub root_read_only: Option<bool>,
    pub root_total_bytes: Option<u64>,
    pub root_used_bytes: Option<u64>,
    pub root_available_bytes: Option<u64>,
    pub block_devices: u32,
    pub devices: Vec<BlockDevice>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LocaleData {
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub ntp_synchronized: Option<bool>,
    pub ntp_server: Option<String>,
    pub ntp_last_sync_epoch: Option<u64>,
    pub ntp_stratum: Option<u8>,
    pub ntp_jitter_ms: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UsbDevice {
    pub name: String,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub speed: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UsbData {
    pub devices: Vec<UsbDevice>,
}

pub fn probe_services() -> DomainRecord<ServicesData> {
    let units = [
        (
            "empiieros-runtime.service",
            Some("/run/empiieros/runtime.sock"),
        ),
        (
            "empiieros-shell-bridge.socket",
            Some("/run/empiieros-shell/runtime.sock"),
        ),
        (
            "systemd-networkd.service",
            Some("/run/systemd/netif/state"),
        ),
        ("systemd-timesyncd.service", None),
        ("systemd-networkd-wait-online.service", None),
        ("sshd.service", None),
        ("systemd-resolved.service", Some("/run/systemd/resolve/resolv.conf")),
        ("empiieros-wifi-restore.service", None),
        ("seatd.service", None),
    ]
    .into_iter()
    .map(|(name, witness)| {
        let observed = witness.is_some_and(|path| Path::new(path).exists())
            || unit_file_exists(name);
        let (state, result, ran, nrestarts) = unit_status(name);
        ServiceRecord {
            name: name.to_string(),
            observed,
            state,
            result,
            ran,
            nrestarts,
        }
    })
    .collect::<Vec<_>>();
    let observed = units.iter().filter(|unit| unit.observed).count();
    DomainRecord {
        availability: if observed == units.len() {
            "observed"
        } else if observed > 0 {
            "partial"
        } else {
            "unavailable"
        },
        source: "linux-runtime-witnesses",
        reason: if observed == units.len() {
            None
        } else {
            Some("one or more unit witnesses are absent".to_string())
        },
        data: ServicesData { units },
    }
}

fn unit_file_exists(name: &str) -> bool {
    [
        "/etc/systemd/system",
        "/usr/lib/systemd/system",
        "/lib/systemd/system",
    ]
    .into_iter()
    .any(|root| Path::new(root).join(name).is_file())
}

fn unit_status(name: &str) -> (String, String, bool, Option<u32>) {
    if let Some(status) = systemd_unit_status(name) {
        return status;
    }
    if Path::new("/sys/fs/cgroup/system.slice").join(name).is_dir() {
        ("active".to_string(), "unknown".to_string(), true, None)
    } else {
        ("inactive".to_string(), "unknown".to_string(), false, None)
    }
}

fn systemd_unit_status(name: &str) -> Option<(String, String, bool, Option<u32>)> {
    if !is_allowlisted_unit(name) {
        return None;
    }
    let result = crate::command_runner::run(
        "/usr/bin/systemctl",
        &[
            "show",
            name,
            "-p",
            "ActiveState",
            "-p",
            "Result",
            "-p",
            "NRestarts",
            "-p",
            "ExecMainStartTimestamp",
        ],
        std::time::Duration::from_secs(2),
    )
    .ok()?;
    if !result.success {
        return None;
    }
    parse_unit_show(result.stdout.trim())
}

fn is_allowlisted_unit(name: &str) -> bool {
    matches!(
        name,
        "empiieros-runtime.service"
            | "empiieros-shell-bridge.socket"
            | "systemd-networkd.service"
            | "systemd-timesyncd.service"
            | "systemd-networkd-wait-online.service"
            | "sshd.service"
            | "systemd-resolved.service"
            | "empiieros-wifi-restore.service"
            | "seatd.service"
    )
}

fn parse_active_state(raw: &str) -> Option<String> {
    matches!(
        raw,
        "active" | "inactive" | "failed" | "activating" | "deactivating"
    )
    .then(|| raw.to_string())
}

fn parse_unit_result(raw: &str) -> Option<String> {
    matches!(
        raw,
        "success"
            | "protocol"
            | "timeout"
            | "exit-code"
            | "wait"
            | "signal"
            | "core-dump"
            | "watchdog"
            | "start-limit-hit"
            | "resources"
    )
    .then(|| raw.to_string())
}

fn parse_nrestarts(raw: &str) -> Option<u32> {
    let value = raw.trim().parse::<u32>().ok()?;
    (value <= 10_000).then_some(value)
}

fn parse_unit_show(raw: &str) -> Option<(String, String, bool, Option<u32>)> {
    let mut state = None;
    let mut result = None;
    let mut ran = false;
    let mut nrestarts = None;
    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("ActiveState=") {
            state = parse_active_state(value.trim());
        } else if let Some(value) = line.strip_prefix("Result=") {
            result = parse_unit_result(value.trim());
        } else if let Some(value) = line.strip_prefix("NRestarts=") {
            nrestarts = parse_nrestarts(value);
        } else if let Some(value) = line.strip_prefix("ExecMainStartTimestamp=") {
            ran = !value.trim().is_empty();
        }
    }
    Some((
        state?,
        result.unwrap_or_else(|| "unknown".to_string()),
        ran,
        nrestarts,
    ))
}

fn drm_driver() -> Option<String> {
    let link = fs::read_link("/sys/class/drm/card0/device/driver").ok()?;
    let name = link.file_name()?.to_string_lossy();
    sanitize_drm_driver(&name)
}

fn drm_pci_id() -> Option<String> {
    let vendor = read_sysfs_trimmed(Path::new("/sys/class/drm/card0/device/vendor"))?;
    let device = read_sysfs_trimmed(Path::new("/sys/class/drm/card0/device/device"))?;
    parse_pci_id(&vendor, &device)
}

fn sanitize_drm_driver(raw: &str) -> Option<String> {
    let value = raw.trim();
    (!value.is_empty()
        && value.len() <= 32
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
    .then(|| value.to_string())
}

fn parse_pci_id(vendor: &str, device: &str) -> Option<String> {
    let vendor = vendor.trim().strip_prefix("0x").unwrap_or(vendor.trim());
    let device = device.trim().strip_prefix("0x").unwrap_or(device.trim());
    if vendor.len() == 4
        && device.len() == 4
        && vendor.chars().all(|ch| ch.is_ascii_hexdigit())
        && device.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        Some(format!(
            "{}:{}",
            vendor.to_ascii_lowercase(),
            device.to_ascii_lowercase()
        ))
    } else {
        None
    }
}

pub fn probe_wifi() -> DomainRecord<WifiData> {
    let mut interfaces = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("wireless").exists() || path.join("phy80211").exists() {
                interfaces.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    interfaces.sort();
    let associations = interfaces
        .iter()
        .filter_map(|interface| {
            crate::wifi_control::associated_link(interface).map(|link| WifiAssociation {
                interface: interface.clone(),
                ssid: link.ssid,
                bssid: link.bssid,
                frequency_mhz: link.frequency_mhz,
                signal_dbm: link.signal_dbm,
                rx_mbps: link.rx_mbps,
                tx_mbps: link.tx_mbps,
            })
        })
        .collect::<Vec<_>>();
    let availability = if interfaces.is_empty() {
        "unavailable"
    } else {
        "observed"
    };
    DomainRecord {
        availability,
        source: if associations.is_empty() {
            "linux-sysfs"
        } else {
            "linux-sysfs+iw"
        },
        reason: if interfaces.is_empty() {
            Some("no wireless netdevice is present".to_string())
        } else {
            None
        },
        data: WifiData {
            interfaces,
            associations,
            radios: probe_radios(),
        },
    }
}

pub fn probe_radios() -> Vec<RadioRecord> {
    let mut radios = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/rfkill") else {
        return radios;
    };
    for entry in entries.flatten() {
        let node = entry.file_name().to_string_lossy().into_owned();
        if !node.starts_with("rfkill") {
            continue;
        }
        let path = entry.path();
        let Some(kind) = read_sysfs_trimmed(&path.join("type")) else {
            continue;
        };
        let Some(name) = read_sysfs_trimmed(&path.join("name")) else {
            continue;
        };
        let Some(soft) = read_sysfs_flag(&path.join("soft")) else {
            continue;
        };
        let Some(hard) = read_sysfs_flag(&path.join("hard")) else {
            continue;
        };
        let Some(state) = read_sysfs_trimmed(&path.join("state"))
            .and_then(|value| value.parse::<u8>().ok())
        else {
            continue;
        };
        radios.push(RadioRecord {
            name,
            kind,
            soft_blocked: soft,
            hard_blocked: hard,
            state,
        });
    }
    radios.sort_by(|left, right| left.name.cmp(&right.name));
    radios
}

fn read_sysfs_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_sysfs_flag(path: &Path) -> Option<bool> {
    match read_sysfs_trimmed(path)?.as_str() {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

const MAX_DRM_MODES: usize = 16;
const MAX_DRM_MODE_CHARS: usize = 32;

fn parse_drm_modes(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|line| {
            let mode = line.trim();
            if mode.is_empty() || mode.len() > MAX_DRM_MODE_CHARS {
                return None;
            }
            mode.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, 'x' | 'X' | '.' | '+' | '-' | '_'))
                .then(|| mode.to_string())
        })
        .take(MAX_DRM_MODES)
        .collect()
}

pub fn probe_display() -> DomainRecord<DisplayData> {
    let mut connectors = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.contains('-') {
                continue;
            }
            let path = entry.path();
            let status = read_sysfs_trimmed(&path.join("status"))
                .unwrap_or_else(|| "unknown".to_string());
            let enabled = read_sysfs_trimmed(&path.join("enabled"))
                .unwrap_or_else(|| "unknown".to_string());
            let dpms = read_sysfs_trimmed(&path.join("dpms"))
                .unwrap_or_else(|| "unknown".to_string());
            let modes = parse_drm_modes(
                &fs::read_to_string(path.join("modes")).unwrap_or_default(),
            );
            let connector_id = read_sysfs_trimmed(&path.join("connector_id"))
                .and_then(|value| value.parse::<u32>().ok());
            connectors.push(ConnectorRecord {
                name,
                status,
                enabled,
                dpms,
                modes,
                connector_id,
            });
        }
    }
    connectors.sort_by(|left, right| left.name.cmp(&right.name));
    let backlights = probe_backlights();
    DomainRecord {
        availability: if connectors.is_empty() {
            "unavailable"
        } else {
            "observed"
        },
        source: "linux-sysfs",
        reason: if connectors.is_empty() {
            Some("no DRM connector status files are present".to_string())
        } else {
            None
        },
        data: DisplayData {
            connectors,
            backlights,
            driver: drm_driver(),
            pci_id: drm_pci_id(),
        },
    }
}

const EV_KEY: u16 = 1;
const EV_REL: u16 = 2;
const EV_ABS: u16 = 3;
const REL_WHEEL: u16 = 8;

pub fn probe_input() -> DomainRecord<InputData> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let node = entry.file_name().to_string_lossy().into_owned();
            if !is_event_node(&node) {
                continue;
            }
            let sys = format!("/sys/class/input/{node}/device");
            let name = fs::read_to_string(format!("{sys}/name"))
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "unknown".to_string());
            let ev = fs::read_to_string(format!("{sys}/capabilities/ev")).unwrap_or_default();
            let rel = fs::read_to_string(format!("{sys}/capabilities/rel")).unwrap_or_default();
            devices.push(InputDevice {
                node,
                name,
                has_key: evdev_bit_set(&ev, EV_KEY),
                has_rel: evdev_bit_set(&ev, EV_REL),
                has_abs: evdev_bit_set(&ev, EV_ABS),
                has_rel_wheel: evdev_bit_set(&rel, REL_WHEEL),
            });
        }
    }
    devices.sort_by(|left, right| left.node.cmp(&right.node));
    let event_nodes = devices.len() as u32;
    let rel_wheel_observed = devices.iter().any(|device| device.has_rel_wheel);
    let uinput_observed = Path::new("/dev/uinput").exists();
    DomainRecord {
        availability: if event_nodes == 0 {
            "unavailable"
        } else {
            "observed"
        },
        source: "linux-dev+sysfs",
        reason: if event_nodes == 0 {
            Some("no /dev/input/event nodes are present".to_string())
        } else {
            None
        },
        data: InputData {
            event_nodes,
            devices,
            rel_wheel_observed,
            uinput_observed,
        },
    }
}

fn is_event_node(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("event") else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit())
}

fn evdev_bit_set(text: &str, bit: u16) -> bool {
    let words: Vec<u64> = text
        .split_whitespace()
        .filter_map(|word| u64::from_str_radix(word, 16).ok())
        .collect();
    if words.is_empty() {
        return false;
    }
    let word_from_lsb = usize::from(bit) / 64;
    let bit_in_word = u64::from(bit) % 64;
    words
        .len()
        .checked_sub(word_from_lsb + 1)
        .and_then(|index| words.get(index))
        .is_some_and(|word| word & (1_u64 << bit_in_word) != 0)
}

pub fn probe_audio() -> DomainRecord<AudioData> {
    let devices = parse_alsa_cards(&fs::read_to_string("/proc/asound/cards").unwrap_or_default());
    let cards = devices.len() as u32;
    let volume = observe_playback_volume();
    DomainRecord {
        availability: if cards == 0 { "unavailable" } else { "observed" },
        source: if volume.is_some() {
            "linux-proc+alsa-mixer"
        } else {
            "linux-proc"
        },
        reason: if cards == 0 {
            Some("no ALSA card entries are present".to_string())
        } else {
            None
        },
        data: AudioData {
            cards,
            devices,
            playback_name: volume.as_ref().map(|item| item.0.clone()),
            observed_percent: volume.map(|item| item.1),
        },
    }
}

fn observe_playback_volume() -> Option<(String, u8)> {
    if !Path::new("/usr/local/sbin/empiieros-audio-volume").is_file() {
        return None;
    }
    let result = crate::command_runner::run(
        "/usr/local/sbin/empiieros-audio-volume",
        &["--observe"],
        std::time::Duration::from_secs(2),
    )
    .ok()?;
    if !result.success {
        return None;
    }
    parse_observed_volume(&result.stdout)
}

fn parse_observed_volume(output: &str) -> Option<(String, u8)> {
    let mut name = None;
    let mut percent = None;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("name=") {
            if !is_allowlisted_mixer(value) {
                return None;
            }
            name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("observed_percent=") {
            percent = value.trim().parse::<u8>().ok().filter(|item| *item <= 100);
        }
    }
    Some((name?, percent?))
}

fn is_allowlisted_mixer(name: &str) -> bool {
    matches!(name, "Speaker" | "Headphone" | "Master" | "PCM")
}

pub fn probe_storage() -> DomainRecord<StorageData> {
    let (root_fstype, root_read_only) = parse_root_mount(&host_proc_mounts().unwrap_or_default());
    let usage = root_statvfs();
    let devices = probe_block_devices();
    DomainRecord {
        availability: if root_fstype.is_some() {
            "observed"
        } else {
            "unavailable"
        },
        source: "linux-host-mounts+sysfs+statvfs",
        reason: if root_fstype.is_some() {
            None
        } else {
            Some("root mount could not be observed".to_string())
        },
        data: StorageData {
            root_fstype,
            root_read_only,
            root_total_bytes: usage.map(|item| item.0),
            root_used_bytes: usage.map(|item| item.1),
            root_available_bytes: usage.map(|item| item.2),
            block_devices: devices.len() as u32,
            devices,
        },
    }
}

fn root_statvfs() -> Option<(u64, u64, u64)> {
    let mut buf = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let rc = unsafe { libc::statvfs(c"/".as_ptr(), buf.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let vfs = unsafe { buf.assume_init() };
    usage_from_statvfs(vfs.f_frsize, vfs.f_blocks, vfs.f_bfree, vfs.f_bavail)
}

fn usage_from_statvfs(frsize: libc::c_ulong, blocks: libc::fsblkcnt_t, bfree: libc::fsblkcnt_t, bavail: libc::fsblkcnt_t) -> Option<(u64, u64, u64)> {
    let frsize = u64::try_from(frsize).ok()?;
    if frsize == 0 {
        return None;
    }
    let total = u64::try_from(blocks).ok()?.checked_mul(frsize)?;
    let free = u64::try_from(bfree).ok()?.checked_mul(frsize)?;
    let available = u64::try_from(bavail).ok()?.checked_mul(frsize)?;
    if total == 0 || free > total {
        return None;
    }
    Some((total, total - free, available))
}

fn probe_block_devices() -> Vec<BlockDevice> {
    let mut devices = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/block") else {
        return devices;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_observed_block(&name) {
            continue;
        }
        let size_bytes = fs::read_to_string(entry.path().join("size"))
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map(|sectors| sectors.saturating_mul(512));
        devices.push(BlockDevice { name, size_bytes });
    }
    devices.sort_by(|left, right| left.name.cmp(&right.name));
    devices
}

fn is_observed_block(name: &str) -> bool {
    !name.starts_with("loop") && !name.starts_with("ram") && !name.starts_with("zram")
}

pub fn probe_locale() -> DomainRecord<LocaleData> {
    let locale = fs::read_to_string("/etc/locale.conf")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("LANG=")
                    .map(|value| value.trim().trim_matches('"').to_string())
            })
        });
    let timezone = fs::read_link("/etc/localtime")
        .ok()
        .and_then(|path| {
            path.to_string_lossy()
                .rsplit("/zoneinfo/")
                .next()
                .map(str::to_string)
        })
        .or_else(|| fs::read_to_string("/etc/timezone").ok().map(|value| value.trim().to_string()));
    let timesync = timesync_text();
    DomainRecord {
        availability: if locale.is_some() || timezone.is_some() {
            "observed"
        } else {
            "unavailable"
        },
        source: "linux-etc+timesync",
        reason: if locale.is_none() && timezone.is_none() {
            Some("locale.conf and timezone are absent".to_string())
        } else {
            None
        },
        data: LocaleData {
            locale,
            timezone,
            ntp_synchronized: ntp_synchronized(),
            ntp_server: timesync
                .as_deref()
                .and_then(parse_timesync_server)
                .or_else(ntp_server_from_helper_result),
            ntp_last_sync_epoch: ntp_last_sync_epoch(),
            ntp_stratum: timesync.as_deref().and_then(parse_ntp_stratum),
            ntp_jitter_ms: timesync.as_deref().and_then(parse_ntp_jitter_ms),
        },
    }
}

pub fn probe_usb() -> DomainRecord<UsbData> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_usb_device_name(&name) {
                continue;
            }
            let path = entry.path();
            devices.push(UsbDevice {
                name,
                vendor_id: read_sysfs_trimmed(&path.join("idVendor"))
                    .and_then(|value| sanitize_usb_id(&value)),
                product_id: read_sysfs_trimmed(&path.join("idProduct"))
                    .and_then(|value| sanitize_usb_id(&value)),
                speed: read_sysfs_trimmed(&path.join("speed"))
                    .and_then(|value| sanitize_usb_speed(&value)),
                manufacturer: read_sysfs_trimmed(&path.join("manufacturer"))
                    .and_then(|value| sanitize_usb_text(&value)),
                product: read_sysfs_trimmed(&path.join("product"))
                    .and_then(|value| sanitize_usb_text(&value)),
            });
        }
    }
    devices.sort_by(|left, right| left.name.cmp(&right.name));
    DomainRecord {
        availability: if devices.is_empty() {
            "unavailable"
        } else {
            "observed"
        },
        source: "linux-sysfs",
        reason: if devices.is_empty() {
            Some("no USB device nodes were observed".to_string())
        } else {
            None
        },
        data: UsbData { devices },
    }
}

fn is_usb_device_name(name: &str) -> bool {
    if name.contains(':') || name.starts_with("usb") {
        return false;
    }
    let mut parts = name.split('-');
    let Some(bus) = parts.next() else {
        return false;
    };
    let Some(port) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && !bus.is_empty()
        && bus.chars().all(|ch| ch.is_ascii_digit())
        && !port.is_empty()
        && port.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
}

fn sanitize_usb_id(raw: &str) -> Option<String> {
    let value = raw.trim().to_ascii_lowercase();
    (value.len() == 4 && value.chars().all(|ch| ch.is_ascii_hexdigit())).then_some(value)
}

fn sanitize_usb_speed(raw: &str) -> Option<String> {
    let value = raw.trim();
    (!value.is_empty() && value.len() <= 8 && value.chars().all(|ch| ch.is_ascii_digit()))
        .then(|| value.to_string())
}

fn sanitize_usb_text(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > 64 {
        return None;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_graphic() || ch == ' ')
        .then(|| value.to_string())
}

pub(crate) fn host_proc_mounts() -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "/run/empiieros/host-mounts",
        "/proc/1/mounts",
        "/proc/mounts",
    ];
    CANDIDATES.iter().find_map(|path| {
        fs::read_to_string(path)
            .ok()
            .filter(|text| !text.is_empty())
    })
}

pub(crate) fn parse_root_mount(text: &str) -> (Option<String>, Option<bool>) {
    let root = text.lines().find(|line| {
        line.split_whitespace()
            .nth(1)
            .is_some_and(|mount| mount == "/")
    });
    let root_fstype = root.and_then(|line| line.split_whitespace().nth(2).map(str::to_string));
    let root_read_only = root.and_then(|line| {
        line.split_whitespace()
            .nth(3)
            .and_then(mount_options_read_only)
    });
    (root_fstype, root_read_only)
}

fn probe_backlights() -> Vec<BacklightRecord> {
    let mut backlights = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/backlight") else {
        return backlights;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let brightness = fs::read_to_string(entry.path().join("brightness"))
            .ok()
            .and_then(|value| value.trim().parse().ok());
        let max_brightness = fs::read_to_string(entry.path().join("max_brightness"))
            .ok()
            .and_then(|value| value.trim().parse().ok());
        backlights.push(BacklightRecord {
            name,
            brightness,
            max_brightness,
        });
    }
    backlights.sort_by(|left, right| left.name.cmp(&right.name));
    backlights
}

pub(crate) fn mount_options_read_only(options: &str) -> Option<bool> {
    let tokens = options.split(',').collect::<Vec<_>>();
    if tokens.iter().any(|option| *option == "rw") {
        Some(false)
    } else if tokens.iter().any(|option| *option == "ro") {
        Some(true)
    } else {
        None
    }
}

fn ntp_synchronized() -> Option<bool> {
    if Path::new("/run/systemd/timesync/synchronized").is_file() {
        return Some(true);
    }
    if Path::new("/run/systemd/timesync").is_dir() {
        return Some(false);
    }
    None
}

fn timesync_text() -> Option<String> {
    let result = crate::command_runner::run(
        "/usr/bin/timedatectl",
        &["show-timesync"],
        std::time::Duration::from_secs(2),
    )
    .ok()?;
    if !result.success || result.stdout.trim().is_empty() {
        return None;
    }
    Some(result.stdout)
}

fn ntp_last_sync_epoch() -> Option<u64> {
    let meta = fs::symlink_metadata("/run/systemd/timesync/synchronized").ok()?;
    if !meta.file_type().is_file() {
        return None;
    }
    meta.modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn parse_timesync_server(text: &str) -> Option<String> {
    for line in text.lines() {
        let Some(value) = line.strip_prefix("ServerName=") else {
            continue;
        };
        let server = value.trim();
        if crate::time_control::is_ntp_server(server) {
            return Some(server.to_string());
        }
    }
    None
}

fn parse_ntp_stratum(text: &str) -> Option<u8> {
    let message = text.lines().find(|line| line.starts_with("NTPMessage="))?;
    let rest = message.split("Stratum=").nth(1)?;
    let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    let stratum = digits.parse::<u8>().ok()?;
    (1..=16).contains(&stratum).then_some(stratum)
}

fn parse_ntp_jitter_ms(text: &str) -> Option<u32> {
    let message = text.lines().find(|line| line.starts_with("NTPMessage="))?;
    let rest = message.split("Jitter=").nth(1)?.trim_start();
    let unit_at = rest.find(|ch: char| ch.is_ascii_alphabetic())?;
    let value = rest.get(..unit_at)?.parse::<f32>().ok()?;
    let unit: String = rest[unit_at..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect();
    let ms = match unit.as_str() {
        "ms" => value,
        "us" => value / 1000.0,
        "s" => value * 1000.0,
        _ => return None,
    };
    (ms.is_finite() && (0.0..=10_000.0).contains(&ms)).then_some(ms.round() as u32)
}

fn ntp_server_from_helper_result() -> Option<String> {
    let body = fs::read_to_string("/run/empiieros/time-sync.result").ok()?;
    for line in body.lines() {
        let Some(("server", value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if crate::time_control::is_ntp_server(value) {
            return Some(value.to_string());
        }
    }
    None
}

fn parse_alsa_cards(text: &str) -> Vec<AudioCard> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (id, rest) = line.split_once(" [")?;
            let id = id.trim();
            if !id.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            let name = rest
                .split_once("]:")
                .map(|(_, value)| value.trim().to_string())
                .filter(|value| !value.is_empty())?;
            Some(AudioCard {
                id: id.to_string(),
                name,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_rw_wins_over_a_stale_ro_token() {
        assert_eq!(mount_options_read_only("rw,relatime"), Some(false));
        assert_eq!(mount_options_read_only("ro,relatime"), Some(true));
        assert_eq!(mount_options_read_only("relatime"), None);
    }

    #[test]
    fn host_root_mount_ignores_protect_system_relatime() {
        let (fstype, read_only) =
            parse_root_mount("/dev/root / ext4 rw,relatime 0 0\nproc /proc proc rw 0 0\n");
        assert_eq!(fstype.as_deref(), Some("ext4"));
        assert_eq!(read_only, Some(false));
    }

    #[test]
    fn alsa_cards_parser_keeps_observed_names() {
        let cards = parse_alsa_cards(
            " 0 [PCH            ]: HDA-Intel - HDA Intel PCH\n                      HDA Intel PCH\n 1 [HDMI           ]: HDA-Intel - HDA Intel HDMI\n",
        );
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].id, "0");
        assert_eq!(cards[0].name, "HDA-Intel - HDA Intel PCH");
        assert_eq!(
            parse_observed_volume("name=Speaker\nobserved_percent=24\n"),
            Some(("Speaker".into(), 24))
        );
        assert!(parse_observed_volume("name=Capture\nobserved_percent=24\n").is_none());
        assert!(parse_observed_volume("name=Speaker\nobserved_percent=101\n").is_none());
        let timesync = "ServerName=162.159.200.1\nNTPMessage={ Leap=0, Version=4, Mode=4, Stratum=3, Precision=-25, RootDelay=23.864ms, Jitter=11.945ms }\n";
        assert_eq!(parse_timesync_server(timesync).as_deref(), Some("162.159.200.1"));
        assert_eq!(parse_ntp_stratum(timesync), Some(3));
        assert_eq!(parse_ntp_jitter_ms(timesync), Some(12));
        assert!(parse_ntp_stratum("NTPMessage={ Stratum=99 }").is_none());
    }

    #[test]
    fn evdev_bitmaps_use_lsb_hex_words() {
        assert!(is_event_node("event9"));
        assert!(!is_event_node("mice"));
        assert!(evdev_bit_set("7", EV_KEY));
        assert!(evdev_bit_set("7", EV_REL));
        assert!(!evdev_bit_set("7", EV_ABS));
        assert!(evdev_bit_set("3", 0));
        assert!(evdev_bit_set("3", 1));
        assert!(!evdev_bit_set("3", REL_WHEEL));
        assert!(evdev_bit_set("100", REL_WHEEL));
        assert!(evdev_bit_set("0 100", REL_WHEEL));
        assert!(!evdev_bit_set("100 0", REL_WHEEL));
        assert!(!evdev_bit_set("0", REL_WHEEL));
    }

    #[test]
    fn service_and_gpu_parsers_keep_observed_tokens() {
        assert!(is_allowlisted_unit("empiieros-wifi-restore.service"));
        assert!(is_allowlisted_unit("systemd-networkd-wait-online.service"));
        assert!(is_allowlisted_unit("sshd.service"));
        assert!(is_allowlisted_unit("systemd-resolved.service"));
        assert!(!is_allowlisted_unit("reboot.service"));
        assert_eq!(
            usage_from_statvfs(4096, 100, 25, 20),
            Some((409_600, 307_200, 81_920))
        );
        assert_eq!(usage_from_statvfs(0, 100, 25, 20), None);
        assert_eq!(parse_active_state("failed").as_deref(), Some("failed"));
        assert!(parse_active_state("running").is_none());
        assert_eq!(parse_unit_result("success").as_deref(), Some("success"));
        assert_eq!(parse_unit_result("exit-code").as_deref(), Some("exit-code"));
        assert!(parse_unit_result("boom").is_none());
        assert_eq!(
            parse_unit_show(
                "ActiveState=inactive\nResult=success\nNRestarts=0\nExecMainStartTimestamp=Thu 2026-09-17 15:48:53 UTC"
            ),
            Some(("inactive".into(), "success".into(), true, Some(0)))
        );
        assert_eq!(
            parse_unit_show("ActiveState=inactive\nResult=success\nExecMainStartTimestamp="),
            Some(("inactive".into(), "success".into(), false, None))
        );
        assert_eq!(
            parse_unit_show("ActiveState=failed\nResult=exit-code\nNRestarts=1\nExecMainStartTimestamp=n/a"),
            Some(("failed".into(), "exit-code".into(), true, Some(1)))
        );
        assert!(parse_nrestarts("10001").is_none());
        assert_eq!(parse_nrestarts("0"), Some(0));
        assert_eq!(sanitize_drm_driver("i915").as_deref(), Some("i915"));
        assert!(sanitize_drm_driver("../evil").is_none());
        assert_eq!(parse_pci_id("0x8086", "0x9BC4").as_deref(), Some("8086:9bc4"));
        assert!(parse_pci_id("8086", "xyz").is_none());
    }

    #[test]
    fn usb_device_names_skip_buses_and_interfaces() {
        assert!(is_usb_device_name("1-11"));
        assert!(is_usb_device_name("2-2"));
        assert!(is_usb_device_name("1-1.3"));
        assert!(!is_usb_device_name("usb1"));
        assert!(!is_usb_device_name("1-11:1.0"));
        assert!(!is_usb_device_name("1-0:1.0"));
        assert_eq!(sanitize_usb_id("0bda"), Some("0bda".to_string()));
        assert_eq!(sanitize_usb_id("0BDA"), Some("0bda".to_string()));
        assert!(sanitize_usb_id("xyz").is_none());
        assert_eq!(sanitize_usb_speed("480"), Some("480".to_string()));
        assert!(sanitize_usb_text("name\nwith\nbreaks").is_none());
        assert_eq!(
            sanitize_usb_text("Integrated_Webcam_HD"),
            Some("Integrated_Webcam_HD".to_string())
        );
    }

    #[test]
    fn virtual_block_names_are_not_observed() {
        assert!(!is_observed_block("loop0"));
        assert!(!is_observed_block("ram1"));
        assert!(!is_observed_block("zram0"));
        assert!(is_observed_block("nvme0n1"));
        assert!(is_observed_block("sda"));
        assert!(is_observed_block("mmcblk0p1"));
    }

    #[test]
    fn drm_modes_keep_bounded_sysfs_names() {
        assert_eq!(
            parse_drm_modes("1920x1080\n1920x1080\n\n3840x2160\n"),
            vec!["1920x1080", "1920x1080", "3840x2160"]
        );
        assert!(parse_drm_modes("not a mode!").is_empty());
        assert_eq!(
            parse_drm_modes(&(0..20).map(|n| format!("{n}x{n}\n")).collect::<String>()).len(),
            MAX_DRM_MODES
        );
    }

    #[test]
    fn empty_wireless_is_unavailable() {
        let record = DomainRecord {
            availability: "unavailable",
            source: "linux-sysfs",
            reason: Some("no wireless netdevice is present".to_string()),
            data: WifiData {
                interfaces: Vec::new(),
                associations: Vec::new(),
                radios: Vec::new(),
            },
        };
        assert_eq!(record.availability, "unavailable");
        assert!(record.data.interfaces.is_empty());
    }
}
