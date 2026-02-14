//! Data validation utilities

use crate::utils::error::gateway_error::{GatewayError, Result};
use regex::Regex;
use std::collections::HashSet;

/// Data validation utilities
pub struct DataValidator;

impl DataValidator {
    /// Validate username
    pub fn validate_username(username: &str) -> Result<()> {
        if username.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() < 3 {
            return Err(GatewayError::Validation(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if username.len() > 50 {
            return Err(GatewayError::Validation(
                "Username cannot exceed 50 characters".to_string(),
            ));
        }

        let username_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|e| GatewayError::Internal(format!("Regex error: {}", e)))?;

        if !username_regex.is_match(username) {
            return Err(GatewayError::Validation(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(GatewayError::Validation(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if password.len() > 128 {
            return Err(GatewayError::Validation(
                "Password cannot exceed 128 characters".to_string(),
            ));
        }

        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        let strength_count = [has_lowercase, has_uppercase, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if strength_count < 3 {
            return Err(GatewayError::Validation(
                "Password must contain at least 3 of: lowercase, uppercase, digit, special character".to_string()
            ));
        }

        Ok(())
    }

    /// Validate team name
    pub fn validate_team_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Team name cannot be empty".to_string(),
            ));
        }

        if name.len() < 2 {
            return Err(GatewayError::Validation(
                "Team name must be at least 2 characters".to_string(),
            ));
        }

        if name.len() > 100 {
            return Err(GatewayError::Validation(
                "Team name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate tags
    pub fn validate_tags(tags: &[String]) -> Result<()> {
        if tags.len() > 20 {
            return Err(GatewayError::Validation(
                "Cannot have more than 20 tags".to_string(),
            ));
        }

        let mut unique_tags = HashSet::new();
        for tag in tags {
            if tag.trim().is_empty() {
                return Err(GatewayError::Validation("Tag cannot be empty".to_string()));
            }

            if tag.len() > 50 {
                return Err(GatewayError::Validation(
                    "Tag cannot exceed 50 characters".to_string(),
                ));
            }

            if !unique_tags.insert(tag.to_lowercase()) {
                return Err(GatewayError::Validation(format!("Duplicate tag: {}", tag)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Username Validation Tests ====================

    #[test]
    fn test_validate_username_valid() {
        assert!(DataValidator::validate_username("john_doe").is_ok());
        assert!(DataValidator::validate_username("user123").is_ok());
        assert!(DataValidator::validate_username("test-user").is_ok());
        assert!(DataValidator::validate_username("ABC").is_ok());
    }

    #[test]
    fn test_validate_username_empty() {
        let result = DataValidator::validate_username("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_username_whitespace_only() {
        let result = DataValidator::validate_username("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_username_too_short() {
        let result = DataValidator::validate_username("ab");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 3"));
    }

    #[test]
    fn test_validate_username_exactly_3_chars() {
        assert!(DataValidator::validate_username("abc").is_ok());
    }

    #[test]
    fn test_validate_username_too_long() {
        let long_name = "a".repeat(51);
        let result = DataValidator::validate_username(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("50"));
    }

    #[test]
    fn test_validate_username_exactly_50_chars() {
        let name = "a".repeat(50);
        assert!(DataValidator::validate_username(&name).is_ok());
    }

    #[test]
    fn test_validate_username_invalid_chars() {
        let result = DataValidator::validate_username("user@name");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("letters, numbers, underscores")
        );
    }

    #[test]
    fn test_validate_username_space_in_middle() {
        let result = DataValidator::validate_username("user name");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_username_special_chars() {
        assert!(DataValidator::validate_username("user!name").is_err());
        assert!(DataValidator::validate_username("user.name").is_err());
        assert!(DataValidator::validate_username("user#name").is_err());
    }

    #[test]
    fn test_validate_username_allowed_chars() {
        assert!(DataValidator::validate_username("User_Name-123").is_ok());
    }

    // ==================== Password Validation Tests ====================

    #[test]
    fn test_validate_password_valid() {
        assert!(DataValidator::validate_password("Password1!").is_ok());
        assert!(DataValidator::validate_password("Secure@Pass123").is_ok());
        assert!(DataValidator::validate_password("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = DataValidator::validate_password("Pass1!");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 8"));
    }

    #[test]
    fn test_validate_password_exactly_8_chars() {
        assert!(DataValidator::validate_password("Pass1!ab").is_ok());
    }

    #[test]
    fn test_validate_password_too_long() {
        let long_pass = "a".repeat(129);
        let result = DataValidator::validate_password(&long_pass);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("128"));
    }

    #[test]
    fn test_validate_password_exactly_128_chars() {
        // Need to meet strength requirements too
        let pass = format!("Aa1!{}", "a".repeat(124));
        assert!(DataValidator::validate_password(&pass).is_ok());
    }

    #[test]
    fn test_validate_password_weak_lowercase_only() {
        let result = DataValidator::validate_password("abcdefgh");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 3"));
    }

    #[test]
    fn test_validate_password_weak_no_special() {
        // Lowercase + uppercase only = 2 strength
        let result = DataValidator::validate_password("AbCdEfGh");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_three_strength() {
        // Lowercase + uppercase + digit = 3 strength (valid)
        assert!(DataValidator::validate_password("Password1").is_ok());
    }

    #[test]
    fn test_validate_password_four_strength() {
        // All four character types
        assert!(DataValidator::validate_password("Password1!").is_ok());
    }

    #[test]
    fn test_validate_password_digit_special_lowercase() {
        assert!(DataValidator::validate_password("password1!").is_ok());
    }

    #[test]
    fn test_validate_password_various_special_chars() {
        assert!(DataValidator::validate_password("Password@1").is_ok());
        assert!(DataValidator::validate_password("Password#1").is_ok());
        assert!(DataValidator::validate_password("Password$1").is_ok());
        assert!(DataValidator::validate_password("Password!1").is_ok());
    }

    // ==================== Team Name Validation Tests ====================

    #[test]
    fn test_validate_team_name_valid() {
        assert!(DataValidator::validate_team_name("My Team").is_ok());
        assert!(DataValidator::validate_team_name("Engineering").is_ok());
        assert!(DataValidator::validate_team_name("AI Research Team").is_ok());
    }

    #[test]
    fn test_validate_team_name_empty() {
        let result = DataValidator::validate_team_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_team_name_whitespace_only() {
        let result = DataValidator::validate_team_name("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_team_name_too_short() {
        let result = DataValidator::validate_team_name("A");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 2"));
    }

    #[test]
    fn test_validate_team_name_exactly_2_chars() {
        assert!(DataValidator::validate_team_name("AB").is_ok());
    }

    #[test]
    fn test_validate_team_name_too_long() {
        let long_name = "a".repeat(101);
        let result = DataValidator::validate_team_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100"));
    }

    #[test]
    fn test_validate_team_name_exactly_100_chars() {
        let name = "a".repeat(100);
        assert!(DataValidator::validate_team_name(&name).is_ok());
    }

    #[test]
    fn test_validate_team_name_special_chars_allowed() {
        // Team names can have special characters
        assert!(DataValidator::validate_team_name("Team #1").is_ok());
        assert!(DataValidator::validate_team_name("R&D Team").is_ok());
        assert!(DataValidator::validate_team_name("Frontend/Backend").is_ok());
    }

    // ==================== Tags Validation Tests ====================

    #[test]
    fn test_validate_tags_empty_list() {
        let tags: Vec<String> = vec![];
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_valid() {
        let tags = vec!["api".to_string(), "backend".to_string(), "v2".to_string()];
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_too_many() {
        let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect();
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("20"));
    }

    #[test]
    fn test_validate_tags_exactly_20() {
        let tags: Vec<String> = (0..20).map(|i| format!("tag{}", i)).collect();
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_empty_tag() {
        let tags = vec!["valid".to_string(), "".to_string()];
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_tags_whitespace_only_tag() {
        let tags = vec!["valid".to_string(), "   ".to_string()];
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_tags_too_long_tag() {
        let long_tag = "a".repeat(51);
        let tags = vec![long_tag];
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("50"));
    }

    #[test]
    fn test_validate_tags_exactly_50_char_tag() {
        let tag = "a".repeat(50);
        let tags = vec![tag];
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_duplicate() {
        let tags = vec!["api".to_string(), "API".to_string()];
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_validate_tags_case_insensitive_duplicate() {
        let tags = vec!["Backend".to_string(), "BACKEND".to_string()];
        let result = DataValidator::validate_tags(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_validate_tags_similar_but_different() {
        let tags = vec!["api".to_string(), "api2".to_string(), "api-v2".to_string()];
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_single_tag() {
        let tags = vec!["single".to_string()];
        assert!(DataValidator::validate_tags(&tags).is_ok());
    }
}
