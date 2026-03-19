//! Tests for cryptographic utilities

use super::*;
use base64::{Engine as _, engine::general_purpose};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_password_hashing() {
    let password = "test_password_123";
    let hash = password::hash_password(password).unwrap();

    assert!(password::verify_password(password, &hash).unwrap());
    assert!(!password::verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_api_key_generation() {
    let api_key = keys::generate_api_key();
    assert!(api_key.starts_with("gw-"));
    assert_eq!(api_key.len(), 35); // "gw-" + 32 characters
}

#[test]
fn test_jwt_secret_generation() {
    let secret = keys::generate_jwt_secret();
    assert_eq!(secret.len(), 64);
    assert!(secret.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_api_key_hashing() {
    let api_key = "gw-test123456789";
    let hash = keys::hash_api_key(api_key, None);
    assert_eq!(hash.len(), 64); // SHA256 hex string
}

#[test]
fn test_api_key_prefix() {
    let api_key = "gw-test123456789";
    let prefix = keys::extract_api_key_prefix(api_key);
    assert_eq!(prefix, "gw-t...6789");
}

#[test]
fn test_hmac_signature() {
    let secret = "test_secret";
    let data = "test_data";

    let signature = hmac::create_hmac_signature(secret, data).unwrap();
    assert!(hmac::verify_hmac_signature(secret, data, &signature).unwrap());
    assert!(!hmac::verify_hmac_signature(secret, "wrong_data", &signature).unwrap());
}

#[test]
fn test_hmac_sha256_specific_case() {
    // Test the specific case mentioned in the question
    let key = "key";
    let message = "message";

    let signature = hmac::create_hmac_signature(key, message).unwrap();
    println!(
        "HMAC-SHA256 for key='{}', message='{}': {}",
        key, message, signature
    );

    // Verify the signature is correctly calculated
    assert!(hmac::verify_hmac_signature(key, message, &signature).unwrap());

    // The correct HMAC-SHA256 for key="key" and message="message"
    let expected = "6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a";
    assert_eq!(signature, expected, "HMAC-SHA256 calculation mismatch");

    // Also test against the incorrect value that was mentioned
    let incorrect = "6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011e917a9c6e0c3d5e4c3b";
    assert_ne!(signature, incorrect, "Should not match the incorrect hash");
}

#[test]
fn test_hmac_sha256_rfc4231_vectors() {
    // Test Case 2 from RFC 4231
    let key = "Jefe";
    let data = "what do ya want for nothing?";
    let expected = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";

    let signature = hmac::create_hmac_signature(key, data).unwrap();
    assert_eq!(signature, expected, "RFC 4231 Test Case 2 failed");
}

#[test]
fn test_constant_time_eq() {
    assert!(hmac::constant_time_eq("hello", "hello"));
    assert!(!hmac::constant_time_eq("hello", "world"));
    assert!(!hmac::constant_time_eq("hello", "hello2"));
}

#[test]
fn test_backup_code_generation() {
    let code = backup::generate_backup_code();
    assert_eq!(code.len(), 9); // 4 digits + "-" + 4 digits
    assert!(code.contains('-'));

    let codes = backup::generate_backup_codes(5);
    assert_eq!(codes.len(), 5);
    assert!(codes.iter().all(|c| c.len() == 9));
}

#[test]
fn test_webhook_signature() {
    let secret = "webhook_secret";
    let payload = r#"{"test": "data"}"#;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let signature = webhooks::generate_webhook_signature(secret, payload, timestamp).unwrap();
    assert!(webhooks::verify_webhook_signature(secret, payload, timestamp, &signature).unwrap());

    // Test with wrong payload
    assert!(!webhooks::verify_webhook_signature(secret, "wrong", timestamp, &signature).unwrap());

    // Test with old timestamp (should fail)
    let old_timestamp = timestamp - 400; // More than 5 minutes old
    let old_signature =
        webhooks::generate_webhook_signature(secret, payload, old_timestamp).unwrap();
    assert!(
        !webhooks::verify_webhook_signature(secret, payload, old_timestamp, &old_signature)
            .unwrap()
    );
}

#[test]
fn test_aes_gcm_encryption_decryption() {
    let key = b"my_secret_encryption_key_123456";
    let plaintext = "Hello, World! This is sensitive data.";

    // Encrypt
    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();

    // Encrypted output should be base64 and different from plaintext
    assert_ne!(encrypted, plaintext);
    assert!(encrypted.len() > plaintext.len()); // Includes nonce + tag

    // Decrypt
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_different_nonces() {
    let key = b"test_key_for_nonce_uniqueness!!";
    let plaintext = "Same message encrypted twice";

    // Encrypt same plaintext twice
    let encrypted1 = encryption::encrypt_data(key, plaintext).unwrap();
    let encrypted2 = encryption::encrypt_data(key, plaintext).unwrap();

    // Each encryption should produce different ciphertext (due to random nonce)
    assert_ne!(encrypted1, encrypted2);

    // Both should decrypt to the same plaintext
    assert_eq!(
        encryption::decrypt_data(key, &encrypted1).unwrap(),
        plaintext
    );
    assert_eq!(
        encryption::decrypt_data(key, &encrypted2).unwrap(),
        plaintext
    );
}

#[test]
fn test_aes_gcm_wrong_key() {
    let key1 = b"correct_key_for_encryption_1234";
    let key2 = b"wrong_key_for_decryption_5678!!";
    let plaintext = "Secret message";

    let encrypted = encryption::encrypt_data(key1, plaintext).unwrap();

    // Decryption with wrong key should fail
    let result = encryption::decrypt_data(key2, &encrypted);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_tampered_data() {
    let key = b"key_for_tamper_test_1234567890!";
    let plaintext = "Important data";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();

    // Tamper with the encrypted data
    let mut tampered_bytes = general_purpose::STANDARD.decode(&encrypted).unwrap();
    if let Some(byte) = tampered_bytes.last_mut() {
        *byte ^= 0xFF; // Flip bits in the last byte
    }
    let tampered = general_purpose::STANDARD.encode(&tampered_bytes);

    // Decryption should fail due to authentication tag mismatch
    let result = encryption::decrypt_data(key, &tampered);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_short_data_rejected() {
    let key = b"test_key_for_short_data_check!!";

    // Data too short (less than nonce + auth tag)
    let short_data = general_purpose::STANDARD.encode([0u8; 10]);
    let result = encryption::decrypt_data(key, &short_data);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_empty_plaintext() {
    let key = b"key_for_empty_plaintext_test!!!";
    let plaintext = "";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_unicode_plaintext() {
    let key = b"key_for_unicode_test_1234567890";
    let plaintext = "Hello 世界! Привет мир! 🔐🔑";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_large_plaintext() {
    let key = b"key_for_large_data_test_1234567";
    let plaintext = "A".repeat(10000); // 10KB of data

    let encrypted = encryption::encrypt_data(key, &plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

// ==================== Salt and Hash Tests ====================

#[test]
fn test_generate_salt() {
    let salt1 = encryption::generate_salt();
    let salt2 = encryption::generate_salt();

    // Salts should be base64 encoded
    assert!(general_purpose::STANDARD.decode(&salt1).is_ok());
    assert!(general_purpose::STANDARD.decode(&salt2).is_ok());

    // Each salt should be unique
    assert_ne!(salt1, salt2);

    // Decoded salt should be 16 bytes
    let decoded = general_purpose::STANDARD.decode(&salt1).unwrap();
    assert_eq!(decoded.len(), 16);
}

#[test]
fn test_generate_salt_multiple() {
    let salts: Vec<String> = (0..100).map(|_| encryption::generate_salt()).collect();

    // All salts should be unique
    let unique_salts: std::collections::HashSet<_> = salts.iter().collect();
    assert_eq!(unique_salts.len(), salts.len());
}

#[test]
fn test_hash_with_salt() {
    let data = "my_password";
    let salt = encryption::generate_salt();

    let hash = encryption::hash_with_salt(data, &salt);

    // Hash should be 64 character hex string (SHA-256)
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Same data + salt should produce same hash
    let hash2 = encryption::hash_with_salt(data, &salt);
    assert_eq!(hash, hash2);

    // Different salt should produce different hash
    let salt2 = encryption::generate_salt();
    let hash3 = encryption::hash_with_salt(data, &salt2);
    assert_ne!(hash, hash3);
}

#[test]
fn test_hash_with_salt_different_data() {
    let salt = "fixed_salt";
    let hash1 = encryption::hash_with_salt("data1", salt);
    let hash2 = encryption::hash_with_salt("data2", salt);

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hash_with_salt_empty_data() {
    let salt = "some_salt";
    let hash = encryption::hash_with_salt("", salt);

    // Should still produce a valid hash
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_with_salt_empty_salt() {
    let data = "some_data";
    let hash = encryption::hash_with_salt(data, "");

    // Should still produce a valid hash
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_with_salt_unicode() {
    let data = "密码🔐";
    let salt = "盐值🧂";
    let hash = encryption::hash_with_salt(data, salt);

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// ==================== TOTP Secret Tests ====================

#[test]
fn test_generate_totp_secret() {
    let secret = encryption::generate_totp_secret();

    // Should be base64 encoded
    assert!(general_purpose::STANDARD.decode(&secret).is_ok());

    // Decoded should be 20 bytes (160 bits as per RFC 6238)
    let decoded = general_purpose::STANDARD.decode(&secret).unwrap();
    assert_eq!(decoded.len(), 20);
}

#[test]
fn test_generate_totp_secret_uniqueness() {
    let secrets: Vec<String> = (0..100)
        .map(|_| encryption::generate_totp_secret())
        .collect();

    // All secrets should be unique
    let unique_secrets: std::collections::HashSet<_> = secrets.iter().collect();
    assert_eq!(unique_secrets.len(), secrets.len());
}

// ==================== Token Generation Tests ====================

#[test]
fn test_generate_token() {
    let token = keys::generate_token(32);
    assert_eq!(token.len(), 32);
    assert!(token.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_generate_token_various_lengths() {
    for length in [8, 16, 32, 64, 128] {
        let token = keys::generate_token(length);
        assert_eq!(token.len(), length);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }
}

#[test]
fn test_generate_token_zero_length() {
    let token = keys::generate_token(0);
    assert!(token.is_empty());
}

#[test]
fn test_generate_token_uniqueness() {
    let tokens: Vec<String> = (0..100).map(|_| keys::generate_token(32)).collect();

    // All tokens should be unique
    let unique_tokens: std::collections::HashSet<_> = tokens.iter().collect();
    assert_eq!(unique_tokens.len(), tokens.len());
}

// ==================== Session Token Tests ====================

#[test]
fn test_generate_session_token() {
    let token = keys::generate_session_token();

    // Should be URL-safe base64 encoded (no padding)
    assert!(general_purpose::URL_SAFE_NO_PAD.decode(&token).is_ok());

    // Decoded should be 32 bytes
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(&token).unwrap();
    assert_eq!(decoded.len(), 32);
}

#[test]
fn test_generate_session_token_uniqueness() {
    let tokens: Vec<String> = (0..100).map(|_| keys::generate_session_token()).collect();

    // All tokens should be unique
    let unique_tokens: std::collections::HashSet<_> = tokens.iter().collect();
    assert_eq!(unique_tokens.len(), tokens.len());
}

#[test]
fn test_generate_session_token_url_safe() {
    let token = keys::generate_session_token();

    // URL-safe base64 uses only alphanumeric, -, and _
    assert!(
        token
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    );
}

// ==================== API Key Prefix Tests ====================

#[test]
fn test_api_key_prefix_short_key() {
    // Keys shorter than 8 characters should return as-is
    assert_eq!(keys::extract_api_key_prefix("abc"), "abc");
    assert_eq!(keys::extract_api_key_prefix("1234567"), "1234567");
    assert_eq!(keys::extract_api_key_prefix(""), "");
    assert_eq!(keys::extract_api_key_prefix("a"), "a");
}

#[test]
fn test_api_key_prefix_exact_8_chars() {
    // Exactly 8 characters should still format with ellipsis
    let prefix = keys::extract_api_key_prefix("12345678");
    assert_eq!(prefix, "1234...5678");
}

#[test]
fn test_api_key_prefix_long_key() {
    let prefix = keys::extract_api_key_prefix("gw-abcdefghijklmnopqrstuvwxyz123456");
    assert_eq!(prefix, "gw-a...3456");
}

// ==================== API Key Hash Tests ====================

#[test]
fn test_api_key_hash_consistency() {
    let api_key = "gw-test123456789";
    let hash1 = keys::hash_api_key(api_key, None);
    let hash2 = keys::hash_api_key(api_key, None);

    // Same key should always produce same hash
    assert_eq!(hash1, hash2);
}

#[test]
fn test_api_key_hash_different_keys() {
    let hash1 = keys::hash_api_key("gw-key1", None);
    let hash2 = keys::hash_api_key("gw-key2", None);

    // Different keys should produce different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_api_key_hash_empty() {
    let hash = keys::hash_api_key("", None);

    // Should still produce a valid hash
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// ==================== Backup Code Hash Tests ====================

#[test]
fn test_hash_backup_code() {
    let code = "1234-5678";
    let hash = backup::hash_backup_code(code);

    // Hash should be 64 character hex string (SHA-256)
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_backup_code_consistency() {
    let code = "9876-5432";
    let hash1 = backup::hash_backup_code(code);
    let hash2 = backup::hash_backup_code(code);

    // Same code should always produce same hash
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_backup_code_different_codes() {
    let hash1 = backup::hash_backup_code("1111-1111");
    let hash2 = backup::hash_backup_code("2222-2222");

    // Different codes should produce different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_backup_code_format() {
    for _ in 0..100 {
        let code = backup::generate_backup_code();

        // Should be in format XXXX-XXXX
        assert_eq!(code.len(), 9);
        assert!(code.chars().nth(4) == Some('-'));

        // All other characters should be digits
        let parts: Vec<&str> = code.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));
        assert!(parts[1].chars().all(|c| c.is_ascii_digit()));
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 4);
    }
}

#[test]
fn test_backup_codes_uniqueness() {
    let codes = backup::generate_backup_codes(100);

    // All codes should be unique
    let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
    // Allow some small chance of collision in 100 codes from 10^8 space
    assert!(unique_codes.len() >= 98);
}

#[test]
fn test_backup_codes_empty() {
    let codes = backup::generate_backup_codes(0);
    assert!(codes.is_empty());
}

#[test]
fn test_backup_codes_single() {
    let codes = backup::generate_backup_codes(1);
    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].len(), 9);
}

// ==================== API Key Generation Tests ====================

#[test]
fn test_api_key_format() {
    for _ in 0..100 {
        let api_key = keys::generate_api_key();

        // Should start with "gw-"
        assert!(api_key.starts_with("gw-"));

        // Should be 35 characters total ("gw-" + 32 random chars)
        assert_eq!(api_key.len(), 35);

        // Random part should be alphanumeric
        let random_part = &api_key[3..];
        assert!(random_part.chars().all(|c| c.is_alphanumeric()));
    }
}

#[test]
fn test_api_key_uniqueness() {
    let keys: Vec<String> = (0..100).map(|_| keys::generate_api_key()).collect();

    // All keys should be unique
    let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
    assert_eq!(unique_keys.len(), keys.len());
}

// ==================== JWT Secret Tests ====================

#[test]
fn test_jwt_secret_format() {
    for _ in 0..100 {
        let secret = keys::generate_jwt_secret();

        // Should be 64 characters
        assert_eq!(secret.len(), 64);

        // Should be alphanumeric
        assert!(secret.chars().all(|c| c.is_alphanumeric()));
    }
}

#[test]
fn test_jwt_secret_uniqueness() {
    let secrets: Vec<String> = (0..100).map(|_| keys::generate_jwt_secret()).collect();

    // All secrets should be unique
    let unique_secrets: std::collections::HashSet<_> = secrets.iter().collect();
    assert_eq!(unique_secrets.len(), secrets.len());
}

// ==================== HMAC Additional Tests ====================

#[test]
fn test_hmac_empty_message() {
    let secret = "secret";
    let message = "";

    let signature = hmac::create_hmac_signature(secret, message).unwrap();
    assert!(hmac::verify_hmac_signature(secret, message, &signature).unwrap());
}

#[test]
fn test_hmac_empty_secret() {
    let secret = "";
    let message = "message";

    let signature = hmac::create_hmac_signature(secret, message).unwrap();
    assert!(hmac::verify_hmac_signature(secret, message, &signature).unwrap());
}

#[test]
fn test_hmac_unicode() {
    let secret = "密钥🔐";
    let message = "消息📝";

    let signature = hmac::create_hmac_signature(secret, message).unwrap();
    assert!(hmac::verify_hmac_signature(secret, message, &signature).unwrap());
}

#[test]
fn test_constant_time_eq_empty_strings() {
    assert!(hmac::constant_time_eq("", ""));
}

#[test]
fn test_constant_time_eq_different_lengths() {
    assert!(!hmac::constant_time_eq("short", "much_longer_string"));
    assert!(!hmac::constant_time_eq("", "not_empty"));
}

// ==================== Password Hashing Additional Tests ====================

#[test]
fn test_password_hash_uniqueness() {
    let password = "same_password";
    let hash1 = password::hash_password(password).unwrap();
    let hash2 = password::hash_password(password).unwrap();

    // Each hash should be unique due to salt
    assert_ne!(hash1, hash2);

    // But both should verify correctly
    assert!(password::verify_password(password, &hash1).unwrap());
    assert!(password::verify_password(password, &hash2).unwrap());
}

#[test]
fn test_password_hash_empty() {
    let password = "";
    let hash = password::hash_password(password).unwrap();

    assert!(password::verify_password(password, &hash).unwrap());
    assert!(!password::verify_password("not_empty", &hash).unwrap());
}

#[test]
fn test_password_hash_unicode() {
    let password = "密码🔒安全";
    let hash = password::hash_password(password).unwrap();

    assert!(password::verify_password(password, &hash).unwrap());
    assert!(!password::verify_password("wrong", &hash).unwrap());
}

#[test]
fn test_password_hash_long() {
    let password = "a".repeat(1000);
    let hash = password::hash_password(&password).unwrap();

    assert!(password::verify_password(&password, &hash).unwrap());
}

// ==================== Webhook Additional Tests ====================

#[test]
fn test_webhook_signature_format() {
    let secret = "webhook_secret";
    let payload = "test payload";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let signature = webhooks::generate_webhook_signature(secret, payload, timestamp).unwrap();

    // Signature should be a valid hex string (64 chars for SHA-256)
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_webhook_signature_timestamp_boundary() {
    let secret = "secret";
    let payload = "payload";
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Exactly 5 minutes (300 seconds) old should still be valid
    let timestamp_at_boundary = current_time - 299;
    let signature =
        webhooks::generate_webhook_signature(secret, payload, timestamp_at_boundary).unwrap();
    assert!(
        webhooks::verify_webhook_signature(secret, payload, timestamp_at_boundary, &signature)
            .unwrap()
    );

    // Just over 5 minutes old should be invalid
    let timestamp_expired = current_time - 301;
    let signature_expired =
        webhooks::generate_webhook_signature(secret, payload, timestamp_expired).unwrap();
    assert!(
        !webhooks::verify_webhook_signature(secret, payload, timestamp_expired, &signature_expired)
            .unwrap()
    );
}
