//! PKCE S256 + unpadded base64url randoms for native OIDC.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn random_b64url_32() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("oidc_random:{error}"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn pkce_challenge_s256(verifier: &str) -> Result<String, String> {
    if verifier.len() < 32 || verifier.len() > 128 {
        return Err("pkce_verifier".into());
    }
    let digest = Sha256::digest(verifier.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_is_s256_unpadded() {
        let verifier = "a".repeat(43);
        let challenge = pkce_challenge_s256(&verifier).unwrap();
        assert_eq!(challenge, pkce_challenge_s256(&verifier).unwrap());
        assert!(!challenge.contains('='));
        assert!(pkce_challenge_s256("short").is_err());
    }
}
