//! AWS Region Management for Bedrock
//!
//! Handles AWS region validation, model availability checks,
//! and region-specific configuration.

use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::sync::LazyLock;

/// All AWS regions that support Bedrock
pub const AWS_REGIONS: &[&str] = &[
    // US regions
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    // EU regions
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-central-1",
    "eu-central-2",
    "eu-north-1",
    "eu-south-1",
    "eu-south-2",
    // Asia Pacific regions
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-northeast-3",
    "ap-south-1",
    "ap-south-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-southeast-3",
    "ap-southeast-4",
    "ap-southeast-5",
    // Other regions
    "ca-central-1",
    "sa-east-1",
    "us-gov-west-1",
    "us-gov-east-1",
];

/// Model family to available regions mapping
static MODEL_REGION_MAPPING: LazyLock<HashMap<&'static str, &'static [&'static str]>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // Claude models - widely available
        map.insert("anthropic.claude", AWS_REGIONS);

        // Titan models - US regions primarily
        map.insert(
            "amazon.titan",
            &["us-east-1", "us-east-2", "us-west-1", "us-west-2"],
        );

        // Nova models - limited availability
        map.insert("amazon.nova", &["us-east-1", "us-west-2"]);

        // AI21 models
        map.insert(
            "ai21",
            &["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-2"],
        );

        // Cohere models
        map.insert(
            "cohere",
            &["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-2"],
        );

        // Mistral models
        map.insert("mistral", &["us-east-1", "us-west-2", "eu-west-1"]);

        // Meta Llama models
        map.insert(
            "meta.llama",
            &["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-2"],
        );

        // Stability AI models
        map.insert(
            "stability",
            &["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"],
        );

        map
    });

/// Validate if a region is supported by Bedrock
pub fn validate_region(region: &str) -> Result<(), ProviderError> {
    if AWS_REGIONS.contains(&region) {
        Ok(())
    } else {
        Err(ProviderError::configuration(
            "bedrock",
            format!(
                "Invalid AWS region: {}. Supported regions: {:?}",
                region, AWS_REGIONS
            ),
        ))
    }
}

/// Check if a model is available in a specific region
pub fn is_model_available_in_region(model_id: &str, region: &str) -> bool {
    // Extract model prefix (e.g., "anthropic.claude" from "anthropic.claude-3-opus")
    let model_prefix = extract_model_prefix(model_id);

    if let Some(regions) = MODEL_REGION_MAPPING.get(model_prefix) {
        regions.contains(&region)
    } else {
        // If no specific mapping, assume available in all regions
        AWS_REGIONS.contains(&region)
    }
}

/// Extract model family prefix from full model ID
fn extract_model_prefix(model_id: &str) -> &str {
    // Handle different model ID patterns
    if model_id.starts_with("anthropic.claude") {
        "anthropic.claude"
    } else if model_id.starts_with("amazon.titan") {
        "amazon.titan"
    } else if model_id.starts_with("amazon.nova") {
        "amazon.nova"
    } else if model_id.starts_with("ai21") {
        "ai21"
    } else if model_id.starts_with("cohere") {
        "cohere"
    } else if model_id.starts_with("mistral") {
        "mistral"
    } else if model_id.starts_with("meta.llama") {
        "meta.llama"
    } else if model_id.starts_with("stability") {
        "stability"
    } else {
        // Fallback to first part before '-'
        model_id.split('-').next().unwrap_or(model_id)
    }
}

/// Get regions where a specific model family is available
#[cfg(test)]
pub fn get_model_regions(model_family: &str) -> Option<&'static [&'static str]> {
    MODEL_REGION_MAPPING.get(model_family).copied()
}

/// Get US regions specifically
#[cfg(test)]
pub fn get_us_regions() -> &'static [&'static str] {
    &["us-east-1", "us-east-2", "us-west-1", "us-west-2"]
}

/// Get EU regions specifically
#[cfg(test)]
pub fn get_eu_regions() -> &'static [&'static str] {
    &[
        "eu-west-1",
        "eu-west-2",
        "eu-west-3",
        "eu-central-1",
        "eu-central-2",
        "eu-north-1",
        "eu-south-1",
        "eu-south-2",
    ]
}

/// Get Asia Pacific regions specifically
#[cfg(test)]
pub fn get_ap_regions() -> &'static [&'static str] {
    &[
        "ap-northeast-1",
        "ap-northeast-2",
        "ap-northeast-3",
        "ap-south-1",
        "ap-south-2",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-southeast-3",
        "ap-southeast-4",
        "ap-southeast-5",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_validation() {
        assert!(validate_region("us-east-1").is_ok());
        assert!(validate_region("eu-west-1").is_ok());
        assert!(validate_region("invalid-region").is_err());
    }

    #[test]
    fn test_model_availability() {
        // Claude should be available in most regions
        assert!(is_model_available_in_region(
            "anthropic.claude-3-opus",
            "us-east-1"
        ));
        assert!(is_model_available_in_region(
            "anthropic.claude-3-opus",
            "eu-west-1"
        ));

        // Nova has limited availability
        assert!(is_model_available_in_region("amazon.nova-pro", "us-east-1"));
        assert!(!is_model_available_in_region(
            "amazon.nova-pro",
            "ap-south-1"
        ));
    }

    #[test]
    fn test_model_prefix_extraction() {
        assert_eq!(
            extract_model_prefix("anthropic.claude-3-opus-20240229"),
            "anthropic.claude"
        );
        assert_eq!(
            extract_model_prefix("amazon.titan-text-express-v1"),
            "amazon.titan"
        );
        assert_eq!(
            extract_model_prefix("meta.llama3-70b-instruct-v1:0"),
            "meta.llama"
        );
    }

    #[test]
    fn test_regional_getters() {
        assert!(get_us_regions().contains(&"us-east-1"));
        assert!(get_eu_regions().contains(&"eu-west-1"));
        assert!(get_ap_regions().contains(&"ap-southeast-1"));
    }
}
