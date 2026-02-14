//! Cost calculation utilities for the Gateway
//!
//! This module provides cost calculation functionality for different AI providers.

use crate::core::models::metrics::cost::{CostInfo, CostRates};
use crate::core::models::metrics::token::TokenUsage;
use crate::utils::error::gateway_error::{GatewayError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost calculator for AI providers
#[derive(Debug, Clone)]
pub struct CostCalculator {
    /// Provider cost configurations
    #[allow(dead_code)]
    provider_costs: HashMap<String, ProviderCostConfig>,
}

/// Provider cost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostConfig {
    /// Provider name
    pub provider: String,
    /// Model cost configurations
    pub models: HashMap<String, ModelCostConfig>,
    /// Default cost configuration
    pub default: Option<ModelCostConfig>,
}

/// Model cost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostConfig {
    /// Model name
    pub model: String,
    /// Input cost per token
    pub input_cost_per_token: f64,
    /// Output cost per token
    pub output_cost_per_token: f64,
    /// Cost per request (if applicable)
    pub cost_per_request: Option<f64>,
    /// Cost per image (for image models)
    pub cost_per_image: Option<f64>,
    /// Cost per audio second (for audio models)
    pub cost_per_audio_second: Option<f64>,
    /// Currency
    pub currency: String,
    /// Billing model
    pub billing_model: BillingModel,
}

/// Billing model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingModel {
    /// Pay per token
    PerToken,
    /// Pay per request
    PerRequest,
    /// Pay per image
    PerImage,
    /// Pay per audio second
    PerAudioSecond,
    /// Flat rate
    FlatRate,
    /// Free
    Free,
}

impl CostCalculator {
    /// Create a new cost calculator
    pub fn new() -> Self {
        Self {
            provider_costs: Self::default_provider_costs(),
        }
    }

    /// Load cost configuration from file
    #[allow(dead_code)]
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GatewayError::Config(format!("Failed to read cost config: {}", e)))?;

        let provider_costs: HashMap<String, ProviderCostConfig> = serde_yaml::from_str(&content)
            .map_err(|e| GatewayError::Config(format!("Failed to parse cost config: {}", e)))?;

        Ok(Self { provider_costs })
    }

    /// Calculate cost for a request
    #[allow(dead_code)]
    pub fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        token_usage: &TokenUsage,
        request_count: u32,
        image_count: Option<u32>,
        audio_seconds: Option<f64>,
    ) -> Result<CostInfo> {
        let provider_config = self
            .provider_costs
            .get(provider)
            .ok_or_else(|| GatewayError::Config(format!("Unknown provider: {}", provider)))?;

        let model_config = provider_config
            .models
            .get(model)
            .or(provider_config.default.as_ref())
            .ok_or_else(|| {
                GatewayError::Config(format!(
                    "Unknown model: {} for provider: {}",
                    model, provider
                ))
            })?;

        let mut input_cost = 0.0;
        let mut output_cost = 0.0;
        let mut total_cost = 0.0;

        match model_config.billing_model {
            BillingModel::PerToken => {
                input_cost = token_usage.input_tokens as f64 * model_config.input_cost_per_token;
                output_cost = token_usage.output_tokens as f64 * model_config.output_cost_per_token;
                total_cost = input_cost + output_cost;
            }
            BillingModel::PerRequest => {
                if let Some(cost_per_request) = model_config.cost_per_request {
                    total_cost = request_count as f64 * cost_per_request;
                }
            }
            BillingModel::PerImage => {
                if let (Some(cost_per_image), Some(images)) =
                    (model_config.cost_per_image, image_count)
                {
                    total_cost = images as f64 * cost_per_image;
                }
            }
            BillingModel::PerAudioSecond => {
                if let (Some(cost_per_second), Some(seconds)) =
                    (model_config.cost_per_audio_second, audio_seconds)
                {
                    total_cost = seconds * cost_per_second;
                }
            }
            BillingModel::FlatRate => {
                // Flat rate billing would be handled at the subscription level
                total_cost = 0.0;
            }
            BillingModel::Free => {
                total_cost = 0.0;
            }
        }

        Ok(CostInfo {
            input_cost,
            output_cost,
            total_cost,
            currency: model_config.currency.clone(),
            rates: CostRates {
                input_cost_per_token: model_config.input_cost_per_token,
                output_cost_per_token: model_config.output_cost_per_token,
                cost_per_request: model_config.cost_per_request,
            },
        })
    }

    /// Get cost rates for a model
    #[allow(dead_code)]
    pub fn get_cost_rates(&self, provider: &str, model: &str) -> Result<CostRates> {
        let provider_config = self
            .provider_costs
            .get(provider)
            .ok_or_else(|| GatewayError::Config(format!("Unknown provider: {}", provider)))?;

        let model_config = provider_config
            .models
            .get(model)
            .or(provider_config.default.as_ref())
            .ok_or_else(|| {
                GatewayError::Config(format!(
                    "Unknown model: {} for provider: {}",
                    model, provider
                ))
            })?;

        Ok(CostRates {
            input_cost_per_token: model_config.input_cost_per_token,
            output_cost_per_token: model_config.output_cost_per_token,
            cost_per_request: model_config.cost_per_request,
        })
    }

    /// Add or update provider cost configuration
    #[allow(dead_code)]
    pub fn add_provider_config(&mut self, config: ProviderCostConfig) {
        self.provider_costs.insert(config.provider.clone(), config);
    }

    /// Get all supported providers
    #[allow(dead_code)]
    pub fn get_providers(&self) -> Vec<String> {
        self.provider_costs.keys().cloned().collect()
    }

    /// Get models for a provider
    #[allow(dead_code)]
    pub fn get_models(&self, provider: &str) -> Vec<String> {
        if let Some(config) = self.provider_costs.get(provider) {
            config.models.keys().cloned().collect()
        } else {
            vec![]
        }
    }

    /// Default provider cost configurations
    fn default_provider_costs() -> HashMap<String, ProviderCostConfig> {
        let mut costs = HashMap::new();

        // OpenAI costs (as of 2024)
        let mut openai_models = HashMap::new();
        openai_models.insert(
            "gpt-4".to_string(),
            ModelCostConfig {
                model: "gpt-4".to_string(),
                input_cost_per_token: 0.00003,
                output_cost_per_token: 0.00006,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );
        openai_models.insert(
            "gpt-4-turbo".to_string(),
            ModelCostConfig {
                model: "gpt-4-turbo".to_string(),
                input_cost_per_token: 0.00001,
                output_cost_per_token: 0.00003,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );
        openai_models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelCostConfig {
                model: "gpt-3.5-turbo".to_string(),
                input_cost_per_token: 0.0000005,
                output_cost_per_token: 0.0000015,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );

        costs.insert(
            "openai".to_string(),
            ProviderCostConfig {
                provider: "openai".to_string(),
                models: openai_models,
                default: Some(ModelCostConfig {
                    model: "default".to_string(),
                    input_cost_per_token: 0.00001,
                    output_cost_per_token: 0.00002,
                    cost_per_request: None,
                    cost_per_image: None,
                    cost_per_audio_second: None,
                    currency: "USD".to_string(),
                    billing_model: BillingModel::PerToken,
                }),
            },
        );

        // Anthropic costs
        let mut anthropic_models = HashMap::new();
        anthropic_models.insert(
            "claude-3-opus".to_string(),
            ModelCostConfig {
                model: "claude-3-opus".to_string(),
                input_cost_per_token: 0.000015,
                output_cost_per_token: 0.000075,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );
        anthropic_models.insert(
            "claude-3-sonnet".to_string(),
            ModelCostConfig {
                model: "claude-3-sonnet".to_string(),
                input_cost_per_token: 0.000003,
                output_cost_per_token: 0.000015,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );

        costs.insert(
            "anthropic".to_string(),
            ProviderCostConfig {
                provider: "anthropic".to_string(),
                models: anthropic_models,
                default: Some(ModelCostConfig {
                    model: "default".to_string(),
                    input_cost_per_token: 0.000003,
                    output_cost_per_token: 0.000015,
                    cost_per_request: None,
                    cost_per_image: None,
                    cost_per_audio_second: None,
                    currency: "USD".to_string(),
                    billing_model: BillingModel::PerToken,
                }),
            },
        );

        costs
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for cost calculations
pub mod utils {
    /// Convert cost between currencies (simplified)
    #[allow(dead_code)]
    pub fn convert_currency(amount: f64, from: &str, to: &str, rate: f64) -> f64 {
        if from == to { amount } else { amount * rate }
    }

    /// Calculate cost savings between two providers
    #[allow(dead_code)]
    pub fn calculate_savings(cost1: f64, cost2: f64) -> (f64, f64) {
        let savings = (cost1 - cost2).abs();
        let percentage = if cost1 > 0.0 {
            (savings / cost1.max(cost2)) * 100.0
        } else {
            0.0
        };
        (savings, percentage)
    }

    /// Estimate monthly cost based on usage patterns
    #[allow(dead_code)]
    pub fn estimate_monthly_cost(
        daily_requests: u32,
        avg_input_tokens: u32,
        avg_output_tokens: u32,
        cost_per_input_token: f64,
        cost_per_output_token: f64,
    ) -> f64 {
        let daily_cost = daily_requests as f64
            * (avg_input_tokens as f64 * cost_per_input_token
                + avg_output_tokens as f64 * cost_per_output_token);
        daily_cost * 30.0 // Approximate month
    }

    /// Calculate cost per request
    #[allow(dead_code)]
    pub fn cost_per_request(total_cost: f64, request_count: u32) -> f64 {
        if request_count > 0 {
            total_cost / request_count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CostCalculator Creation Tests ====================

    #[test]
    fn test_cost_calculator_new() {
        let calculator = CostCalculator::new();
        // Should have default providers
        let providers = calculator.get_providers();
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_cost_calculator_default() {
        let calculator = CostCalculator::default();
        let providers = calculator.get_providers();
        assert!(!providers.is_empty());
    }

    // ==================== Cost Calculation Tests ====================

    #[test]
    fn test_cost_calculation() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        let cost = calculator
            .calculate_cost("openai", "gpt-3.5-turbo", &token_usage, 1, None, None)
            .unwrap();

        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.currency, "USD");
    }

    #[test]
    fn test_cost_calculation_gpt4() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        let cost = calculator
            .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
            .unwrap();

        // GPT-4 input: 0.00003 * 1000 = 0.03
        // GPT-4 output: 0.00006 * 500 = 0.03
        assert!((cost.input_cost - 0.03).abs() < 1e-10);
        assert!((cost.output_cost - 0.03).abs() < 1e-10);
        assert!((cost.total_cost - 0.06).abs() < 1e-10);
    }

    #[test]
    fn test_cost_calculation_gpt4_turbo() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        let cost = calculator
            .calculate_cost("openai", "gpt-4-turbo", &token_usage, 1, None, None)
            .unwrap();

        // GPT-4-turbo input: 0.00001 * 1000 = 0.01
        // GPT-4-turbo output: 0.00003 * 500 = 0.015
        assert!((cost.input_cost - 0.01).abs() < 1e-10);
        assert!((cost.output_cost - 0.015).abs() < 1e-10);
        assert!((cost.total_cost - 0.025).abs() < 1e-10);
    }

    #[test]
    fn test_cost_calculation_anthropic() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        let cost = calculator
            .calculate_cost("anthropic", "claude-3-opus", &token_usage, 1, None, None)
            .unwrap();

        // Claude-3-opus input: 0.000015 * 1000 = 0.015
        // Claude-3-opus output: 0.000075 * 500 = 0.0375
        assert!((cost.input_cost - 0.015).abs() < 1e-10);
        assert!((cost.output_cost - 0.0375).abs() < 1e-10);
        assert!((cost.total_cost - 0.0525).abs() < 1e-10);
    }

    #[test]
    fn test_cost_calculation_zero_tokens() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(0, 0);

        let cost = calculator
            .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
            .unwrap();

        assert_eq!(cost.input_cost, 0.0);
        assert_eq!(cost.output_cost, 0.0);
        assert_eq!(cost.total_cost, 0.0);
    }

    #[test]
    fn test_cost_calculation_large_tokens() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(100000, 50000);

        let cost = calculator
            .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
            .unwrap();

        // GPT-4 input: 0.00003 * 100000 = 3.0
        // GPT-4 output: 0.00006 * 50000 = 3.0
        assert!((cost.input_cost - 3.0).abs() < 1e-10);
        assert!((cost.output_cost - 3.0).abs() < 1e-10);
        assert!((cost.total_cost - 6.0).abs() < 1e-10);
    }

    // ==================== Default Model Fallback Tests ====================

    #[test]
    fn test_cost_calculation_default_model() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        // Use an unknown model that should fall back to default
        let cost = calculator
            .calculate_cost("openai", "unknown-model", &token_usage, 1, None, None)
            .unwrap();

        // Default openai model: input 0.00001, output 0.00002
        assert!((cost.input_cost - 0.01).abs() < 1e-10);
        assert!((cost.output_cost - 0.01).abs() < 1e-10);
        assert!((cost.total_cost - 0.02).abs() < 1e-10);
    }

    // ==================== Error Cases ====================

    #[test]
    fn test_unknown_provider() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(100, 50);

        let result = calculator.calculate_cost(
            "unknown_provider",
            "unknown_model",
            &token_usage,
            1,
            None,
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown provider"));
    }

    #[test]
    fn test_unknown_model_no_default() {
        // Create a provider without default config
        let mut calculator = CostCalculator::new();
        let config = ProviderCostConfig {
            provider: "test_provider".to_string(),
            models: HashMap::new(),
            default: None,
        };
        calculator.add_provider_config(config);

        let token_usage = TokenUsage::new(100, 50);
        let result = calculator.calculate_cost(
            "test_provider",
            "unknown_model",
            &token_usage,
            1,
            None,
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown model"));
    }

    // ==================== Cost Rates Tests ====================

    #[test]
    fn test_cost_rates() {
        let calculator = CostCalculator::new();
        let rates = calculator.get_cost_rates("openai", "gpt-4").unwrap();

        assert!(rates.input_cost_per_token > 0.0);
        assert!(rates.output_cost_per_token > 0.0);
    }

    #[test]
    fn test_cost_rates_unknown_provider() {
        let calculator = CostCalculator::new();
        let result = calculator.get_cost_rates("unknown_provider", "model");

        assert!(result.is_err());
    }

    #[test]
    fn test_cost_rates_unknown_model_no_default() {
        let mut calculator = CostCalculator::new();
        let config = ProviderCostConfig {
            provider: "test_provider".to_string(),
            models: HashMap::new(),
            default: None,
        };
        calculator.add_provider_config(config);

        let result = calculator.get_cost_rates("test_provider", "unknown_model");
        assert!(result.is_err());
    }

    #[test]
    fn test_cost_rates_default_fallback() {
        let calculator = CostCalculator::new();
        let rates = calculator
            .get_cost_rates("openai", "unknown-model")
            .unwrap();

        // Should fall back to default rates
        assert_eq!(rates.input_cost_per_token, 0.00001);
        assert_eq!(rates.output_cost_per_token, 0.00002);
    }

    // ==================== Provider/Model Management Tests ====================

    #[test]
    fn test_get_providers() {
        let calculator = CostCalculator::new();
        let providers = calculator.get_providers();

        assert!(providers.len() >= 2);
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_get_models() {
        let calculator = CostCalculator::new();

        let openai_models = calculator.get_models("openai");
        assert!(openai_models.contains(&"gpt-4".to_string()));
        assert!(openai_models.contains(&"gpt-4-turbo".to_string()));
        assert!(openai_models.contains(&"gpt-3.5-turbo".to_string()));

        let anthropic_models = calculator.get_models("anthropic");
        assert!(anthropic_models.contains(&"claude-3-opus".to_string()));
        assert!(anthropic_models.contains(&"claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_get_models_unknown_provider() {
        let calculator = CostCalculator::new();
        let models = calculator.get_models("unknown_provider");
        assert!(models.is_empty());
    }

    #[test]
    fn test_add_provider_config() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "test-model".to_string(),
            ModelCostConfig {
                model: "test-model".to_string(),
                input_cost_per_token: 0.001,
                output_cost_per_token: 0.002,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "EUR".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );

        let config = ProviderCostConfig {
            provider: "test_provider".to_string(),
            models,
            default: None,
        };

        calculator.add_provider_config(config);

        let providers = calculator.get_providers();
        assert!(providers.contains(&"test_provider".to_string()));

        let test_models = calculator.get_models("test_provider");
        assert!(test_models.contains(&"test-model".to_string()));
    }

    // ==================== BillingModel Tests ====================

    #[test]
    fn test_billing_model_per_request() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "per-request-model".to_string(),
            ModelCostConfig {
                model: "per-request-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
                cost_per_request: Some(0.05),
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerRequest,
            },
        );

        calculator.add_provider_config(ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        });

        let token_usage = TokenUsage::new(1000, 500);
        let cost = calculator
            .calculate_cost("test", "per-request-model", &token_usage, 10, None, None)
            .unwrap();

        // 10 requests * 0.05 = 0.50
        assert!((cost.total_cost - 0.50).abs() < 1e-10);
    }

    #[test]
    fn test_billing_model_per_image() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "image-model".to_string(),
            ModelCostConfig {
                model: "image-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
                cost_per_request: None,
                cost_per_image: Some(0.02),
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerImage,
            },
        );

        calculator.add_provider_config(ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        });

        let token_usage = TokenUsage::new(0, 0);
        let cost = calculator
            .calculate_cost("test", "image-model", &token_usage, 1, Some(5), None)
            .unwrap();

        // 5 images * 0.02 = 0.10
        assert!((cost.total_cost - 0.10).abs() < 1e-10);
    }

    #[test]
    fn test_billing_model_per_audio_second() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "audio-model".to_string(),
            ModelCostConfig {
                model: "audio-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: Some(0.001),
                currency: "USD".to_string(),
                billing_model: BillingModel::PerAudioSecond,
            },
        );

        calculator.add_provider_config(ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        });

        let token_usage = TokenUsage::new(0, 0);
        let cost = calculator
            .calculate_cost("test", "audio-model", &token_usage, 1, None, Some(60.0))
            .unwrap();

        // 60 seconds * 0.001 = 0.06
        assert!((cost.total_cost - 0.06).abs() < 1e-10);
    }

    #[test]
    fn test_billing_model_flat_rate() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "flat-rate-model".to_string(),
            ModelCostConfig {
                model: "flat-rate-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::FlatRate,
            },
        );

        calculator.add_provider_config(ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        });

        let token_usage = TokenUsage::new(1000, 500);
        let cost = calculator
            .calculate_cost("test", "flat-rate-model", &token_usage, 1, None, None)
            .unwrap();

        // Flat rate means 0 cost at request level
        assert_eq!(cost.total_cost, 0.0);
    }

    #[test]
    fn test_billing_model_free() {
        let mut calculator = CostCalculator::new();

        let mut models = HashMap::new();
        models.insert(
            "free-model".to_string(),
            ModelCostConfig {
                model: "free-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::Free,
            },
        );

        calculator.add_provider_config(ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        });

        let token_usage = TokenUsage::new(1000, 500);
        let cost = calculator
            .calculate_cost("test", "free-model", &token_usage, 1, None, None)
            .unwrap();

        assert_eq!(cost.total_cost, 0.0);
    }

    // ==================== Utils Tests ====================

    #[test]
    fn test_utils() {
        let savings = utils::calculate_savings(1.0, 0.8);
        assert!((savings.0 - 0.2).abs() < 1e-10);
        assert!((savings.1 - 20.0).abs() < 1e-10);

        let monthly_cost = utils::estimate_monthly_cost(100, 1000, 500, 0.00001, 0.00002);
        assert!(monthly_cost > 0.0);

        let cost_per_req = utils::cost_per_request(10.0, 100);
        assert_eq!(cost_per_req, 0.1);
    }

    #[test]
    fn test_convert_currency_same() {
        let amount = utils::convert_currency(100.0, "USD", "USD", 1.0);
        assert_eq!(amount, 100.0);
    }

    #[test]
    fn test_convert_currency_different() {
        let amount = utils::convert_currency(100.0, "USD", "EUR", 0.85);
        assert!((amount - 85.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_savings_equal() {
        let (savings, percentage) = utils::calculate_savings(1.0, 1.0);
        assert_eq!(savings, 0.0);
        assert_eq!(percentage, 0.0);
    }

    #[test]
    fn test_calculate_savings_reverse() {
        // Order shouldn't matter for absolute savings
        let (savings1, _) = utils::calculate_savings(1.0, 0.5);
        let (savings2, _) = utils::calculate_savings(0.5, 1.0);
        assert_eq!(savings1, savings2);
    }

    #[test]
    fn test_calculate_savings_zero() {
        let (savings, percentage) = utils::calculate_savings(0.0, 0.0);
        assert_eq!(savings, 0.0);
        assert_eq!(percentage, 0.0);
    }

    #[test]
    fn test_estimate_monthly_cost_zero_requests() {
        let cost = utils::estimate_monthly_cost(0, 1000, 500, 0.00001, 0.00002);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_monthly_cost_calculation() {
        // 100 daily requests, 1000 input tokens, 500 output tokens
        // Daily cost = 100 * (1000 * 0.00001 + 500 * 0.00002) = 100 * (0.01 + 0.01) = 2.0
        // Monthly = 2.0 * 30 = 60.0
        let cost = utils::estimate_monthly_cost(100, 1000, 500, 0.00001, 0.00002);
        assert!((cost - 60.0).abs() < 1e-10);
    }

    #[test]
    fn test_cost_per_request_zero_requests() {
        let cpr = utils::cost_per_request(10.0, 0);
        assert_eq!(cpr, 0.0);
    }

    #[test]
    fn test_cost_per_request_zero_cost() {
        let cpr = utils::cost_per_request(0.0, 100);
        assert_eq!(cpr, 0.0);
    }

    // ==================== ModelCostConfig Tests ====================

    #[test]
    fn test_model_cost_config_serialization() {
        let config = ModelCostConfig {
            model: "test-model".to_string(),
            input_cost_per_token: 0.001,
            output_cost_per_token: 0.002,
            cost_per_request: Some(0.05),
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::PerToken,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ModelCostConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "test-model");
        assert_eq!(deserialized.input_cost_per_token, 0.001);
        assert_eq!(deserialized.cost_per_request, Some(0.05));
    }

    #[test]
    fn test_billing_model_serialization() {
        let models = vec![
            (BillingModel::PerToken, "\"per_token\""),
            (BillingModel::PerRequest, "\"per_request\""),
            (BillingModel::PerImage, "\"per_image\""),
            (BillingModel::PerAudioSecond, "\"per_audio_second\""),
            (BillingModel::FlatRate, "\"flat_rate\""),
            (BillingModel::Free, "\"free\""),
        ];

        for (model, expected) in models {
            let json = serde_json::to_string(&model).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_provider_cost_config_serialization() {
        let mut models = HashMap::new();
        models.insert(
            "model1".to_string(),
            ModelCostConfig {
                model: "model1".to_string(),
                input_cost_per_token: 0.001,
                output_cost_per_token: 0.002,
                cost_per_request: None,
                cost_per_image: None,
                cost_per_audio_second: None,
                currency: "USD".to_string(),
                billing_model: BillingModel::PerToken,
            },
        );

        let config = ProviderCostConfig {
            provider: "test".to_string(),
            models,
            default: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderCostConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.provider, "test");
        assert!(deserialized.models.contains_key("model1"));
    }

    // ==================== CostInfo Tests ====================

    #[test]
    fn test_cost_info_structure() {
        let calculator = CostCalculator::new();
        let token_usage = TokenUsage::new(1000, 500);

        let cost = calculator
            .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
            .unwrap();

        // Verify rates are included in the response
        assert!(cost.rates.input_cost_per_token > 0.0);
        assert!(cost.rates.output_cost_per_token > 0.0);
    }
}
