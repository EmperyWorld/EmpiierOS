//! Minimal JSON helpers (MIT; kinship with ewonika/probe/src/json.rs).

pub fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn opt_u32(value: Option<u32>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "null".into(),
    }
}

pub fn opt_str(value: &Option<String>) -> String {
    match value {
        Some(v) => escape(v),
        None => "null".into(),
    }
}
