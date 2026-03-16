//! Pricing configuration models
//!
//! This module provides unified pricing management for all providers

use crate::config::models::defaults::default_true;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPricing {
    /// Model name
    pub model: String,
    /// Input token cost per 1K tokens
    pub input_cost_per_1k: f64,
    /// Output token cost per 1K tokens  
    pub output_cost_per_1k: f64,
    /// Currency (e.g., "USD", "CNY")
    pub currency: String,
    /// Last updated timestamp
    #[serde(default = "chrono::Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// Optional notes about pricing
    pub notes: Option<String>,
}

/// Provider pricing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPricing {
    /// Provider name (openai, anthropic, glm, etc.)
    pub provider: String,
    /// Default pricing for unknown models
    pub default_pricing: ModelPricing,
    /// Per-model pricing
    pub models: HashMap<String, ModelPricing>,
    /// Whether to use external pricing API
    #[serde(default)]
    pub use_external_api: bool,
    /// External pricing API endpoint
    pub external_api_url: Option<String>,
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

/// Global pricing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    /// Default currency
    #[serde(default = "default_currency")]
    pub default_currency: String,
    /// Pricing source priority
    #[serde(default)]
    pub source_priority: Vec<PricingSource>,
    /// Provider-specific pricing configurations
    pub providers: HashMap<String, ProviderPricing>,
    /// Global fallback pricing
    pub fallback_pricing: ModelPricing,
    /// Enable pricing cache
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    /// Update pricing automatically
    #[serde(default)]
    pub auto_update: bool,
    /// Update interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
}

/// Pricing data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingSource {
    /// Use config file pricing
    Config,
    /// Use external API
    ExternalApi,
    /// Use provider's official API
    ProviderApi,
    /// Use cached data
    Cache,
}

/// Pricing update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingUpdateEvent {
    pub provider: String,
    pub model: String,
    pub old_pricing: ModelPricing,
    pub new_pricing: ModelPricing,
    pub timestamp: DateTime<Utc>,
    pub source: PricingSource,
}

impl Default for PricingConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Default
        providers.insert("openai".to_string(), ProviderPricing {
            provider: "openai".to_string(),
            default_pricing: ModelPricing {
                model: "unknown".to_string(),
                input_cost_per_1k: 0.01,
                output_cost_per_1k: 0.03,
                currency: "USD".to_string(),
                updated_at: Utc::now(),
                notes: Some("OpenAI default pricing".to_string()),
            },
            models: HashMap::new(),
            use_external_api: false,
            external_api_url: None,
            cache_ttl: default_cache_ttl(),
        });

        // Default
        providers.insert("glm".to_string(), ProviderPricing {
            provider: "glm".to_string(),
            default_pricing: ModelPricing {
                model: "unknown".to_string(),
                input_cost_per_1k: 0.0001,
                output_cost_per_1k: 0.0003,
                currency: "USD".to_string(),
                updated_at: Utc::now(),
                notes: Some("GLM default pricing (converted from RMB)".to_string()),
            },
            models: HashMap::new(),
            use_external_api: false,
            external_api_url: None,
            cache_ttl: default_cache_ttl(),
        });

        Self {
            default_currency: default_currency(),
            source_priority: vec![
                PricingSource::Cache,
                PricingSource::Config,
                PricingSource::ExternalApi,
                PricingSource::ProviderApi,
            ],
            providers,
            fallback_pricing: ModelPricing {
                model: "fallback".to_string(),
                input_cost_per_1k: 0.01,
                output_cost_per_1k: 0.01,
                currency: "USD".to_string(),
                updated_at: Utc::now(),
                notes: Some("Global fallback pricing".to_string()),
            },
            enable_cache: true,
            auto_update: false,
            update_interval: default_update_interval(),
        }
    }
}

impl ModelPricing {
    /// Calculate cost for given tokens
    pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }

    /// Check if pricing is stale
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.updated_at);
        age.num_seconds() > max_age_seconds as i64
    }

    /// Create a new pricing entry
    pub fn new(model: &str, input_cost: f64, output_cost: f64, currency: &str) -> Self {
        Self {
            model: model.to_string(),
            input_cost_per_1k: input_cost,
            output_cost_per_1k: output_cost,
            currency: currency.to_string(),
            updated_at: Utc::now(),
            notes: None,
        }
    }
}

impl ProviderPricing {
    /// Get pricing for a specific model
    pub fn get_model_pricing(&self, model: &str) -> &ModelPricing {
        self.models.get(model).unwrap_or(&self.default_pricing)
    }

    /// Add or update model pricing
    pub fn set_model_pricing(&mut self, pricing: ModelPricing) {
        self.models.insert(pricing.model.clone(), pricing);
    }

    /// Remove model pricing
    pub fn remove_model_pricing(&mut self, model: &str) -> Option<ModelPricing> {
        self.models.remove(model)
    }

    /// Get all models for this provider
    pub fn get_models(&self) -> Vec<&str> {
        self.models.keys().map(|k| k.as_str()).collect()
    }
}

// Default value functions
fn default_currency() -> String {
    "USD".to_string()
}

fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_update_interval() -> u64 {
    86400 // 24 hours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing_calculate_cost() {
        let pricing = ModelPricing::new("test-model", 0.001, 0.002, "USD");
        
        // 1000 input tokens * 0.001 + 500 output tokens * 0.002 = 0.001 + 0.001 = 0.002
        let cost = pricing.calculate_cost(1000, 500);
        assert!((cost - 0.002).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pricing_is_stale() {
        let mut pricing = ModelPricing::new("test", 0.001, 0.002, "USD");
        
        // Fresh pricing should not be stale
        assert!(!pricing.is_stale(3600));
        
        // Set old timestamp
        pricing.updated_at = Utc::now() - chrono::Duration::hours(2);
        assert!(pricing.is_stale(3600)); // 1 hour max age
    }

    #[test]
    fn test_provider_pricing_get_model() {
        let mut provider = ProviderPricing {
            provider: "test".to_string(),
            default_pricing: ModelPricing::new("default", 0.01, 0.02, "USD"),
            models: HashMap::new(),
            use_external_api: false,
            external_api_url: None,
            cache_ttl: 3600,
        };

        // Should return default for unknown model
        let pricing = provider.get_model_pricing("unknown-model");
        assert_eq!(pricing.input_cost_per_1k, 0.01);

        // Add specific model pricing
        provider.set_model_pricing(ModelPricing::new("specific", 0.005, 0.01, "USD"));
        let pricing = provider.get_model_pricing("specific");
        assert_eq!(pricing.input_cost_per_1k, 0.005);
    }

    #[test]
    fn test_pricing_config_default() {
        let config = PricingConfig::default();
        
        assert_eq!(config.default_currency, "USD");
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("glm"));
        assert!(config.enable_cache);
    }
}