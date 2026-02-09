use super::*;
use crate::config::models::enterprise::SsoConfig;
use crate::utils::test_env;

const TEST_ENV_KEYS: [&str; 17] = [
    ENV_HOST,
    ENV_PORT,
    ENV_WORKERS,
    ENV_TIMEOUT,
    ENV_DATABASE_URL,
    ENV_ENABLE_JWT,
    ENV_ENABLE_API_KEY,
    ENV_JWT_SECRET,
    ENV_JWT_EXPIRATION,
    ENV_API_KEY_HEADER,
    ENV_PROVIDERS,
    "LITELLM_PROVIDER_OPENAI_TYPE",
    "LITELLM_PROVIDER_OPENAI_API_KEY",
    "LITELLM_PROVIDER_OPENAI_BASE_URL",
    "LITELLM_PROVIDER_OPENAI_MODELS",
    "LITELLM_PROVIDER_OPENAI_TAGS",
    "LITELLM_PROVIDER_OPENAI_MAX_RETRIES",
];

fn clear_test_env() {
    for key in TEST_ENV_KEYS {
        remove_test_env(key);
    }
}

fn set_test_env(key: &str, value: &str) {
    test_env::set_var(key, value);
}

fn remove_test_env(key: &str) {
    test_env::remove_var(key);
}

fn create_test_provider(name: &str) -> ProviderConfig {
    ProviderConfig {
        name: name.to_string(),
        provider_type: "openai".to_string(),
        api_key: "test-api-key".to_string(),
        ..ProviderConfig::default()
    }
}

fn create_valid_config() -> GatewayConfig {
    let mut config = GatewayConfig {
        providers: vec![create_test_provider("test-provider")],
        ..Default::default()
    };
    config.storage.database.url = "postgres://localhost/test".to_string();
    config.auth.jwt_secret = "test-secret-key".to_string();
    config
}

// ==================== GatewayConfig Default Tests ====================

#[test]
fn test_gateway_config_default() {
    let config = GatewayConfig::default();
    assert!(config.providers.is_empty());
    assert_eq!(config.server.port, 8000);
}

#[test]
fn test_gateway_config_from_env() {
    let _guard = test_env::lock();
    clear_test_env();
    set_test_env(ENV_HOST, "127.0.0.1");
    set_test_env(ENV_PORT, "18080");
    set_test_env(
        ENV_DATABASE_URL,
        "postgresql://env-user:env-pass@localhost/env-db",
    );
    set_test_env(ENV_ENABLE_JWT, "true");
    set_test_env(ENV_JWT_SECRET, "StrongJwtSecretWithMixedCaseAndNumbers1234");
    set_test_env(ENV_PROVIDERS, "openai");
    set_test_env("LITELLM_PROVIDER_OPENAI_TYPE", "openai");
    set_test_env("LITELLM_PROVIDER_OPENAI_API_KEY", "sk-test-key");
    set_test_env(
        "LITELLM_PROVIDER_OPENAI_BASE_URL",
        "https://api.openai.com/v1",
    );
    set_test_env("LITELLM_PROVIDER_OPENAI_MODELS", "gpt-4o,gpt-4.1");
    set_test_env("LITELLM_PROVIDER_OPENAI_TAGS", "prod,primary");
    set_test_env("LITELLM_PROVIDER_OPENAI_MAX_RETRIES", "5");

    let config = GatewayConfig::from_env().unwrap();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 18080);
    assert_eq!(
        config.storage.database.url,
        "postgresql://env-user:env-pass@localhost/env-db"
    );
    assert_eq!(config.providers.len(), 1);
    assert_eq!(config.providers[0].name, "openai");
    assert_eq!(config.providers[0].provider_type, "openai");
    assert_eq!(config.providers[0].api_key, "sk-test-key");
    assert_eq!(
        config.providers[0].base_url,
        Some("https://api.openai.com/v1".to_string())
    );
    assert_eq!(
        config.providers[0].models,
        vec!["gpt-4o".to_string(), "gpt-4.1".to_string()]
    );
    assert_eq!(
        config.providers[0].tags,
        vec!["prod".to_string(), "primary".to_string()]
    );
    assert_eq!(config.providers[0].max_retries, 5);

    clear_test_env();
}

#[test]
fn test_gateway_config_from_env_requires_providers() {
    let _guard = test_env::lock();
    clear_test_env();
    set_test_env(ENV_ENABLE_JWT, "true");
    set_test_env(ENV_JWT_SECRET, "StrongJwtSecretWithMixedCaseAndNumbers1234");

    let result = GatewayConfig::from_env();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains(ENV_PROVIDERS));

    clear_test_env();
}

#[test]
fn test_gateway_config_from_env_invalid_port() {
    let _guard = test_env::lock();
    clear_test_env();
    set_test_env(ENV_PORT, "invalid-port");
    set_test_env(ENV_ENABLE_JWT, "true");
    set_test_env(ENV_JWT_SECRET, "StrongJwtSecretWithMixedCaseAndNumbers1234");
    set_test_env(ENV_PROVIDERS, "openai");
    set_test_env("LITELLM_PROVIDER_OPENAI_TYPE", "openai");
    set_test_env("LITELLM_PROVIDER_OPENAI_API_KEY", "sk-test-key");

    let result = GatewayConfig::from_env();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains(ENV_PORT));

    clear_test_env();
}

// ==================== GatewayConfig Validation Tests ====================

#[test]
fn test_gateway_config_validate_success() {
    let config = create_valid_config();
    assert!(config.validate().is_ok());
}

#[test]
fn test_gateway_config_validate_port_zero() {
    let mut config = create_valid_config();
    config.server.port = 0;
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("port"));
}

#[test]
fn test_gateway_config_validate_no_providers() {
    let mut config = create_valid_config();
    config.providers = vec![];
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("provider"));
}

#[test]
fn test_gateway_config_validate_empty_provider_name() {
    let mut config = create_valid_config();
    config.providers[0].name = "".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name"));
}

#[test]
fn test_gateway_config_validate_duplicate_provider_names() {
    let mut config = create_valid_config();
    config.providers.push(create_test_provider("test-provider"));
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate"));
}

#[test]
fn test_gateway_config_validate_empty_api_key() {
    let mut config = create_valid_config();
    config.providers[0].api_key = "".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("API key"));
}

#[test]
fn test_gateway_config_validate_empty_database_url() {
    let mut config = create_valid_config();
    config.storage.database.url = "".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Database URL"));
}

#[test]
fn test_gateway_config_validate_empty_jwt_secret() {
    let mut config = create_valid_config();
    config.auth.jwt_secret = "".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("JWT secret"));
}

// ==================== GatewayConfig Provider Lookup Tests ====================

#[test]
fn test_gateway_config_get_provider_found() {
    let config = create_valid_config();
    let provider = config.get_provider("test-provider");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name, "test-provider");
}

#[test]
fn test_gateway_config_get_provider_not_found() {
    let config = create_valid_config();
    let provider = config.get_provider("non-existent");
    assert!(provider.is_none());
}

#[test]
fn test_gateway_config_get_providers_by_type() {
    let mut config = create_valid_config();
    let mut anthropic_provider = create_test_provider("anthropic-provider");
    anthropic_provider.provider_type = "anthropic".to_string();
    config.providers.push(anthropic_provider);

    let openai_providers = config.get_providers_by_type("openai");
    assert_eq!(openai_providers.len(), 1);

    let anthropic_providers = config.get_providers_by_type("anthropic");
    assert_eq!(anthropic_providers.len(), 1);
}

#[test]
fn test_gateway_config_get_providers_by_tag() {
    let mut config = create_valid_config();
    config.providers[0].tags = vec!["production".to_string(), "fast".to_string()];

    let mut tagged_provider = create_test_provider("tagged-provider");
    tagged_provider.tags = vec!["staging".to_string()];
    config.providers.push(tagged_provider);

    let prod_providers = config.get_providers_by_tag("production");
    assert_eq!(prod_providers.len(), 1);

    let staging_providers = config.get_providers_by_tag("staging");
    assert_eq!(staging_providers.len(), 1);

    let no_tag_providers = config.get_providers_by_tag("nonexistent");
    assert!(no_tag_providers.is_empty());
}

// ==================== GatewayConfig Feature Check Tests ====================

#[test]
fn test_gateway_config_is_feature_enabled_jwt_auth() {
    let mut config = GatewayConfig::default();
    config.auth.enable_jwt = true;
    assert!(config.is_feature_enabled("jwt_auth"));

    config.auth.enable_jwt = false;
    assert!(!config.is_feature_enabled("jwt_auth"));
}

#[test]
fn test_gateway_config_is_feature_enabled_api_key_auth() {
    let mut config = GatewayConfig::default();
    config.auth.enable_api_key = true;
    assert!(config.is_feature_enabled("api_key_auth"));
}

#[test]
fn test_gateway_config_is_feature_enabled_rbac() {
    let mut config = GatewayConfig::default();
    config.auth.rbac.enabled = true;
    assert!(config.is_feature_enabled("rbac"));
}

#[test]
fn test_gateway_config_is_feature_enabled_metrics() {
    let mut config = GatewayConfig::default();
    config.monitoring.metrics.enabled = true;
    assert!(config.is_feature_enabled("metrics"));
}

#[test]
fn test_gateway_config_is_feature_enabled_tracing() {
    let mut config = GatewayConfig::default();
    config.monitoring.tracing.enabled = true;
    assert!(config.is_feature_enabled("tracing"));
}

#[test]
fn test_gateway_config_is_feature_enabled_health_checks() {
    let config = GatewayConfig::default();
    assert!(config.is_feature_enabled("health_checks"));
}

#[test]
fn test_gateway_config_is_feature_enabled_caching() {
    let mut config = GatewayConfig::default();
    config.cache.enabled = true;
    assert!(config.is_feature_enabled("caching"));
}

#[test]
fn test_gateway_config_is_feature_enabled_semantic_cache() {
    let mut config = GatewayConfig::default();
    config.cache.semantic_cache = true;
    assert!(config.is_feature_enabled("semantic_cache"));
}

#[test]
fn test_gateway_config_is_feature_enabled_rate_limiting() {
    let mut config = GatewayConfig::default();
    config.rate_limit.enabled = true;
    assert!(config.is_feature_enabled("rate_limiting"));
}

#[test]
fn test_gateway_config_is_feature_enabled_enterprise() {
    let mut config = GatewayConfig::default();
    config.enterprise.enabled = true;
    assert!(config.is_feature_enabled("enterprise"));
}

#[test]
fn test_gateway_config_is_feature_enabled_sso() {
    let mut config = GatewayConfig::default();
    config.enterprise.sso = Some(SsoConfig {
        provider: "okta".to_string(),
        client_id: "client".to_string(),
        client_secret: "secret".to_string(),
        redirect_url: "https://redirect.example.com".to_string(),
        settings: std::collections::HashMap::new(),
    });
    assert!(config.is_feature_enabled("sso"));
}

#[test]
fn test_gateway_config_is_feature_enabled_audit_logging() {
    let mut config = GatewayConfig::default();
    config.enterprise.audit_logging = true;
    assert!(config.is_feature_enabled("audit_logging"));
}

#[test]
fn test_gateway_config_is_feature_enabled_advanced_analytics() {
    let mut config = GatewayConfig::default();
    config.enterprise.advanced_analytics = true;
    assert!(config.is_feature_enabled("advanced_analytics"));
}

#[test]
fn test_gateway_config_is_feature_enabled_unknown() {
    let config = GatewayConfig::default();
    assert!(!config.is_feature_enabled("unknown_feature"));
}

// ==================== GatewayConfig Environment Tests ====================

#[test]
fn test_gateway_config_for_environment_development() {
    let config = GatewayConfig::default();
    let dev_config = config.for_environment("development");
    assert!(dev_config.server.dev_mode);
    assert!(dev_config.monitoring.tracing.enabled);
}

#[test]
fn test_gateway_config_for_environment_production() {
    let config = GatewayConfig::default();
    let prod_config = config.for_environment("production");
    assert!(!prod_config.server.dev_mode);
    assert!(prod_config.monitoring.metrics.enabled);
    assert!(prod_config.monitoring.tracing.enabled);
}

#[test]
fn test_gateway_config_for_environment_testing() {
    let config = GatewayConfig::default();
    let test_config = config.for_environment("testing");
    assert!(test_config.server.dev_mode);
    assert!(!test_config.cache.enabled);
    assert!(!test_config.rate_limit.enabled);
}

#[test]
fn test_gateway_config_for_environment_unknown() {
    let config = GatewayConfig::default();
    let unknown_config = config.for_environment("unknown");
    assert_eq!(unknown_config.server.dev_mode, config.server.dev_mode);
}

// ==================== GatewayConfig Merge Tests ====================

#[test]
fn test_gateway_config_merge_providers() {
    let base = create_valid_config();
    let mut other = GatewayConfig {
        providers: vec![create_test_provider("other-provider")],
        ..Default::default()
    };
    other.storage.database.url = "postgres://other".to_string();
    other.auth.jwt_secret = "other-secret".to_string();

    let merged = base.merge(other);
    assert_eq!(merged.providers.len(), 2);
}

#[test]
fn test_gateway_config_merge_provider_override() {
    let base = create_valid_config();
    let mut other = GatewayConfig::default();
    let mut override_provider = create_test_provider("test-provider");
    override_provider.api_key = "new-api-key".to_string();
    other.providers = vec![override_provider];

    let merged = base.merge(other);
    assert_eq!(merged.providers.len(), 1);
    assert_eq!(merged.providers[0].api_key, "new-api-key");
}

// ==================== GatewayConfig Serialization Tests ====================

#[test]
fn test_gateway_config_serialization() {
    let config = create_valid_config();
    let json = serde_json::to_value(&config).unwrap();
    assert!(json["server"].is_object());
    assert!(json["providers"].is_array());
}

#[test]
fn test_gateway_config_clone() {
    let config = create_valid_config();
    let cloned = config.clone();
    assert_eq!(config.providers.len(), cloned.providers.len());
}
