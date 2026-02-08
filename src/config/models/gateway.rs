//! Main gateway configuration

#![allow(missing_docs)]

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: Vec<ProviderConfig>,
    /// Router configuration
    pub router: RouterConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Caching configuration
    #[serde(default)]
    pub cache: CacheConfig,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Enterprise features configuration
    #[serde(default)]
    pub enterprise: EnterpriseConfig,
}

impl GatewayConfig {
    pub fn from_env() -> crate::utils::error::error::Result<Self> {
        Ok(Self {
            server: ServerConfig::default(),
            providers: vec![],
            router: RouterConfig::default(),
            storage: StorageConfig::default(),
            auth: AuthConfig::default(),
            monitoring: MonitoringConfig::default(),
            cache: CacheConfig::default(),
            rate_limit: RateLimitConfig::default(),
            enterprise: EnterpriseConfig::default(),
        })
    }
}

impl GatewayConfig {
    /// Merge two configurations, with other taking precedence
    pub fn merge(mut self, other: Self) -> Self {
        self.server = self.server.merge(other.server);

        // Merge providers (other takes precedence for same names)
        let mut provider_map: HashMap<String, ProviderConfig> = self
            .providers
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        for provider in other.providers {
            provider_map.insert(provider.name.clone(), provider);
        }

        self.providers = provider_map.into_values().collect();
        self.router = self.router.merge(other.router);
        self.storage = self.storage.merge(other.storage);
        self.auth = self.auth.merge(other.auth);
        self.monitoring = self.monitoring.merge(other.monitoring);
        self.cache = self.cache.merge(other.cache);
        self.rate_limit = self.rate_limit.merge(other.rate_limit);
        self.enterprise = self.enterprise.merge(other.enterprise);

        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server config
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // Validate providers
        if self.providers.is_empty() {
            return Err("At least one provider must be configured".to_string());
        }

        let mut provider_names = std::collections::HashSet::new();
        for provider in &self.providers {
            if provider.name.is_empty() {
                return Err("Provider name cannot be empty".to_string());
            }
            if !provider_names.insert(&provider.name) {
                return Err(format!("Duplicate provider name: {}", provider.name));
            }
            if provider.api_key.is_empty() {
                return Err(format!(
                    "API key is required for provider: {}",
                    provider.name
                ));
            }
        }

        // Validate storage config
        if self.storage.database.url.is_empty() {
            return Err("Database URL is required".to_string());
        }

        // Validate auth config
        if self.auth.jwt_secret.is_empty() {
            return Err("JWT secret is required".to_string());
        }

        Ok(())
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Get providers by type
    pub fn get_providers_by_type(&self, provider_type: &str) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|p| p.provider_type == provider_type)
            .collect()
    }

    /// Get providers by tag
    pub fn get_providers_by_tag(&self, tag: &str) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "jwt_auth" => self.auth.enable_jwt,
            "api_key_auth" => self.auth.enable_api_key,
            "rbac" => self.auth.rbac.enabled,
            "metrics" => self.monitoring.metrics.enabled,
            "tracing" => self.monitoring.tracing.enabled,
            "health_checks" => true, // Always enabled
            "caching" => self.cache.enabled,
            "semantic_cache" => self.cache.semantic_cache,
            "rate_limiting" => self.rate_limit.enabled,
            "enterprise" => self.enterprise.enabled,
            "sso" => self.enterprise.sso.is_some(),
            "audit_logging" => self.enterprise.audit_logging,
            "advanced_analytics" => self.enterprise.advanced_analytics,
            _ => false,
        }
    }

    /// Get environment-specific configuration
    pub fn for_environment(&self, env: &str) -> Self {
        let mut config = self.clone();

        match env {
            "development" => {
                config.server.dev_mode = true;
                config.monitoring.tracing.enabled = true;
            }
            "production" => {
                config.server.dev_mode = false;
                config.monitoring.metrics.enabled = true;
                config.monitoring.tracing.enabled = true;
            }
            "testing" => {
                config.server.dev_mode = true;
                config.cache.enabled = false;
                config.rate_limit.enabled = false;
            }
            _ => {}
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let config = GatewayConfig::from_env().unwrap();
        assert!(config.providers.is_empty());
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
}
