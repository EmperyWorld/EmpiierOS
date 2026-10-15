//! Default production actuator — always fail closed.

use crate::act::FirewallActuator;
use crate::error::{ShieldError, ShieldResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct UnavailableActuator;

impl FirewallActuator for UnavailableActuator {
    fn backend_name(&self) -> &'static str {
        "unavailable"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn block_peer(&mut self, _peer_ip: &str, _reason: &str) -> ShieldResult<()> {
        Err(ShieldError::NotAvailable(
            "floor firewall helper is not available".into(),
        ))
    }

    fn unblock_peer(&mut self, _peer_ip: &str) -> ShieldResult<bool> {
        Err(ShieldError::NotAvailable(
            "floor firewall helper is not available".into(),
        ))
    }

    fn list_blocked(&self) -> ShieldResult<Vec<String>> {
        Ok(Vec::new())
    }
}
