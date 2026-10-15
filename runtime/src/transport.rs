//! Transport & local SOCKS listener probe (P59).
//!
//! Harvested pattern from living EliteEW `w-transport-core` — status read only.
//! WireGuard kernel / userspace smoltcp stack: DROP.
//! Not wired into `launcher.status` yet (env probe races unit tests).

#![allow(dead_code)]

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub const BACKEND_LOOPBACK_SOCKS: &str = "loopback_socks";
pub const BACKEND_UNAVAILABLE: &str = "unavailable";

pub const REASON_SOCKS_LISTENING: &str = "socks_listening";
pub const REASON_SOCKS_DOWN: &str = "socks_down";
pub const REASON_PROBE_DISABLED: &str = "probe_disabled";

pub const DEFAULT_SOCKS_PORT: u16 = 1080;

/// Lab/smoke opt-out: `EMPIIEROS_TRANSPORT_PROBE=0` forces unavailable stub.
pub fn probe_enabled_from_env() -> bool {
    !matches!(
        std::env::var("EMPIIEROS_TRANSPORT_PROBE").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

pub fn socks_port_from_env() -> u16 {
    std::env::var("EMPIIEROS_TRANSPORT_SOCKS_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_SOCKS_PORT)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportSnapshot {
    pub backend: String,
    pub listening: bool,
    pub protected: bool,
    pub proxy_url: Option<String>,
    pub reason: String,
    pub detail: String,
}

pub fn probe_socks_listener(addr: SocketAddr, timeout: Duration) -> bool {
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

pub fn probe_transport() -> TransportSnapshot {
    if !probe_enabled_from_env() {
        return TransportSnapshot {
            backend: BACKEND_UNAVAILABLE.into(),
            listening: false,
            protected: false,
            proxy_url: None,
            reason: REASON_PROBE_DISABLED.into(),
            detail: "transport.status · probe disabled by env (EMPIIEROS_TRANSPORT_PROBE=0)".into(),
        };
    }
    let port = socks_port_from_env();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let timeout = Duration::from_millis(50);
    if probe_socks_listener(addr, timeout) {
        let proxy_url = format!("socks5://127.0.0.1:{port}");
        TransportSnapshot {
            backend: BACKEND_LOOPBACK_SOCKS.into(),
            listening: true,
            protected: false,
            proxy_url: Some(proxy_url),
            reason: REASON_SOCKS_LISTENING.into(),
            detail: format!(
                "transport.status · SOCKS5 listening on 127.0.0.1:{port} (protected:false; not L7)"
            ),
        }
    } else {
        TransportSnapshot {
            backend: BACKEND_UNAVAILABLE.into(),
            listening: false,
            protected: false,
            proxy_url: None,
            reason: REASON_SOCKS_DOWN.into(),
            detail: format!(
                "transport.status · SOCKS5 port {port} down (connection refused; tunnel inactive)"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn probe_disabled_via_env() {
        std::env::set_var("EMPIIEROS_TRANSPORT_PROBE", "0");
        let snap = probe_transport();
        assert!(!snap.listening);
        assert!(!snap.protected);
        assert_eq!(snap.reason, REASON_PROBE_DISABLED);
        assert_eq!(snap.backend, BACKEND_UNAVAILABLE);
        assert!(snap.proxy_url.is_none());
        std::env::remove_var("EMPIIEROS_TRANSPORT_PROBE");
    }

    #[test]
    fn probe_detects_active_listener() {
        std::env::remove_var("EMPIIEROS_TRANSPORT_PROBE");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        std::env::set_var("EMPIIEROS_TRANSPORT_SOCKS_PORT", port.to_string());

        let snap = probe_transport();
        assert!(snap.listening);
        assert!(!snap.protected);
        assert_eq!(snap.reason, REASON_SOCKS_LISTENING);
        assert_eq!(snap.backend, BACKEND_LOOPBACK_SOCKS);
        assert_eq!(
            snap.proxy_url.as_deref(),
            Some(format!("socks5://127.0.0.1:{port}").as_str())
        );

        drop(listener);
        std::env::remove_var("EMPIIEROS_TRANSPORT_SOCKS_PORT");
    }

    #[test]
    fn probe_detects_inactive_port() {
        std::env::remove_var("EMPIIEROS_TRANSPORT_PROBE");
        // Bind and immediately drop to find an unused port
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind temp");
            listener.local_addr().expect("port").port()
        };
        std::env::set_var("EMPIIEROS_TRANSPORT_SOCKS_PORT", port.to_string());

        let snap = probe_transport();
        assert!(!snap.listening);
        assert!(!snap.protected);
        assert_eq!(snap.reason, REASON_SOCKS_DOWN);
        assert_eq!(snap.backend, BACKEND_UNAVAILABLE);
        assert!(snap.proxy_url.is_none());

        std::env::remove_var("EMPIIEROS_TRANSPORT_SOCKS_PORT");
    }
}
