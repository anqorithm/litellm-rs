//! Cost calculation utilities for the Gateway
//!
//! This module provides cost calculation functionality for different AI providers.

use crate::core::models::metrics::cost::{CostInfo, CostRates};
use crate::core::models::metrics::token::TokenUsage;
use crate::utils::error::error::{GatewayError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost calculator for AI providers
#[derive(Debug, Clone)]
pub struct CostCalculator {
    /// Provider cost configurations
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
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GatewayError::Config(format!("Failed to read cost config: {}", e)))?;

        let provider_costs: HashMap<String, ProviderCostConfig> = serde_yaml::from_str(&content)
            .map_err(|e| GatewayError::Config(format!("Failed to parse cost config: {}", e)))?;

        Ok(Self { provider_costs })
    }

    /// Calculate cost for a request
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
    pub fn add_provider_config(&mut self, config: ProviderCostConfig) {
        self.provider_costs.insert(config.provider.clone(), config);
    }

    /// Get all supported providers
    pub fn get_providers(&self) -> Vec<String> {
        self.provider_costs.keys().cloned().collect()
    }

    /// Get models for a provider
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
    pub fn convert_currency(amount: f64, from: &str, to: &str, rate: f64) -> f64 {
        if from == to { amount } else { amount * rate }
    }

    /// Calculate cost savings between two providers
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
    pub fn cost_per_request(total_cost: f64, request_count: u32) -> f64 {
        if request_count > 0 {
            total_cost / request_count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
#[path = "cost_tests.rs"]
mod tests;
