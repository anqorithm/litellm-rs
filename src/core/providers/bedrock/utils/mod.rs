//! Utility modules for Bedrock provider
//!
//! Contains shared utilities for AWS authentication, region management,
//! cost calculation, and other common functionality.

pub mod auth;
pub mod cost;
pub mod region;

// Re-export main types and functions
pub use auth::{AwsAuth, AwsCredentials};
pub use cost::{CostCalculator, ModelPricing};
pub use region::{AWS_REGIONS, is_model_available_in_region, validate_region};

/// Normalize Bedrock model IDs coming from external callers.
/// - Strips optional "bedrock/" prefix
/// - Strips optional region prefix like "us." or "us-east-1."
pub fn normalize_bedrock_model_id(model_id: &str) -> String {
    let trimmed = model_id.trim();
    let without_prefix = trimmed.strip_prefix("bedrock/").unwrap_or(trimmed);
    let without_region = strip_region_prefix(without_prefix);
    without_region.to_string()
}

fn strip_region_prefix(model_id: &str) -> &str {
    let (prefix, rest) = match model_id.split_once('.') {
        Some(parts) => parts,
        None => return model_id,
    };

    if is_region_prefix(prefix) {
        rest
    } else {
        model_id
    }
}

fn is_region_prefix(prefix: &str) -> bool {
    matches!(prefix, "us" | "eu" | "ap" | "sa" | "ca" | "me" | "af")
        || (prefix.len() >= 4
            && prefix.contains('-')
            && prefix
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
}

#[cfg(test)]
mod tests {
    use super::normalize_bedrock_model_id;

    #[test]
    fn test_normalize_bedrock_model_id() {
        assert_eq!(
            normalize_bedrock_model_id("bedrock/us.anthropic.claude-3-5-sonnet-20241022-v2:0"),
            "anthropic.claude-3-5-sonnet-20241022-v2:0"
        );
        assert_eq!(
            normalize_bedrock_model_id("bedrock/anthropic.claude-3-opus-20240229"),
            "anthropic.claude-3-opus-20240229"
        );
        assert_eq!(
            normalize_bedrock_model_id("us-east-1.anthropic.claude-3-haiku-20240307"),
            "anthropic.claude-3-haiku-20240307"
        );
        assert_eq!(
            normalize_bedrock_model_id("anthropic.claude-3-opus-20240229"),
            "anthropic.claude-3-opus-20240229"
        );
    }
}
