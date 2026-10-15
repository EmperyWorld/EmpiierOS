//! Wire-compatible Runtime NDJSON v1 response for Seat Browser.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct BrowserRequest {
    pub id: String,
    pub op: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserResponse {
    pub protocol_version: u32,
    pub id: String,
    pub ok: bool,
    pub status: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BrowserError>,
}

impl BrowserResponse {
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

    pub fn failure(
        id: impl Into<String>,
        status: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            id: id.into(),
            ok: false,
            status: status.into(),
            data: json!({}),
            error: Some(BrowserError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    #[must_use]
    pub fn encode_line(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap_or_else(|_| {
            br#"{"protocol_version":1,"id":"","ok":false,"status":"failed","data":{},"error":{"code":"serialize","message":"encode failed"}}"#.to_vec()
        });
        bytes.push(b'\n');
        bytes
    }
}
