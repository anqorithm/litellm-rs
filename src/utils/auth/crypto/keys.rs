//! Key and token generation utilities

use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac, digest::KeyInit as HmacKeyInit};
use rand::{Rng, distr::Alphanumeric};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Generate a secure API key
pub fn generate_api_key() -> String {
    let prefix = "gw";
    let random_part: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    format!("{}-{}", prefix, random_part)
}

/// Generate a JWT secret
pub fn generate_jwt_secret() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// Generate a secure random token
pub fn generate_token(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a secure session token
pub fn generate_session_token() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

/// Hash API key for storage.
///
/// When `hmac_secret` is `Some`, computes `HMAC-SHA256(secret, api_key)`.
/// When `hmac_secret` is `None`, falls back to plain `SHA-256(api_key)` for
/// backward compatibility.
pub fn hash_api_key(api_key: &str, hmac_secret: Option<&str>) -> String {
    match hmac_secret {
        Some(secret) => {
            // HMAC accepts keys of any size; new_from_slice only fails for
            // algorithms with a fixed key requirement, which HMAC does not have.
            // Using unwrap_or_else to satisfy lint while keeping the infallible
            // return type — a failure here would indicate a broken crypto backend.
            let mut mac = match <HmacSha256 as HmacKeyInit>::new_from_slice(secret.as_bytes()) {
                Ok(m) => m,
                Err(_) => {
                    // Unreachable for HMAC, but fall back to plain SHA-256 rather
                    // than panicking.
                    let mut hasher = Sha256::new();
                    hasher.update(api_key.as_bytes());
                    return hex::encode(hasher.finalize());
                }
            };
            mac.update(api_key.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        None => {
            let mut hasher = Sha256::new();
            hasher.update(api_key.as_bytes());
            hex::encode(hasher.finalize())
        }
    }
}

/// Generate API key prefix for identification
pub fn extract_api_key_prefix(api_key: &str) -> String {
    if api_key.len() >= 8 {
        format!("{}...{}", &api_key[..4], &api_key[api_key.len() - 4..])
    } else {
        api_key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== generate_api_key Tests ====================

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("gw-"));
    }

    #[test]
    fn test_generate_api_key_length() {
        let key = generate_api_key();
        assert_eq!(key.len(), 35); // "gw-" (3) + 32 random chars
    }

    #[test]
    fn test_generate_api_key_uniqueness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_api_key_alphanumeric() {
        let key = generate_api_key();
        let random_part = &key[3..];
        assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    // ==================== generate_jwt_secret Tests ====================

    #[test]
    fn test_generate_jwt_secret_length() {
        let secret = generate_jwt_secret();
        assert_eq!(secret.len(), 64);
    }

    #[test]
    fn test_generate_jwt_secret_uniqueness() {
        let secret1 = generate_jwt_secret();
        let secret2 = generate_jwt_secret();
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_generate_jwt_secret_alphanumeric() {
        let secret = generate_jwt_secret();
        assert!(secret.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    // ==================== generate_token Tests ====================

    #[test]
    fn test_generate_token_length() {
        let token = generate_token(16);
        assert_eq!(token.len(), 16);
    }

    #[test]
    fn test_generate_token_zero_length() {
        let token = generate_token(0);
        assert!(token.is_empty());
    }

    #[test]
    fn test_generate_token_large_length() {
        let token = generate_token(1000);
        assert_eq!(token.len(), 1000);
    }

    #[test]
    fn test_generate_token_alphanumeric() {
        let token = generate_token(100);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let token1 = generate_token(32);
        let token2 = generate_token(32);
        assert_ne!(token1, token2);
    }

    // ==================== generate_session_token Tests ====================

    #[test]
    fn test_generate_session_token_length() {
        let token = generate_session_token();
        assert_eq!(token.len(), 43); // 32 bytes -> 43 chars in URL-safe base64 without padding
    }

    #[test]
    fn test_generate_session_token_uniqueness() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_session_token_url_safe() {
        let token = generate_session_token();
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    // ==================== hash_api_key Tests (plain SHA-256 fallback) ====================

    #[test]
    fn test_hash_api_key_length() {
        let hash = hash_api_key("test-key", None);
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_hash_api_key_consistency() {
        let hash1 = hash_api_key("same-key", None);
        let hash2 = hash_api_key("same-key", None);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_different_keys() {
        let hash1 = hash_api_key("key1", None);
        let hash2 = hash_api_key("key2", None);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_empty() {
        let hash = hash_api_key("", None);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_api_key_hex_format() {
        let hash = hash_api_key("test", None);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ==================== hash_api_key Tests (HMAC-SHA256) ====================

    #[test]
    fn test_hash_api_key_hmac_length() {
        let hash = hash_api_key("test-key", Some("server-secret"));
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_api_key_hmac_consistency() {
        let hash1 = hash_api_key("same-key", Some("secret"));
        let hash2 = hash_api_key("same-key", Some("secret"));
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_hmac_differs_from_plain() {
        let plain = hash_api_key("test-key", None);
        let hmac = hash_api_key("test-key", Some("secret"));
        assert_ne!(plain, hmac);
    }

    #[test]
    fn test_hash_api_key_hmac_different_secrets() {
        let hash1 = hash_api_key("same-key", Some("secret1"));
        let hash2 = hash_api_key("same-key", Some("secret2"));
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_hmac_hex_format() {
        let hash = hash_api_key("test", Some("secret"));
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ==================== extract_api_key_prefix Tests ====================

    #[test]
    fn test_extract_api_key_prefix_long() {
        let prefix = extract_api_key_prefix("gw-abcdefghijklmnop");
        assert_eq!(prefix, "gw-a...mnop");
    }

    #[test]
    fn test_extract_api_key_prefix_exact_8() {
        let prefix = extract_api_key_prefix("12345678");
        assert_eq!(prefix, "1234...5678");
    }

    #[test]
    fn test_extract_api_key_prefix_short() {
        let prefix = extract_api_key_prefix("short");
        assert_eq!(prefix, "short");
    }

    #[test]
    fn test_extract_api_key_prefix_empty() {
        let prefix = extract_api_key_prefix("");
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_extract_api_key_prefix_7_chars() {
        let prefix = extract_api_key_prefix("1234567");
        assert_eq!(prefix, "1234567");
    }
}
