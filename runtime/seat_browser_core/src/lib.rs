//! EmpiierOS Seat Browser — pure decision logic (prep bay).
//!
//! Not the Runtime daemon and not a WebKit host.
//! Harvest lockstep: EliteEW EBrowser `ebrowser_nav_policy` semantics.
//!
//! Product SoT: `docs/apps/SeatBrowser/EMPIIEROS-SEAT-BROWSER.md`

pub mod bridge;
pub mod downloads;
pub mod policy;
pub mod protocol;
pub mod service;
pub mod tabs;

pub use bridge::{handle_runtime_line, is_seat_browser_op, SEAT_BROWSER_OPS};
pub use policy::{
    default_missing_policy, evaluate_navigation, evaluate_navigation_str, validate_policy_for_save,
    DenyReason, NavDecision, PolicyMode, PolicySnapshot, POLICY_TTL_SECONDS,
};
pub use service::{BrowserService, ServiceConfig};
