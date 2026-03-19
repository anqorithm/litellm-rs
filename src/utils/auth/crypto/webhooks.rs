//! Webhook and upload token signature utilities

use super::hmac::{constant_time_eq, create_hmac_signature};
use crate::utils::error::gateway_error::Result;
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a webhook signature
pub fn generate_webhook_signature(secret: &str, payload: &str, timestamp: u64) -> Result<String> {
    let data = format!("{}.{}", timestamp, payload);
    create_hmac_signature(secret, &data)
}

/// Verify webhook signature
pub fn verify_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: u64,
    signature: &str,
) -> Result<bool> {
    // Check timestamp is within acceptable range (e.g., 5 minutes)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| {
            crate::utils::error::gateway_error::GatewayError::Internal(format!(
                "System clock error: {}",
                e
            ))
        })?;

    if now.saturating_sub(timestamp) > 300 {
        return Ok(false); // Timestamp too old
    }

    let expected_signature = generate_webhook_signature(secret, payload, timestamp)?;
    Ok(constant_time_eq(&expected_signature, signature))
}

/// Generate a secure file upload token
pub fn generate_upload_token(user_id: &str, expires_at: u64) -> Result<String> {
    let data = format!("{}:{}", user_id, expires_at);
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize()))
}

/// Verify file upload token
pub fn verify_upload_token(token: &str, user_id: &str, expires_at: u64) -> Result<bool> {
    let expected_token = generate_upload_token(user_id, expires_at)?;
    Ok(constant_time_eq(&expected_token, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== generate_webhook_signature Tests ====================

    #[test]
    fn test_generate_webhook_signature_basic() {
        let secret = "webhook-secret";
        let payload = r#"{"event": "test"}"#;
        let timestamp = 1700000000u64;

        let signature = generate_webhook_signature(secret, payload, timestamp).unwrap();

        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // HMAC-SHA256 hex is 64 chars
    }

    #[test]
    fn test_generate_webhook_signature_consistency() {
        let secret = "test-secret";
        let payload = "test payload";
        let timestamp = 1700000000u64;

        let sig1 = generate_webhook_signature(secret, payload, timestamp).unwrap();
        let sig2 = generate_webhook_signature(secret, payload, timestamp).unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_generate_webhook_signature_different_timestamps() {
        let secret = "test-secret";
        let payload = "test payload";

        let sig1 = generate_webhook_signature(secret, payload, 1700000000).unwrap();
        let sig2 = generate_webhook_signature(secret, payload, 1700000001).unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_generate_webhook_signature_different_secrets() {
        let payload = "test payload";
        let timestamp = 1700000000u64;

        let sig1 = generate_webhook_signature("secret1", payload, timestamp).unwrap();
        let sig2 = generate_webhook_signature("secret2", payload, timestamp).unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_generate_webhook_signature_different_payloads() {
        let secret = "test-secret";
        let timestamp = 1700000000u64;

        let sig1 = generate_webhook_signature(secret, "payload1", timestamp).unwrap();
        let sig2 = generate_webhook_signature(secret, "payload2", timestamp).unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_generate_webhook_signature_hex_format() {
        let signature = generate_webhook_signature("secret", "payload", 1700000000).unwrap();
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ==================== verify_webhook_signature Tests ====================

    #[test]
    fn test_verify_webhook_signature_valid_recent() {
        let secret = "verify-test-secret";
        let payload = r#"{"event": "order.created"}"#;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = generate_webhook_signature(secret, payload, now).unwrap();
        let is_valid = verify_webhook_signature(secret, payload, now, &signature).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_verify_webhook_signature_expired() {
        let secret = "verify-test-secret";
        let payload = "test payload";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old_timestamp = now - 400; // 400 seconds ago (> 300 limit)

        let signature = generate_webhook_signature(secret, payload, old_timestamp).unwrap();
        let is_valid =
            verify_webhook_signature(secret, payload, old_timestamp, &signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_webhook_signature_wrong_secret() {
        let payload = "test payload";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = generate_webhook_signature("correct-secret", payload, now).unwrap();
        let is_valid = verify_webhook_signature("wrong-secret", payload, now, &signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_webhook_signature_tampered_payload() {
        let secret = "test-secret";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = generate_webhook_signature(secret, "original payload", now).unwrap();
        let is_valid =
            verify_webhook_signature(secret, "tampered payload", now, &signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_webhook_signature_invalid_signature() {
        let secret = "test-secret";
        let payload = "test payload";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let is_valid = verify_webhook_signature(secret, payload, now, "invalid-signature").unwrap();

        assert!(!is_valid);
    }

    // ==================== generate_upload_token Tests ====================

    #[test]
    fn test_generate_upload_token_basic() {
        let user_id = "user-123";
        let expires_at = 1700000000u64;

        let token = generate_upload_token(user_id, expires_at).unwrap();

        assert!(!token.is_empty());
        // URL-safe base64 without padding for SHA256 (32 bytes) = 43 chars
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn test_generate_upload_token_consistency() {
        let user_id = "user-456";
        let expires_at = 1700000000u64;

        let token1 = generate_upload_token(user_id, expires_at).unwrap();
        let token2 = generate_upload_token(user_id, expires_at).unwrap();

        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_upload_token_different_users() {
        let expires_at = 1700000000u64;

        let token1 = generate_upload_token("user-1", expires_at).unwrap();
        let token2 = generate_upload_token("user-2", expires_at).unwrap();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_upload_token_different_expiry() {
        let user_id = "user-123";

        let token1 = generate_upload_token(user_id, 1700000000).unwrap();
        let token2 = generate_upload_token(user_id, 1700000001).unwrap();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_upload_token_url_safe() {
        let token = generate_upload_token("user-test", 1700000000).unwrap();
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    // ==================== verify_upload_token Tests ====================

    #[test]
    fn test_verify_upload_token_valid() {
        let user_id = "user-verify";
        let expires_at = 1700000000u64;

        let token = generate_upload_token(user_id, expires_at).unwrap();
        let is_valid = verify_upload_token(&token, user_id, expires_at).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_verify_upload_token_wrong_user() {
        let expires_at = 1700000000u64;

        let token = generate_upload_token("correct-user", expires_at).unwrap();
        let is_valid = verify_upload_token(&token, "wrong-user", expires_at).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_upload_token_wrong_expiry() {
        let user_id = "user-123";

        let token = generate_upload_token(user_id, 1700000000).unwrap();
        let is_valid = verify_upload_token(&token, user_id, 1700000001).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_verify_upload_token_invalid_token() {
        let is_valid = verify_upload_token("invalid-token", "user-123", 1700000000).unwrap();
        assert!(!is_valid);
    }
}
