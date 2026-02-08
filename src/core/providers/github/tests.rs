//! Integration tests for GitHub Models provider

use super::*;
use crate::utils::test_env;

#[tokio::test]
async fn test_github_provider_new_without_key() {
    let config = GitHubConfig::default();
    // Clear env var for this test
    test_env::remove_var("GITHUB_TOKEN");
    let provider = GitHubProvider::new(config).await;
    assert!(provider.is_err());
}

#[tokio::test]
async fn test_github_config_from_env() {
    let _guard = test_env::lock();
    test_env::set_var("GITHUB_TOKEN", "ghp_test_env_key");
    let config = GitHubConfig::default();
    assert_eq!(config.get_api_key(), Some("ghp_test_env_key".to_string()));
    test_env::remove_var("GITHUB_TOKEN");
}

#[test]
fn test_github_model_info_completeness() {
    // Ensure all models have required fields populated
    for model_id in get_available_models() {
        let info = get_model_info(model_id).unwrap();
        assert!(!info.model_id.is_empty());
        assert!(!info.display_name.is_empty());
        assert!(info.max_context_length > 0);
        assert!(info.max_output_length > 0);
        // Costs can be zero for free models
        assert!(info.input_cost_per_million >= 0.0);
        assert!(info.output_cost_per_million >= 0.0);
    }
}

#[test]
fn test_github_config_serialization_roundtrip() {
    let config = GitHubConfig {
        api_key: Some("ghp_test123".to_string()),
        api_base: Some("https://custom.api.com".to_string()),
        timeout: 45,
        max_retries: 5,
        debug: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GitHubConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.api_base, deserialized.api_base);
    assert_eq!(config.timeout, deserialized.timeout);
    assert_eq!(config.max_retries, deserialized.max_retries);
    assert_eq!(config.debug, deserialized.debug);
}
