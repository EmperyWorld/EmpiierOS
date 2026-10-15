//! Persist inner Capsule v3 material after a verified open.
//!
//! Entitlement and channel stay in Runtime state (`0600`). Shell IPC sees
//! flags only. Activation tokens and WG conf never enter the response.
//! This is not W-transport bring-up and not `product.open`.

use crate::capsule_v3::InnerCapsule;
use crate::state_store::atomic_write_file;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const INNER_DIR: &str = "capsule_inner";
const FLAGS_NAME: &str = "flags.json";
const ENTITLEMENT_NAME: &str = "entitlement.json";
const CHANNEL_NAME: &str = "channel.json";
const CAPSULES_DIR: &str = "capsules";
const SEALED_NAME: &str = "current.ecap";

pub const CHANNEL_MISSING: &str = "missing";
pub const CHANNEL_PRESENT_INVALID: &str = "present_invalid";
pub const CHANNEL_VALID_IDLE: &str = "valid_idle";
pub const CHANNEL_VALID_UP: &str = "valid_up";
pub const PORTAL_IFACE: &str = "wg-portal";

#[derive(Debug, Clone, Default)]
pub struct InnerView {
    pub entitlement_present: bool,
    pub channel_present: bool,
    pub channel_valid: bool,
    pub channel_up: bool,
    pub channel_code: String,
    pub allow_offline: Option<bool>,
    pub allow_online: Option<bool>,
    pub allow_fofline: Option<bool>,
}

pub fn persist(state_root: &Path, inner: &InnerCapsule, blob: &[u8]) -> Result<InnerView, String> {
    let dir = state_root.join(INNER_DIR);
    fs::create_dir_all(&dir).map_err(|error| format!("inner_store_failed:{error}"))?;
    set_dir_mode(&dir)?;
    let verdict = channel_verdict(inner.channel.as_ref().filter(|_| inner.flags.has_channel));
    let view = InnerView {
        entitlement_present: inner.entitlement.is_some(),
        channel_present: inner.flags.has_channel,
        channel_valid: verdict.valid,
        channel_up: false,
        channel_code: verdict.code.to_string(),
        allow_offline: inner.flags.allow_offline,
        allow_online: inner.flags.allow_online,
        allow_fofline: inner.flags.allow_fofline,
    };
    let flags = json!({
        "entitlement_present": view.entitlement_present,
        "channel_present": view.channel_present,
        "channel_valid": view.channel_valid,
        "channel_up": view.channel_up,
        "channel_code": view.channel_code,
        "allow_offline": view.allow_offline,
        "allow_online": view.allow_online,
        "allow_fofline": view.allow_fofline,
        "mode": inner.flags.mode,
        "note": "flags only — entitlement/channel stay out of IPC; not tunnel start"
    });
    write_json(&dir, FLAGS_NAME, &flags)?;
    match &inner.entitlement {
        Some(doc) => write_json(&dir, ENTITLEMENT_NAME, doc)?,
        None => remove_if_present(&dir.join(ENTITLEMENT_NAME))?,
    }
    match &inner.channel {
        Some(doc) if inner.flags.has_channel => write_json(&dir, CHANNEL_NAME, doc)?,
        _ => remove_if_present(&dir.join(CHANNEL_NAME))?,
    }
    persist_sealed(state_root, blob)?;
    Ok(view)
}

pub fn require_consumed(state_root: &Path) -> Result<(), String> {
    crate::launcher_pin::require_unlocked(state_root)?;
    if !observe(state_root).entitlement_present {
        return Err("license_required".into());
    }
    Ok(())
}

pub fn observe(state_root: &Path) -> InnerView {
    let raw = fs::read_to_string(state_root.join(INNER_DIR).join(FLAGS_NAME)).ok();
    let Some(value) = raw.and_then(|text| serde_json::from_str::<Value>(&text).ok()) else {
        return InnerView {
            channel_code: CHANNEL_MISSING.into(),
            ..InnerView::default()
        };
    };
    let channel = fs::read_to_string(state_root.join(INNER_DIR).join(CHANNEL_NAME))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let verdict = channel_verdict(channel.as_ref());
    let channel_up = portal_iface_up();
    let code = if verdict.valid && channel_up {
        CHANNEL_VALID_UP
    } else {
        verdict.code
    };
    InnerView {
        entitlement_present: value
            .get("entitlement_present")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        channel_present: value
            .get("channel_present")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || channel.is_some(),
        channel_valid: verdict.valid,
        channel_up,
        channel_code: code.to_string(),
        allow_offline: value.get("allow_offline").and_then(Value::as_bool),
        allow_online: value.get("allow_online").and_then(Value::as_bool),
        allow_fofline: value.get("allow_fofline").and_then(Value::as_bool),
    }
}

pub fn channel_block(state_root: &Path) -> Option<&'static str> {
    read_validated_conf(state_root).err()
}

pub fn load_validated_conf(state_root: &Path) -> Result<String, String> {
    read_validated_conf(state_root).map_err(str::to_string)
}

fn read_validated_conf(state_root: &Path) -> Result<String, &'static str> {
    let raw = fs::read_to_string(state_root.join(INNER_DIR).join(CHANNEL_NAME))
        .map_err(|_| "channel_missing")?;
    let doc: Value = serde_json::from_str(&raw).map_err(|_| "channel_json")?;
    let conf = doc
        .get("wg_conf")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or("channel_conf_missing")?;
    crate::wg_conf::validate_portal_conf(conf).map_err(|error| error.code)?;
    Ok(conf.to_string())
}

pub fn portal_iface_up() -> bool {
    let path = Path::new("/sys/class/net").join(PORTAL_IFACE).join("operstate");
    match fs::read_to_string(path) {
        Ok(text) => !matches!(text.trim(), "down"),
        Err(_) => false,
    }
}

struct ChannelVerdict {
    valid: bool,
    code: &'static str,
}

fn channel_verdict(channel: Option<&Value>) -> ChannelVerdict {
    let Some(doc) = channel else {
        return ChannelVerdict {
            valid: false,
            code: CHANNEL_MISSING,
        };
    };
    let Some(conf) = doc.get("wg_conf").and_then(Value::as_str).map(str::trim) else {
        return ChannelVerdict {
            valid: false,
            code: CHANNEL_PRESENT_INVALID,
        };
    };
    if conf.is_empty() {
        return ChannelVerdict {
            valid: false,
            code: CHANNEL_PRESENT_INVALID,
        };
    }
    match crate::wg_conf::validate_portal_conf(conf) {
        Ok(()) => ChannelVerdict {
            valid: true,
            code: CHANNEL_VALID_IDLE,
        },
        Err(_) => ChannelVerdict {
            valid: false,
            code: CHANNEL_PRESENT_INVALID,
        },
    }
}

fn persist_sealed(state_root: &Path, blob: &[u8]) -> Result<(), String> {
    let dir = state_root.join(CAPSULES_DIR);
    fs::create_dir_all(&dir).map_err(|error| format!("sealed_store_failed:{error}"))?;
    set_dir_mode(&dir)?;
    atomic_write_file(&dir, SEALED_NAME, blob, 0o600)
        .map_err(|error| format!("sealed_store_failed:{error}"))?;
    Ok(())
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), String> {
    let body = format!("{value}\n");
    if body.contains("activation_token") {
        return Err("inner_token_redaction_failed".into());
    }
    atomic_write_file(dir, name, body.as_bytes(), 0o600)
        .map_err(|error| format!("inner_store_failed:{error}"))?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("inner_store_failed:{error}")),
    }
}

fn set_dir_mode(dir: &Path) -> Result<(), String> {
    let mut permissions = fs::metadata(dir)
        .map_err(|error| format!("inner_store_failed:{error}"))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(dir, permissions)
        .map_err(|error| format!("inner_store_failed:{error}"))?;
    Ok(())
}
