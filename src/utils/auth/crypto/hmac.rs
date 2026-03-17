//! HMAC signature creation and verification

use crate::utils::error::gateway_error::{GatewayError, Result};
use hmac::{Hmac, Mac, digest::KeyInit as HmacKeyInit};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Create HMAC signature
pub fn create_hmac_signature(secret: &str, data: &str) -> Result<String> {
    let mut mac = <HmacSha256 as HmacKeyInit>::new_from_slice(secret.as_bytes())
        .map_err(|e| GatewayError::Auth(format!("Invalid HMAC key: {}", e)))?;

    mac.update(data.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Verify HMAC signature
pub fn verify_hmac_signature(secret: &str, data: &str, signature: &str) -> Result<bool> {
    let expected_signature = create_hmac_signature(secret, data)?;
    Ok(constant_time_eq(&expected_signature, signature))
}

/// Constant-time string comparison
pub(crate) fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== create_hmac_signature Tests ====================

    #[test]
    fn test_create_hmac_signature_basic() {
        let secret = "my-secret-key";
        let data = "Hello, World!";

        let signature = create_hmac_signature(secret, data).unwrap();

        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_create_hmac_signature_consistency() {
        let secret = "test-key";
        let data = "test-data";

        let sig1 = create_hmac_signature(secret, data).unwrap();
        let sig2 = create_hmac_signature(secret, data).unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_create_hmac_signature_different_data() {
        let secret = "same-key";

        let sig1 = create_hmac_signature(secret, "data1").unwrap();
        let sig2 = create_hmac_signature(secret, "data2").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_create_hmac_signature_different_key() {
        let data = "same-data";

        let sig1 = create_hmac_signature("key1", data).unwrap();
        let sig2 = create_hmac_signature("key2", data).unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_create_hmac_signature_hex_format() {
        let signature = create_hmac_signature("key", "data").unwrap();
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_create_hmac_signature_empty_data() {
        let signature = create_hmac_signature("key", "").unwrap();
        assert_eq!(signature.len(), 64);
    }

    // ==================== verify_hmac_signature Tests ====================

    #[test]
    fn test_verify_hmac_signature_valid() {
        let secret = "verify-test-key";
        let data = "message to verify";

        let signature = create_hmac_signature(secret, data).unwrap();
        let is_valid = verify_hmac_signature(secret, data, &signature).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_verify_hmac_signature_invalid_signature() {
        let secret = "test-key";
        let data = "test-data";

        let is_valid = verify_hmac_signature(secret, data, "invalid-signature").unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_hmac_signature_wrong_secret() {
        let data = "test-data";

        let signature = create_hmac_signature("correct-key", data).unwrap();
        let is_valid = verify_hmac_signature("wrong-key", data, &signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_hmac_signature_tampered_data() {
        let secret = "tamper-test";
        let original_data = "original data";

        let signature = create_hmac_signature(secret, original_data).unwrap();
        let is_valid = verify_hmac_signature(secret, "tampered data", &signature).unwrap();

        assert!(!is_valid);
    }

    // ==================== constant_time_eq Tests ====================

    #[test]
    fn test_constant_time_eq_equal() {
        assert!(constant_time_eq("hello", "hello"));
    }

    #[test]
    fn test_constant_time_eq_not_equal() {
        assert!(!constant_time_eq("hello", "world"));
    }

    #[test]
    fn test_constant_time_eq_different_length() {
        assert!(!constant_time_eq("short", "longer-string"));
    }

    #[test]
    fn test_constant_time_eq_empty() {
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn test_constant_time_eq_single_char_diff() {
        assert!(!constant_time_eq("hellO", "hello"));
    }
}
