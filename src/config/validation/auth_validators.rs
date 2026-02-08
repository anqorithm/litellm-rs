//! Authentication configuration validators
//!
//! This module provides validation implementations for authentication-related
//! configuration structures including AuthConfig and RbacConfig.

use super::trait_def::Validate;
use crate::config::models::auth::{AuthConfig, RbacConfig};
use tracing::debug;

impl Validate for AuthConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating auth configuration");

        if self.jwt_secret.is_empty() {
            return Err("JWT secret cannot be empty".to_string());
        }

        if self.jwt_secret == "change-me-in-production" && !cfg!(test) {
            return Err("JWT secret must be changed from default value in production".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT secret should be at least 32 characters long".to_string());
        }

        if self.jwt_expiration == 0 {
            return Err("JWT expiration must be greater than 0".to_string());
        }

        if self.jwt_expiration > 86400 * 30 {
            // 30 days
            return Err("JWT expiration should not exceed 30 days".to_string());
        }

        if self.api_key_header.is_empty() {
            return Err("API key header cannot be empty".to_string());
        }

        self.rbac.validate()?;

        Ok(())
    }
}

impl Validate for RbacConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.default_role.is_empty() {
            return Err("Default role cannot be empty when RBAC is enabled".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::trait_def::Validate;
    use super::*; // Import the trait explicitly

    fn create_valid_auth_config() -> AuthConfig {
        AuthConfig {
            enable_jwt: true,
            enable_api_key: true,
            jwt_secret: "a".repeat(32), // 32 character secret
            jwt_expiration: 3600,       // 1 hour
            api_key_header: "X-API-Key".to_string(),
            rbac: RbacConfig::default(),
        }
    }

    // Helper to call the Validate trait method explicitly
    fn validate_config<T: Validate>(config: &T) -> Result<(), String> {
        Validate::validate(config)
    }

    // ==================== AuthConfig Validation Tests ====================

    #[test]
    fn test_auth_config_valid() {
        let config = create_valid_auth_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_auth_config_empty_jwt_secret() {
        let mut config = create_valid_auth_config();
        config.jwt_secret = "".to_string();

        let result = validate_config(&config);
        assert!(
            result.is_err(),
            "Expected validation to fail for empty JWT secret"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("JWT secret cannot be empty"),
            "Got error: {}",
            err
        );
    }

    #[test]
    fn test_auth_config_short_jwt_secret() {
        let mut config = create_valid_auth_config();
        config.jwt_secret = "short".to_string(); // Less than 32 chars

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 32 characters"));
    }

    #[test]
    fn test_auth_config_jwt_secret_exactly_32_chars() {
        let mut config = create_valid_auth_config();
        config.jwt_secret = "a".repeat(32);

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_auth_config_jwt_secret_long() {
        let mut config = create_valid_auth_config();
        config.jwt_secret = "a".repeat(100);

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_auth_config_zero_jwt_expiration() {
        let mut config = create_valid_auth_config();
        config.jwt_expiration = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than 0"));
    }

    #[test]
    fn test_auth_config_max_jwt_expiration() {
        let mut config = create_valid_auth_config();
        config.jwt_expiration = 86400 * 30; // Exactly 30 days

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_auth_config_jwt_expiration_too_long() {
        let mut config = create_valid_auth_config();
        config.jwt_expiration = 86400 * 30 + 1; // More than 30 days

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("30 days"));
    }

    #[test]
    fn test_auth_config_empty_api_key_header() {
        let mut config = create_valid_auth_config();
        config.api_key_header = "".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("API key header cannot be empty")
        );
    }

    #[test]
    fn test_auth_config_custom_api_key_header() {
        let mut config = create_valid_auth_config();
        config.api_key_header = "Authorization".to_string();

        assert!(validate_config(&config).is_ok());
    }

    // ==================== RbacConfig Validation Tests ====================

    #[test]
    fn test_rbac_config_disabled_empty_role() {
        let config = RbacConfig {
            enabled: false,
            default_role: "".to_string(),
            ..Default::default()
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_rbac_config_enabled_with_role() {
        let config = RbacConfig {
            enabled: true,
            default_role: "user".to_string(),
            ..Default::default()
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_rbac_config_enabled_empty_role() {
        let config = RbacConfig {
            enabled: true,
            default_role: "".to_string(),
            ..Default::default()
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Default role cannot be empty"));
    }

    // ==================== AuthConfig with RbacConfig Tests ====================

    #[test]
    fn test_auth_config_with_valid_rbac() {
        let mut config = create_valid_auth_config();
        config.rbac = RbacConfig {
            enabled: true,
            default_role: "admin".to_string(),
            ..Default::default()
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_auth_config_with_invalid_rbac() {
        let mut config = create_valid_auth_config();
        config.rbac = RbacConfig {
            enabled: true,
            default_role: "".to_string(),
            ..Default::default()
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Default role"));
    }
}
