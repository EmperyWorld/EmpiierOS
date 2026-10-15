//! DPoP `iat` aligned to Keycloak HTTP Date.
//!
//! Living FoFline `native_oidc/dpop_iat.rs` / DESKTOP-ATOMIC §9.7 / §9.19:
//! Keycloak 26.7.1 `DPoPIsActiveCheck` window is roughly `[now-25, now+15]`
//! on the IdP clock. Local wall clock alone is not enough.

use std::time::{SystemTime, UNIX_EPOCH};

/// Keycloak `DPoPUtil.DEFAULT_PROOF_LIFETIME` (26.7.1).
#[allow(dead_code)]
pub const KC_DPOP_PROOF_LIFETIME_SECS: i64 = 10;
/// Keycloak `DPoPUtil.DEFAULT_ALLOWED_CLOCK_SKEW` (26.7.1).
pub const KC_DPOP_CLOCK_SKEW_SECS: i64 = 15;
/// Forward bias on HTTP Date so edge lag + RTT stay inside the past window.
pub const DPOP_HTTP_DATE_FORWARD_BIAS_SECS: u64 = 8;

pub fn local_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| item.as_secs())
        .unwrap_or(0)
}

/// Operator wall clock must be a real year. Installer/RTC junk (1970)
/// breaks air TLS and Keycloak DPoP even when the portal is up.
pub fn require_sane_wall_clock() -> Result<(), &'static str> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "wall_clock_unavailable")?
        .as_secs();
    if secs > i64::MAX as u64 {
        return Err("wall_clock_unavailable");
    }
    let mut now = secs as libc::time_t;
    let mut tm = std::mem::MaybeUninit::<libc::tm>::uninit();
    let ptr = unsafe { libc::localtime_r(&mut now, tm.as_mut_ptr()) };
    if ptr.is_null() {
        return Err("wall_clock_unavailable");
    }
    let year = unsafe { tm.assume_init().tm_year } + 1900;
    if !(2024..=2100).contains(&year) {
        return Err("wall_clock_required");
    }
    Ok(())
}

pub fn wall_clock_block() -> Option<&'static str> {
    require_sane_wall_clock().err()
}

pub fn wall_clock_is_sane() -> bool {
    require_sane_wall_clock().is_ok()
}

/// Parse IMF-fix HTTP `Date` (e.g. `Fri, 14 Aug 2026 05:01:00 GMT`) to unix seconds.
pub fn parse_http_date_to_unix(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    let rest = trimmed.split_once(", ")?.1;
    let mut parts = rest.split_whitespace();
    let day: u64 = parts.next()?.parse().ok()?;
    let month = month_num(parts.next()?)?;
    let year: i32 = parts.next()?.parse().ok()?;
    let time = parts.next()?;
    let zone = parts.next()?;
    if !zone.eq_ignore_ascii_case("GMT") && !zone.eq_ignore_ascii_case("UTC") {
        return None;
    }
    let mut hms = time.split(':');
    let hour: u64 = hms.next()?.parse().ok()?;
    let minute: u64 = hms.next()?.parse().ok()?;
    let second: u64 = hms.next()?.parse().ok()?;
    if day == 0 || day > 31 || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    civil_utc_to_unix(year, month, day, hour, minute, second)
}

#[allow(dead_code)]
pub fn iat_active_on_server(iat: i64, server_now: i64) -> bool {
    let skew = KC_DPOP_CLOCK_SKEW_SECS;
    let lifetime = KC_DPOP_PROOF_LIFETIME_SECS;
    iat <= server_now + skew && iat >= server_now - lifetime - skew
}

/// Date is the IdP clock SoT. `iat` must pass `DPoPIsActiveCheck` on that
/// clock. Do **not** clamp to `local+15`: with timesyncd masked the
/// installer clock can lag by minutes, and that clamp ages `iat` out of
/// the IdP past window (`oidc_http_400` / `invalid_dpop_proof`).
/// Date behind local (edge lag) → Date + 8s bias, still ≤ Date+15.
/// Never returns local when Date parsed.
pub fn choose_dpop_iat_from_http_date(server: u64, local: u64) -> u64 {
    if server >= local {
        return server;
    }
    server
        .saturating_add(DPOP_HTTP_DATE_FORWARD_BIAS_SECS)
        .min(server.saturating_add(KC_DPOP_CLOCK_SKEW_SECS as u64))
}

fn month_num(name: &str) -> Option<u64> {
    Some(match name {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    })
}

fn civil_utc_to_unix(
    year: i32,
    month: u64,
    day: u64,
    hour: u64,
    minute: u64,
    second: u64,
) -> Option<u64> {
    if year < 1970 {
        return None;
    }
    let mut days = 0i64;
    for item in 1970..year {
        days += if leap(item) { 366 } else { 365 };
    }
    let lengths = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for item in 1..month {
        days += i64::from(lengths[item as usize]);
        if item == 2 && leap(year) {
            days += 1;
        }
    }
    days += i64::try_from(day).ok()? - 1;
    let secs = days
        .checked_mul(86_400)?
        .checked_add(i64::try_from(hour).ok()?.checked_mul(3_600)?)?
        .checked_add(i64::try_from(minute).ok()?.checked_mul(60)?)?
        .checked_add(i64::try_from(second).ok()?)?;
    u64::try_from(secs).ok()
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_imf_fix_http_date() {
        let unix = parse_http_date_to_unix("Fri, 14 Aug 2026 05:01:00 GMT").unwrap();
        assert_eq!(unix, 1_786_683_660);
    }

    #[test]
    fn rejects_garbage_date() {
        assert!(parse_http_date_to_unix("not-a-date").is_none());
        assert!(parse_http_date_to_unix("").is_none());
    }

    #[test]
    fn kc_active_window_matches_26_7_1() {
        let now = 1_786_683_660_i64;
        assert!(iat_active_on_server(now, now));
        assert!(iat_active_on_server(now + 15, now));
        assert!(!iat_active_on_server(now + 16, now));
        assert!(iat_active_on_server(now - 25, now));
        assert!(!iat_active_on_server(now - 26, now));
    }

    #[test]
    fn http_date_forward_bias_keeps_lagged_edge_inside_past_window() {
        let local = 1_786_683_660_u64;
        let edge_behind = local - 22;
        let iat = choose_dpop_iat_from_http_date(edge_behind, local);
        assert_eq!(iat, edge_behind + DPOP_HTTP_DATE_FORWARD_BIAS_SECS);
        assert!(iat_active_on_server(iat as i64, local as i64));
        assert!(iat_active_on_server(iat as i64, edge_behind as i64));
    }

    #[test]
    fn date_ahead_of_local_keeps_date_not_local_fallback() {
        let local = 1_786_683_660_u64;
        let server = local + 8;
        let iat = choose_dpop_iat_from_http_date(server, local);
        assert_eq!(iat, server);
        assert_ne!(iat, local);
        assert!(iat_active_on_server(iat as i64, server as i64));
        assert!(iat_active_on_server(iat as i64, local as i64));
    }

    #[test]
    fn date_far_ahead_uses_idp_clock_not_local_cap() {
        let local = 1_786_683_660_u64;
        let server = local + 20;
        let iat = choose_dpop_iat_from_http_date(server, local);
        assert_eq!(iat, server);
        assert!(iat_active_on_server(iat as i64, server as i64));
    }

    #[test]
    fn date_minutes_ahead_stays_inside_idp_window() {
        let local = 1_786_683_660_u64;
        let server = local + 90;
        let iat = choose_dpop_iat_from_http_date(server, local);
        assert_eq!(iat, server);
        assert!(iat_active_on_server(iat as i64, server as i64));
        assert!(!iat_active_on_server(iat as i64, local as i64));
    }

    #[test]
    fn sane_wall_clock_accepts_current_host_year() {
        assert!(require_sane_wall_clock().is_ok());
        assert!(wall_clock_is_sane());
    }
}
