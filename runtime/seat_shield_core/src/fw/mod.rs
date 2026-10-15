//! Floor helper backends for `seat-shield-fw`.
//!
//! Same argv contract for every backend. No shell strings.

pub(crate) mod nft;
mod state_file;

use std::path::PathBuf;

pub use nft::{NftBackend, NftConfig, DEFAULT_NFT_BIN};
pub use state_file::StateFileBackend;

/// Selected storage/enforcement plane behind the helper argv contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FwBackendKind {
    StateFile,
    Nft,
}

impl FwBackendKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StateFile => "state",
            Self::Nft => "nft",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "state" | "statefile" | "file" => Ok(Self::StateFile),
            "nft" => Ok(Self::Nft),
            other => Err(format!("unknown --backend: {other} (state|nft)")),
        }
    }
}

/// Mutable peer-block plane.
pub trait FwBackend {
    fn block(&mut self, peer_ip: &str) -> Result<(), String>;
    fn unblock(&mut self, peer_ip: &str) -> Result<bool, String>;
    fn list(&self) -> Result<Vec<String>, String>;
}

/// Parsed helper CLI (global flags + action).
#[derive(Debug, Clone)]
pub struct FwCli {
    pub state_dir: PathBuf,
    pub backend: FwBackendKind,
    pub nft: NftConfig,
    pub action: FwAction,
}

#[derive(Debug, Clone)]
pub enum FwAction {
    Block { peer_ip: String, reason: String },
    Unblock { peer_ip: String },
    List,
}

/// Flags Runtime may pin ahead of `--state-dir` when invoking the helper.
pub const HELPER_PREFIX_FLAGS: &[&str] = &[
    "--backend",
    "--nft-bin",
    "--nft-family",
    "--nft-table",
    "--nft-set",
    "--nft-set6",
];

/// Validate allowlisted helper prefix argv (flag/value pairs only).
pub fn validate_helper_prefix(prefix: &[String]) -> Result<(), String> {
    if prefix.len() % 2 != 0 {
        return Err("helper prefix must be flag/value pairs".into());
    }
    let mut i = 0;
    while i < prefix.len() {
        let flag = &prefix[i];
        let value = &prefix[i + 1];
        if !HELPER_PREFIX_FLAGS.contains(&flag.as_str()) {
            return Err(format!("helper prefix flag not allowlisted: {flag}"));
        }
        if value.is_empty() || value.contains('\0') {
            return Err(format!("invalid value for {flag}"));
        }
        if value.starts_with('-') {
            return Err(format!("{flag} value must not look like a flag"));
        }
        if flag == "--nft-bin" && !value.starts_with('/') {
            return Err("--nft-bin must be absolute".into());
        }
        if flag == "--backend" {
            FwBackendKind::parse(value)?;
        }
        if matches!(
            flag.as_str(),
            "--nft-family" | "--nft-table" | "--nft-set" | "--nft-set6"
        ) {
            nft::validate_nft_name(flag, value)?;
        }
        i += 2;
    }
    Ok(())
}

pub fn parse_cli(args: &[String]) -> Result<FwCli, String> {
    let mut state_dir: Option<PathBuf> = None;
    let mut backend = FwBackendKind::StateFile;
    let mut nft = NftConfig::default();
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--state-dir" => {
                i += 1;
                let p = args.get(i).ok_or("--state-dir requires path")?;
                state_dir = Some(PathBuf::from(p));
            }
            "--backend" => {
                i += 1;
                let v = args.get(i).ok_or("--backend requires value")?;
                backend = FwBackendKind::parse(v)?;
            }
            "--nft-bin" => {
                i += 1;
                let v = args.get(i).ok_or("--nft-bin requires path")?;
                nft.nft_bin = PathBuf::from(v);
            }
            "--nft-family" => {
                i += 1;
                let v = args.get(i).ok_or("--nft-family requires value")?;
                nft::validate_nft_name("--nft-family", v)?;
                nft.family = v.clone();
            }
            "--nft-table" => {
                i += 1;
                let v = args.get(i).ok_or("--nft-table requires value")?;
                nft::validate_nft_name("--nft-table", v)?;
                nft.table = v.clone();
            }
            "--nft-set" => {
                i += 1;
                let v = args.get(i).ok_or("--nft-set requires value")?;
                nft::validate_nft_name("--nft-set", v)?;
                nft.set_v4 = v.clone();
            }
            "--nft-set6" => {
                i += 1;
                let v = args.get(i).ok_or("--nft-set6 requires value")?;
                nft::validate_nft_name("--nft-set6", v)?;
                nft.set_v6 = v.clone();
            }
            other => rest.push(other.to_string()),
        }
        i += 1;
    }

    let state_dir = state_dir.ok_or("--state-dir ABS_DIR is required")?;
    if !state_dir.is_absolute() {
        return Err("--state-dir must be absolute".into());
    }
    if backend == FwBackendKind::Nft {
        nft.validate()?;
    }

    let action_name = rest.first().map(String::as_str).unwrap_or("");
    let action = match action_name {
        "block" => {
            let peer_ip = require_flag(&rest, "--ip")?;
            validate_ip(&peer_ip)?;
            let reason = optional_flag(&rest, "--reason").unwrap_or_default();
            FwAction::Block { peer_ip, reason }
        }
        "unblock" => {
            let peer_ip = require_flag(&rest, "--ip")?;
            validate_ip(&peer_ip)?;
            FwAction::Unblock { peer_ip }
        }
        "list" => FwAction::List,
        "" => return Err("action required: block|unblock|list".into()),
        other => return Err(format!("unknown action: {other}")),
    };

    Ok(FwCli {
        state_dir,
        backend,
        nft,
        action,
    })
}

/// Open the selected backend and apply one action. Returns JSON-ready fields.
pub fn dispatch(cli: &FwCli) -> Result<FwOutcome, String> {
    match cli.backend {
        FwBackendKind::StateFile => {
            let mut backend = StateFileBackend::open(&cli.state_dir)?;
            apply(&mut backend, &cli.action)
        }
        FwBackendKind::Nft => {
            let mut backend = NftBackend::new(cli.nft.clone())?;
            apply(&mut backend, &cli.action)
        }
    }
}

fn apply(backend: &mut dyn FwBackend, action: &FwAction) -> Result<FwOutcome, String> {
    match action {
        FwAction::Block { peer_ip, reason: _ } => {
            backend.block(peer_ip)?;
            Ok(FwOutcome::Block {
                peer_ip: peer_ip.clone(),
            })
        }
        FwAction::Unblock { peer_ip } => {
            let removed = backend.unblock(peer_ip)?;
            Ok(FwOutcome::Unblock {
                peer_ip: peer_ip.clone(),
                removed,
            })
        }
        FwAction::List => Ok(FwOutcome::List {
            peers: backend.list()?,
        }),
    }
}

#[derive(Debug, Clone)]
pub enum FwOutcome {
    Block { peer_ip: String },
    Unblock { peer_ip: String, removed: bool },
    List { peers: Vec<String> },
}

fn require_flag(args: &[String], flag: &str) -> Result<String, String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            return args
                .get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{flag} requires a value"));
        }
        i += 1;
    }
    Err(format!("{flag} is required"))
}

fn optional_flag(args: &[String], flag: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

fn validate_ip(raw: &str) -> Result<(), String> {
    raw.parse::<std::net::IpAddr>()
        .map(|_| ())
        .map_err(|_| format!("invalid IP: {raw}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn parse_state_block() {
        let cli = parse_cli(&[
            "--state-dir".into(),
            "/tmp/ss".into(),
            "block".into(),
            "--ip".into(),
            "198.51.100.20".into(),
        ])
        .unwrap();
        assert_eq!(cli.backend, FwBackendKind::StateFile);
        match cli.action {
            FwAction::Block { peer_ip, .. } => assert_eq!(peer_ip, "198.51.100.20"),
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn parse_nft_requires_absolute_bin() {
        let err = parse_cli(&[
            "--state-dir".into(),
            "/tmp/ss".into(),
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            "nft".into(),
            "list".into(),
        ])
        .unwrap_err();
        assert!(err.contains("absolute"), "{err}");
    }

    #[test]
    fn state_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "seat-shield-fw-state-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let cli = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "block".into(),
            "--ip".into(),
            "203.0.113.9".into(),
        ])
        .unwrap();
        match dispatch(&cli).unwrap() {
            FwOutcome::Block { peer_ip } => assert_eq!(peer_ip, "203.0.113.9"),
            _ => panic!("block"),
        }
        let list = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "list".into(),
        ])
        .unwrap();
        match dispatch(&list).unwrap() {
            FwOutcome::List { peers } => assert_eq!(peers, vec!["203.0.113.9".to_string()]),
            _ => panic!("list"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nft_via_fake_bin() {
        let dir = std::env::temp_dir().join(format!(
            "seat-shield-fw-nft-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("fake-nft");
        let store = dir.join("nft_store.json");
        // Fake nft: argv-only recorder with JSON list.
        let script = format!(
            r#"#!/usr/bin/env python3
import json, sys
from pathlib import Path
store = Path({store:?})
args = sys.argv[1:]
def load():
    if store.exists():
        return json.loads(store.read_text())
    return {{"peers": []}}
def save(data):
    store.write_text(json.dumps(data))
if args[:1] == ["-j"] and args[1:2] == ["list"]:
    data = load()
    elems = [{{"elem": {{"val": p}}}} for p in data["peers"]]
    print(json.dumps({{"nftables": [{{"set": {{"elem": elems}}}}]}}))
    sys.exit(0)
if args[:2] == ["add", "element"]:
    ip = args[args.index("{{")+1]
    data = load()
    if ip not in data["peers"]:
        data["peers"].append(ip)
        save(data)
    sys.exit(0)
if args[:2] == ["delete", "element"]:
    ip = args[args.index("{{")+1]
    data = load()
    if ip in data["peers"]:
        data["peers"].remove(ip)
        save(data)
        sys.exit(0)
    sys.stderr.write("Error: Could not find element\n")
    sys.exit(1)
sys.stderr.write("unexpected: " + " ".join(args) + "\n")
sys.exit(2)
"#
        );
        fs::write(&fake, script).unwrap();
        let mut perms = fs::metadata(&fake).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake, perms).unwrap();

        let block = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            fake.to_string_lossy().into(),
            "--nft-table".into(),
            "seat_shield".into(),
            "--nft-set".into(),
            "blocked4".into(),
            "--nft-set6".into(),
            "blocked6".into(),
            "block".into(),
            "--ip".into(),
            "198.51.100.44".into(),
        ])
        .unwrap();
        dispatch(&block).unwrap();

        let block6 = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            fake.to_string_lossy().into(),
            "--nft-table".into(),
            "seat_shield".into(),
            "block".into(),
            "--ip".into(),
            "2001:db8::44".into(),
        ])
        .unwrap();
        dispatch(&block6).unwrap();

        let list = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            fake.to_string_lossy().into(),
            "--nft-table".into(),
            "seat_shield".into(),
            "list".into(),
        ])
        .unwrap();
        match dispatch(&list).unwrap() {
            FwOutcome::List { peers } => {
                assert!(peers.contains(&"198.51.100.44".to_string()));
                assert!(peers.contains(&"2001:db8::44".to_string()));
            }
            _ => panic!("list"),
        }

        let unblock = parse_cli(&[
            "--state-dir".into(),
            dir.to_string_lossy().into(),
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            fake.to_string_lossy().into(),
            "--nft-table".into(),
            "seat_shield".into(),
            "unblock".into(),
            "--ip".into(),
            "198.51.100.44".into(),
        ])
        .unwrap();
        match dispatch(&unblock).unwrap() {
            FwOutcome::Unblock { removed, .. } => assert!(removed),
            _ => panic!("unblock"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn helper_prefix_allowlist() {
        validate_helper_prefix(&[
            "--backend".into(),
            "nft".into(),
            "--nft-bin".into(),
            "/usr/sbin/nft".into(),
        ])
        .unwrap();
        assert!(validate_helper_prefix(&["--evil".into(), "x".into()]).is_err());
    }
}
