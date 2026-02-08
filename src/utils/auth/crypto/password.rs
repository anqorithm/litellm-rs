//! Password hashing and verification using Argon2

use crate::utils::error::error::{GatewayError, Result};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| GatewayError::Crypto(format!("Failed to hash password: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| GatewayError::Crypto(format!("Failed to parse password hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(GatewayError::Crypto(format!(
            "Password verification failed: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== hash_password Tests ====================

    #[test]
    fn test_hash_password_produces_hash() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();

        assert!(!hash.is_empty());
        // Argon2 hashes start with $argon2
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_unique_each_time() {
        let password = "same-password";

        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_empty() {
        let hash = hash_password("").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_long() {
        let password = "x".repeat(1000);
        let hash = hash_password(&password).unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_unicode() {
        let password = "密码🔐пароль";
        let hash = hash_password(password).unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    // ==================== verify_password Tests ====================

    #[test]
    fn test_verify_password_correct() {
        let password = "correct-password";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "original-password";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password("wrong-password", &hash).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hash = hash_password("").unwrap();
        let is_valid = verify_password("", &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_unicode() {
        let password = "密码🔐пароль";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = "CaseSensitive";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password("casesensitive", &hash).unwrap();
        assert!(!is_valid);
    }
}
