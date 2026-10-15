//! NDJSON request/response — EmpiierOS Runtime protocol v1 aligned.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ShieldError, ShieldResult};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldRequest {
    pub id: String,
    pub op: String,
    #[serde(default)]
    pub params: Value,
}

impl ShieldRequest {
    pub fn validate(&self) -> ShieldResult<()> {
        if self.id.is_empty() || self.id.len() > 96 {
            return Err(ShieldError::InvalidRequest("invalid_request_id"));
        }
        if !self
            .id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(ShieldError::InvalidRequest("invalid_request_id"));
        }
        if self.op.is_empty() || self.op.len() > 96 {
            return Err(ShieldError::InvalidRequest("invalid_operation"));
        }
        if !self
            .op
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-'))
        {
            return Err(ShieldError::InvalidRequest("invalid_operation"));
        }
        if !self.params.is_object() && !self.params.is_null() {
            return Err(ShieldError::InvalidRequest("params_must_be_object"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ShieldErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ShieldResponse {
    pub protocol_version: u32,
    pub id: String,
    pub ok: bool,
    pub status: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ShieldErrorBody>,
}

impl ShieldResponse {
    #[must_use]
    pub fn success(id: impl Into<String>, status: impl Into<String>, data: Value) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            id: id.into(),
            ok: true,
            status: status.into(),
            data,
            error: None,
        }
    }

    #[must_use]
    pub fn from_error(id: impl Into<String>, err: &ShieldError) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            id: id.into(),
            ok: false,
            status: err.httpish_status().into(),
            data: Value::Null,
            error: Some(ShieldErrorBody {
                code: err.code().into(),
                message: err.to_string(),
            }),
        }
    }

    #[must_use]
    pub fn encode_line(&self) -> Vec<u8> {
        let mut encoded = serde_json::to_vec(self).unwrap_or_else(|_| {
            br#"{"protocol_version":1,"id":"","ok":false,"status":"internal_error","data":null,"error":{"code":"serialization_failed","message":"response serialization failed"}}"#
                .to_vec()
        });
        encoded.push(b'\n');
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_rejects_unknown_fields_and_bad_identifiers() {
        assert!(serde_json::from_str::<ShieldRequest>(
            r#"{"id":"r1","op":"seat.shield.status","params":{},"extra":true}"#
        )
        .is_err());

        let request: ShieldRequest =
            serde_json::from_str(r#"{"id":"../x","op":"seat.shield.status","params":{}}"#)
                .expect("parse");
        assert!(matches!(
            request.validate(),
            Err(ShieldError::InvalidRequest("invalid_request_id"))
        ));
    }

    #[test]
    fn response_is_single_json_line() {
        let line = ShieldResponse::success("r1", "observed", serde_json::json!({"ready": true}))
            .encode_line();
        assert_eq!(line.last(), Some(&b'\n'));
        let value: Value = serde_json::from_slice(&line[..line.len() - 1]).expect("json");
        assert_eq!(value["ok"], true);
        assert_eq!(value["protocol_version"], PROTOCOL_VERSION);
    }
}
