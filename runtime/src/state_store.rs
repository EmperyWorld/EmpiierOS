use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const AUDIT_FILE: &str = "audit.jsonl";
const MAX_AUDIT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct StateStore {
    root: PathBuf,
    audit_lock: Arc<Mutex<()>>,
}

#[derive(Serialize)]
struct AuditEvent<'a> {
    timestamp_epoch: u64,
    operation: &'a str,
    status: &'a str,
    detail: &'a str,
}

impl StateStore {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn open(root: PathBuf) -> io::Result<Self> {
        // Traverse-only for the kiosk: listing stays closed, secrets stay 0600.
        ensure_secure_directory(&root, 0o751)?;
        Ok(Self {
            root,
            audit_lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn append_audit(&self, operation: &str, status: &str, detail: &str) -> io::Result<()> {
        let _guard = self
            .audit_lock
            .lock()
            .map_err(|_| io::Error::other("audit lock poisoned"))?;
        let path = self.root.join(AUDIT_FILE);
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if meta.file_type().is_symlink() || !meta.file_type().is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "audit path is not a regular file",
                ));
            }
            if meta.len() >= MAX_AUDIT_BYTES {
                return Err(io::Error::other("audit size limit reached"));
            }
        }

        let clean_detail = detail.replace(['\r', '\n'], " ");
        let event = AuditEvent {
            timestamp_epoch: now_epoch(),
            operation,
            status,
            detail: &clean_detail,
        };
        let mut line = serde_json::to_vec(&event).map_err(io::Error::other)?;
        line.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(path)?;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
        file.write_all(&line)?;
        file.sync_data()
    }

    pub fn record_start(&self) -> io::Result<u64> {
        let path = self.root.join("start_count");
        let current = match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() || !meta.file_type().is_file() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "start_count is not a regular file",
                ));
            }
            Ok(_) => fs::read_to_string(&path)?
                .trim()
                .parse::<u64>()
                .unwrap_or(0),
            Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error),
        };
        let next = current.saturating_add(1);
        atomic_write_file(&self.root, "start_count", format!("{next}\n").as_bytes(), 0o600)?;
        Ok(next)
    }
}

pub fn ensure_secure_directory(path: &Path, mode: u32) -> io::Result<()> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a secure directory",
            ));
        }
        if meta.uid() != unsafe { libc::geteuid() } {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "directory owner does not match runtime effective uid",
            ));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

pub fn atomic_write_file(dir: &Path, name: &str, contents: &[u8], mode: u32) -> io::Result<PathBuf> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.bytes().any(|b| b.is_ascii_control())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsafe destination name",
        ));
    }
    let meta = fs::symlink_metadata(dir)?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination parent is not a directory",
        ));
    }

    let nonce = format!("{}.{}.tmp", std::process::id(), now_epoch_nanos());
    let temp_path = dir.join(format!(".{name}.{nonce}"));
    let final_path = dir.join(name);
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(mode)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(mode))?;
        fs::rename(&temp_path, &final_path)?;
        OpenOptions::new().read(true).open(dir)?.sync_all()?;
        Ok(final_path.clone())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn now_epoch_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "empiieros-state-{label}-{}-{}",
            std::process::id(),
            now_epoch_nanos()
        ))
    }

    #[test]
    fn atomic_write_replaces_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("nofollow");
        fs::create_dir_all(&root).expect("root");
        let victim = root.join("victim");
        fs::write(&victim, b"do-not-touch").expect("victim");
        symlink(&victim, root.join("config")).expect("link");

        atomic_write_file(&root, "config", b"safe", 0o600).expect("replace");
        assert_eq!(fs::read(&victim).expect("victim read"), b"do-not-touch");
        assert_eq!(fs::read(root.join("config")).expect("config"), b"safe");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn state_store_rejects_symlink_root() {
        use std::os::unix::fs::symlink;

        let base = temp_dir("root-link");
        let target = base.join("target");
        fs::create_dir_all(&target).expect("target");
        let link = base.join("link");
        symlink(&target, &link).expect("link");
        assert!(StateStore::open(link).is_err());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn audit_permissions_are_repaired_before_append() {
        let root = temp_dir("audit-mode");
        let store = StateStore::open(root.clone()).expect("store");
        let audit = root.join(AUDIT_FILE);
        fs::write(&audit, b"").expect("audit");
        fs::set_permissions(&audit, fs::Permissions::from_mode(0o666)).expect("loose mode");

        store.append_audit("test", "ok", "detail").expect("append");

        let mode = fs::metadata(&audit).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn open_sets_traverse_mode_without_world_listing() {
        let root = temp_dir("traverse");
        let _store = StateStore::open(root.clone()).expect("store");
        let mode = fs::metadata(&root).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o751);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn start_count_increments_on_a_regular_file() {
        let root = temp_dir("starts");
        let store = StateStore::open(root.clone()).expect("store");
        assert_eq!(store.record_start().expect("first"), 1);
        assert_eq!(store.record_start().expect("second"), 2);
        assert_eq!(fs::read_to_string(root.join("start_count")).expect("read").trim(), "2");
        let _ = fs::remove_dir_all(root);
    }
}
