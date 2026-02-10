//! JWT utility functions

use super::types::{Claims, JwtHandler};
use std::time::{SystemTime, UNIX_EPOCH};

impl JwtHandler {
    /// Extract token from Authorization header
    pub fn extract_token_from_header(header_value: &str) -> Option<String> {
        header_value
            .strip_prefix("Bearer ")
            .map(|token| token.to_string())
    }

    /// Get token expiration time
    pub fn get_expiration(&self) -> u64 {
        self.expiration
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX); // If system time is invalid, treat as expired

        claims.exp < now
    }

    /// Get time until token expires
    pub fn time_until_expiry(&self, claims: &Claims) -> Option<u64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        if claims.exp > now {
            Some(claims.exp - now)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::TokenType;
    use super::*;
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
    use uuid::Uuid;

    fn create_test_handler() -> JwtHandler {
        JwtHandler {
            encoding_key: EncodingKey::from_secret(b"test_secret_key"),
            decoding_key: DecodingKey::from_secret(b"test_secret_key"),
            algorithm: Algorithm::HS256,
            expiration: 3600,
            issuer: "test_issuer".to_string(),
        }
    }

    fn create_test_claims(exp_offset_secs: i64) -> Claims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Claims {
            sub: Uuid::new_v4(),
            iat: now,
            exp: (now as i64 + exp_offset_secs) as u64,
            iss: "test_issuer".to_string(),
            aud: "api".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "user".to_string(),
            permissions: vec!["read".to_string()],
            team_id: None,
            session_id: None,
            token_type: TokenType::Access,
        }
    }

    // ==================== extract_token_from_header Tests ====================

    #[test]
    fn test_extract_token_valid_bearer() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_some());
        assert_eq!(
            token.unwrap(),
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"
        );
    }

    #[test]
    fn test_extract_token_empty_bearer() {
        let header = "Bearer ";
        let token = JwtHandler::extract_token_from_header(header);
        assert_eq!(token, Some("".to_string()));
    }

    #[test]
    fn test_extract_token_no_bearer_prefix() {
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_none());
    }

    #[test]
    fn test_extract_token_lowercase_bearer() {
        let header = "bearer token123";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_none()); // Case-sensitive
    }

    #[test]
    fn test_extract_token_basic_auth() {
        let header = "Basic dXNlcm5hbWU6cGFzc3dvcmQ=";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_none());
    }

    #[test]
    fn test_extract_token_empty_string() {
        let header = "";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_none());
    }

    #[test]
    fn test_extract_token_only_bearer() {
        let header = "Bearer";
        let token = JwtHandler::extract_token_from_header(header);
        assert!(token.is_none()); // No space after Bearer
    }

    #[test]
    fn test_extract_token_with_spaces_in_token() {
        let header = "Bearer token with spaces";
        let token = JwtHandler::extract_token_from_header(header);
        assert_eq!(token, Some("token with spaces".to_string()));
    }

    // ==================== get_expiration Tests ====================

    #[test]
    fn test_get_expiration_default() {
        let handler = create_test_handler();
        assert_eq!(handler.get_expiration(), 3600);
    }

    #[test]
    fn test_get_expiration_custom() {
        let handler = JwtHandler {
            encoding_key: EncodingKey::from_secret(b"secret"),
            decoding_key: DecodingKey::from_secret(b"secret"),
            algorithm: Algorithm::HS256,
            expiration: 7200,
            issuer: "test".to_string(),
        };
        assert_eq!(handler.get_expiration(), 7200);
    }

    #[test]
    fn test_get_expiration_short() {
        let handler = JwtHandler {
            encoding_key: EncodingKey::from_secret(b"secret"),
            decoding_key: DecodingKey::from_secret(b"secret"),
            algorithm: Algorithm::HS256,
            expiration: 300, // 5 minutes
            issuer: "test".to_string(),
        };
        assert_eq!(handler.get_expiration(), 300);
    }

    #[test]
    fn test_get_expiration_long() {
        let handler = JwtHandler {
            encoding_key: EncodingKey::from_secret(b"secret"),
            decoding_key: DecodingKey::from_secret(b"secret"),
            algorithm: Algorithm::HS256,
            expiration: 604800, // 1 week
            issuer: "test".to_string(),
        };
        assert_eq!(handler.get_expiration(), 604800);
    }

    // ==================== is_token_expired Tests ====================

    #[test]
    fn test_is_token_expired_not_expired() {
        let handler = create_test_handler();
        let claims = create_test_claims(3600); // Expires in 1 hour
        assert!(!handler.is_token_expired(&claims));
    }

    #[test]
    fn test_is_token_expired_already_expired() {
        let handler = create_test_handler();
        let claims = create_test_claims(-3600); // Expired 1 hour ago
        assert!(handler.is_token_expired(&claims));
    }

    #[test]
    fn test_is_token_expired_just_now() {
        let handler = create_test_handler();
        let claims = create_test_claims(-1); // Just expired
        assert!(handler.is_token_expired(&claims));
    }

    #[test]
    fn test_is_token_expired_far_future() {
        let handler = create_test_handler();
        let claims = create_test_claims(86400 * 365); // 1 year from now
        assert!(!handler.is_token_expired(&claims));
    }

    // ==================== time_until_expiry Tests ====================

    #[test]
    fn test_time_until_expiry_valid() {
        let handler = create_test_handler();
        let claims = create_test_claims(3600); // Expires in 1 hour
        let time_left = handler.time_until_expiry(&claims);
        assert!(time_left.is_some());
        // Should be close to 3600 (allow 1 second tolerance)
        assert!(time_left.unwrap() >= 3599 && time_left.unwrap() <= 3600);
    }

    #[test]
    fn test_time_until_expiry_expired() {
        let handler = create_test_handler();
        let claims = create_test_claims(-3600); // Expired 1 hour ago
        let time_left = handler.time_until_expiry(&claims);
        assert!(time_left.is_none());
    }

    #[test]
    fn test_time_until_expiry_just_expired() {
        let handler = create_test_handler();
        let claims = create_test_claims(-1); // Just expired
        let time_left = handler.time_until_expiry(&claims);
        assert!(time_left.is_none());
    }

    #[test]
    fn test_time_until_expiry_short_time() {
        let handler = create_test_handler();
        let claims = create_test_claims(60); // Expires in 1 minute
        let time_left = handler.time_until_expiry(&claims);
        assert!(time_left.is_some());
        assert!(time_left.unwrap() >= 59 && time_left.unwrap() <= 60);
    }

    #[test]
    fn test_time_until_expiry_long_time() {
        let handler = create_test_handler();
        let claims = create_test_claims(604800); // Expires in 1 week
        let time_left = handler.time_until_expiry(&claims);
        assert!(time_left.is_some());
        // Allow 1 second tolerance
        assert!(time_left.unwrap() >= 604799);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_expiry_consistency() {
        let handler = create_test_handler();
        let claims = create_test_claims(3600);

        // If token is not expired, time_until_expiry should return Some
        if !handler.is_token_expired(&claims) {
            assert!(handler.time_until_expiry(&claims).is_some());
        }
    }

    #[test]
    fn test_expired_token_no_time_left() {
        let handler = create_test_handler();
        let claims = create_test_claims(-100);

        // If token is expired, time_until_expiry should return None
        assert!(handler.is_token_expired(&claims));
        assert!(handler.time_until_expiry(&claims).is_none());
    }
}
