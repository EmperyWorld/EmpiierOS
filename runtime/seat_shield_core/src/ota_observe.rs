//! OTA capsule observe hooks — baseline rematch + VIP exemption helpers.
//!
//! Contract: `session/SEAT-SHIELD.md` §9.3 · `session/OTA-CAPSULE.md`
//! Not `security.integrity.*`. Not IOC feed. No auto quarantine of `/usr`.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::{ShieldError, ShieldResult};
use crate::event::{Severity, ShieldEvent};
use crate::store::ShieldStore;
use crate::timeutil::now_epoch;

/// Portal VIP used by `empiieros-ota-fetch` (WireGuard seat path).
pub const OTA_PORTAL_VIP: &str = "10.88.89.1";

const BASELINE_NAME: &str = "ota_baseline.json";
const MAX_BASELINE_ENTRIES: usize = 64;
const MAX_REMATCH_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaBaselineStatus {
    Missing,
    Ok,
    Mismatch,
    Stale,
}

impl OtaBaselineStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Ok => "ok",
            Self::Mismatch => "mismatch",
            Self::Stale => "stale",
        }
    }
}

#[must_use]
pub fn is_ota_portal_vip(ip: &str) -> bool {
    ip.trim() == OTA_PORTAL_VIP
}

/// Rematch `{state}/seat_shield/ota_baseline.json` against on-disk digests.
/// Absent file → `missing` (honest until E4 cutover writes it).
#[must_use]
pub fn observe_ota_baseline(state_root: &Path) -> OtaBaselineStatus {
    let path = baseline_path(state_root);
    let Ok(raw) = fs::read_to_string(&path) else {
        return OtaBaselineStatus::Missing;
    };
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&raw) else {
        return OtaBaselineStatus::Stale;
    };
    if map.is_empty() || map.len() > MAX_BASELINE_ENTRIES {
        return OtaBaselineStatus::Stale;
    }

    let mut checked = 0usize;
    for (dest, want_v) in &map {
        let Some(want) = want_v.as_str() else {
            return OtaBaselineStatus::Stale;
        };
        let want = want.trim().to_ascii_lowercase();
        if want.len() != 64 || !want.chars().all(|c| c.is_ascii_hexdigit()) {
            return OtaBaselineStatus::Stale;
        }
        if !dest.starts_with('/') || dest.contains("..") || dest.contains('\0') {
            return OtaBaselineStatus::Stale;
        }
        let dest_path = rematch_path(dest);
        let Ok(meta) = fs::metadata(&dest_path) else {
            return OtaBaselineStatus::Mismatch;
        };
        if !meta.is_file() {
            return OtaBaselineStatus::Mismatch;
        }
        if meta.len() > MAX_REMATCH_BYTES {
            // Bound: skip oversized; do not auto-quarantine.
            continue;
        }
        let Ok(got) = sha256_file_capped(&dest_path, MAX_REMATCH_BYTES) else {
            return OtaBaselineStatus::Mismatch;
        };
        if got != want {
            return OtaBaselineStatus::Mismatch;
        }
        checked = checked.saturating_add(1);
    }
    if checked == 0 {
        OtaBaselineStatus::Stale
    } else {
        OtaBaselineStatus::Ok
    }
}

/// Append observe-only `ota_capsule_staged` after a verified stage. Fail-soft.
pub fn emit_ota_capsule_staged(
    state_root: &Path,
    app_build_id: &str,
    channel: &str,
) -> ShieldResult<()> {
    let store = ShieldStore::open(state_root)?;
    let summary = format!("OTA capsule staged (build={app_build_id}, channel={channel})");
    let event = ShieldEvent {
        kind: "ota_capsule_staged".into(),
        severity: Severity::Medium,
        summary,
        entity_ip: Some(OTA_PORTAL_VIP.into()),
        entity_process: Some("empiieros-ota-fetch".into()),
        entity_port: None,
        explain_why: "Verified OTA capsule is in ota-capsule/current. This event is stage, not cutover."
            .into(),
        explain_action:
            "Observe only. Typed APPLY_OTA in Updates runs launcher.ota_apply cutover."
                .into(),
        confidence_milli: 950,
        tags: vec![
            "seat".into(),
            "ota".into(),
            "observe_only".into(),
            channel.to_string(),
        ],
        sequence: 0,
        epoch: now_epoch(),
        chain_id: None,
    };
    store.append_events(vec![event])?;
    Ok(())
}

/// Observe-only mismatch event when baseline rematch fails. No quarantine.
pub fn ota_component_mismatch_event(detail: &str) -> ShieldEvent {
    ShieldEvent {
        kind: "ota_component_mismatch".into(),
        severity: Severity::Critical,
        summary: format!("OTA baseline mismatch: {detail}"),
        entity_ip: None,
        entity_process: None,
        entity_port: None,
        explain_why: "On-disk digest for a P4 OTA destination does not match ota_baseline.json."
            .into(),
        explain_action: "Observe only. Do not auto-quarantine /usr or /opt/empiieros.".into(),
        confidence_milli: 900,
        tags: vec!["seat".into(), "ota".into(), "observe_only".into()],
        sequence: 0,
        epoch: now_epoch(),
        chain_id: None,
    }
}

/// Observe-only mismatch event when baseline rematch fails. No quarantine.
pub fn emit_ota_component_mismatch(state_root: &Path, detail: &str) -> ShieldResult<()> {
    let store = ShieldStore::open(state_root)?;
    store.append_events(vec![ota_component_mismatch_event(detail)])?;
    Ok(())
}

fn baseline_path(state_root: &Path) -> PathBuf {
    state_root.join("seat_shield").join(BASELINE_NAME)
}

/// Write `{state}/seat_shield/ota_baseline.json` (dest → sha256). E4 cutover.
pub fn write_ota_baseline(
    state_root: &Path,
    digests: &std::collections::BTreeMap<String, String>,
) -> ShieldResult<()> {
    if digests.is_empty() || digests.len() > MAX_BASELINE_ENTRIES {
        return Err(ShieldError::InvalidParams(
            "ota_baseline entry count out of bounds".into(),
        ));
    }
    let mut map = serde_json::Map::new();
    for (dest, digest) in digests {
        let d = digest.trim().to_ascii_lowercase();
        if d.len() != 64 || !d.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ShieldError::InvalidParams(
                "ota_baseline digest must be 64 hex".into(),
            ));
        }
        if !dest.starts_with('/') || dest.contains("..") || dest.contains('\0') {
            return Err(ShieldError::InvalidParams(
                "ota_baseline dest must be absolute".into(),
            ));
        }
        map.insert(dest.clone(), Value::String(d));
    }
    let store = ShieldStore::open(state_root)?;
    let _ = store; // ensure seat_shield/ exists
    let path = baseline_path(state_root);
    let parent = path.parent().ok_or_else(|| {
        ShieldError::InvalidParams("ota_baseline parent missing".into())
    })?;
    let raw = serde_json::to_vec_pretty(&Value::Object(map))
        .map_err(|e| ShieldError::Serialize(e.to_string()))?;
    // Atomic replace under seat_shield/.
    let tmp = parent.join(format!(".ota_baseline.{}.tmp", std::process::id()));
    fs::write(&tmp, &raw).map_err(|source| ShieldError::Io {
        context: "ota_baseline_tmp",
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp, &path).map_err(|source| ShieldError::Io {
        context: "ota_baseline_rename",
        source,
    })?;
    Ok(())
}

fn rematch_path(dest: &str) -> PathBuf {
    if let Ok(root) = std::env::var("EMPIIEROS_OTA_CUTOVER_ROOT") {
        let abs = Path::new(dest);
        let rel = abs.strip_prefix("/").unwrap_or(abs);
        return PathBuf::from(root).join(rel);
    }
    PathBuf::from(dest)
}

fn sha256_file_capped(path: &Path, max_bytes: u64) -> Result<String, ShieldError> {
    let mut file = fs::File::open(path).map_err(|source| ShieldError::Io {
        context: "ota_baseline_open",
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    let mut total = 0_u64;
    loop {
        let n = file.read(&mut buf).map_err(|source| ShieldError::Io {
            context: "ota_baseline_read",
            source,
        })?;
        if n == 0 {
            break;
        }
        total = total.saturating_add(n as u64);
        if total > max_bytes {
            return Err(ShieldError::Observe(
                "ota rematch file exceeds 32 MiB".into(),
            ));
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("seat_ota_obs_{label}_{stamp}"))
    }

    #[test]
    fn vip_constant_and_match() {
        assert!(is_ota_portal_vip(OTA_PORTAL_VIP));
        assert!(is_ota_portal_vip("10.88.89.1"));
        assert!(!is_ota_portal_vip("10.88.89.2"));
        assert!(!is_ota_portal_vip("198.51.100.1"));
    }

    #[test]
    fn baseline_absent_is_missing() {
        let root = tmp_root("miss");
        let _ = fs::create_dir_all(&root);
        assert_eq!(observe_ota_baseline(&root), OtaBaselineStatus::Missing);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn baseline_ok_and_mismatch() {
        let root = tmp_root("ok");
        let shield = root.join("seat_shield");
        fs::create_dir_all(&shield).unwrap();
        let dest = root.join("runtime-bin");
        fs::write(&dest, b"hello-ota").unwrap();
        let digest = format!("{:x}", Sha256::digest(b"hello-ota"));
        let baseline = serde_json::json!({ dest.to_string_lossy().to_string(): digest });
        fs::write(
            shield.join(BASELINE_NAME),
            serde_json::to_vec_pretty(&baseline).unwrap(),
        )
        .unwrap();
        assert_eq!(observe_ota_baseline(&root), OtaBaselineStatus::Ok);

        fs::write(&dest, b"tampered").unwrap();
        assert_eq!(observe_ota_baseline(&root), OtaBaselineStatus::Mismatch);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn emit_staged_appends_event() {
        let root = tmp_root("emit");
        emit_ota_capsule_staged(&root, "build-1", "os_component").unwrap();
        let store = ShieldStore::open(&root).unwrap();
        let events = store.read_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "ota_capsule_staged");
        assert_eq!(events[0].entity_ip.as_deref(), Some(OTA_PORTAL_VIP));
        assert!(!events[0].explain_why.contains("planted-gated"));
        assert!(!events[0].explain_action.contains("plant (E4)"));
        assert!(events[0].explain_why.contains("stage, not cutover"));
        let _ = fs::remove_dir_all(&root);
    }
}
