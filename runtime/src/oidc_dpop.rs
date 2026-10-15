//! P-256 DPoP JWK thumbprint (RFC 7638) and ES256 proof (RFC 9449).

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

fn b64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn signing_key(secret: &[u8; 32]) -> Result<SigningKey, String> {
    SigningKey::from_slice(secret).map_err(|error| format!("dpop_key:{error}"))
}

fn public_xy(secret: &[u8; 32]) -> Result<(String, String), String> {
    let encoded = signing_key(secret)?.verifying_key().to_encoded_point(false);
    let x = encoded.x().ok_or_else(|| "dpop_x".to_string())?;
    let y = encoded.y().ok_or_else(|| "dpop_y".to_string())?;
    Ok((b64url(x.as_slice()), b64url(y.as_slice())))
}

pub fn generate_secret() -> Result<[u8; 32], String> {
    let mut secret = [0u8; 32];
    for _ in 0..8 {
        getrandom::getrandom(&mut secret).map_err(|error| format!("dpop_random:{error}"))?;
        if signing_key(&secret).is_ok() {
            return Ok(secret);
        }
    }
    secret.zeroize();
    Err("dpop_random".into())
}

pub fn jkt_from_secret(secret: &[u8; 32]) -> Result<String, String> {
    let (x, y) = public_xy(secret)?;
    let canonical = format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#);
    Ok(b64url(&Sha256::digest(canonical.as_bytes())))
}

pub fn sign_dpop_proof(
    secret: &[u8; 32],
    htm: &str,
    htu: &str,
    iat: u64,
    jti: &str,
) -> Result<String, String> {
    let method = htm.trim().to_ascii_uppercase();
    if method != "POST" {
        return Err("dpop_htm".into());
    }
    let htu_n = htu.trim();
    if htu_n.is_empty() || htu_n.contains('?') || htu_n.contains('#') {
        return Err("dpop_htu".into());
    }
    let (x, y) = public_xy(secret)?;
    let header = serde_json::json!({
        "typ": "dpop+jwt",
        "alg": "ES256",
        "jwk": { "kty": "EC", "crv": "P-256", "x": x, "y": y }
    });
    let payload = serde_json::json!({
        "jti": jti,
        "htm": method,
        "htu": htu_n,
        "iat": iat
    });
    let signing_input = format!(
        "{}.{}",
        b64url(header.to_string().as_bytes()),
        b64url(payload.to_string().as_bytes())
    );
    let signature: Signature = signing_key(secret)?.sign(signing_input.as_bytes());
    Ok(format!("{signing_input}.{}", b64url(&signature.to_bytes())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_secret() -> [u8; 32] {
        let bytes = hex::decode("c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721")
            .unwrap();
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        out
    }

    #[test]
    fn jkt_is_stable_and_public_only() {
        let jkt = jkt_from_secret(&sample_secret()).unwrap();
        assert_eq!(jkt, jkt_from_secret(&sample_secret()).unwrap());
        assert!(!jkt.contains('='));
        let proof = sign_dpop_proof(
            &sample_secret(),
            "POST",
            "https://auth.eliteew.ru/realms/empery/protocol/openid-connect/token",
            1_700_000_000,
            "jti-1",
        )
        .unwrap();
        assert_eq!(proof.split('.').count(), 3);
        assert!(!proof.contains("\"d\""));
        assert!(sign_dpop_proof(&sample_secret(), "POST", "https://auth.eliteew.ru/token?x=1", 1, "j").is_err());
    }
}
