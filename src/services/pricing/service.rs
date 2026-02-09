//! Main pricing service implementation

use super::types::{
    CostRange, CostResult, CostType, ModelInfo, PricingData, PricingEventType, PricingStatistics,
    PricingUpdateEvent,
};
use crate::utils::error::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Pricing service using LiteLLM data format
#[derive(Debug, Clone)]
pub struct PricingService {
    /// Consolidated pricing data - single lock for model data and timestamp
    pub(super) pricing_data: Arc<RwLock<PricingData>>,
    /// HTTP client for fetching updates
    pub(super) http_client: reqwest::Client,
    /// Pricing data source URL
    pub(super) pricing_url: String,
    /// Cache TTL
    pub(super) cache_ttl: Duration,
    /// Event broadcaster for updates
    pub(super) event_sender: broadcast::Sender<PricingUpdateEvent>,
}

impl PricingService {
    /// Create a new pricing service
    pub fn new(pricing_url: Option<String>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        let service = Self {
            pricing_data: Arc::new(RwLock::new(PricingData {
                models: HashMap::new(),
                last_updated: SystemTime::UNIX_EPOCH,
            })),
            http_client: reqwest::Client::new(),
            pricing_url: pricing_url.unwrap_or_else(|| "data/model_prices.json".to_string()),
            cache_ttl: Duration::from_secs(3600), // 1 hour
            event_sender,
        };

        info!("Pricing service initialized with LiteLLM data source");
        service
    }

    /// Get model information
    pub fn get_model_info(&self, model: &str) -> Option<ModelInfo> {
        let data = self.pricing_data.read();
        data.models.get(model).cloned()
    }

    /// Calculate completion cost
    pub async fn calculate_completion_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        prompt: Option<&str>,
        completion: Option<&str>,
        total_time_seconds: Option<f64>,
    ) -> Result<CostResult> {
        // Auto-refresh if needed
        if self.needs_refresh() {
            if let Err(e) = self.refresh_pricing_data().await {
                warn!("Failed to refresh pricing data: {}", e);
            }
        }

        let model_info = self
            .get_model_info(model)
            .ok_or_else(|| GatewayError::not_found(format!("Model not found: {}", model)))?;

        // Provider-specific cost calculation
        match model_info.litellm_provider.as_str() {
            "openai" | "azure" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            "anthropic" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            "google" | "vertex_ai" => self.calculate_google_cost(
                model,
                &model_info,
                input_tokens,
                output_tokens,
                prompt,
                completion,
            ),
            "replicate" | "together_ai" | "baseten" => self.calculate_time_based_cost(
                model,
                &model_info,
                total_time_seconds.unwrap_or(0.0),
            ),
            "zhipuai" | "glm" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            _ => {
                // Default to token-based calculation
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
        }
    }

    /// Calculate token-based cost
    pub(super) fn calculate_token_based_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<CostResult> {
        let input_cost_per_token = model_info.input_cost_per_token.unwrap_or(0.0);
        let output_cost_per_token = model_info.output_cost_per_token.unwrap_or(0.0);

        let input_cost = (input_tokens as f64) * input_cost_per_token;
        let output_cost = (output_tokens as f64) * output_cost_per_token;
        let total_cost = input_cost + output_cost;

        Ok(CostResult {
            input_cost,
            output_cost,
            total_cost,
            input_tokens,
            output_tokens,
            model: model.to_string(),
            provider: model_info.litellm_provider.clone(),
            cost_type: CostType::TokenBased,
        })
    }

    /// Calculate Google/Vertex AI cost (character or token based)
    fn calculate_google_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        input_tokens: u32,
        output_tokens: u32,
        prompt: Option<&str>,
        completion: Option<&str>,
    ) -> Result<CostResult> {
        // Check if character-based pricing is available
        if model_info.input_cost_per_character.is_some()
            || model_info.output_cost_per_character.is_some()
        {
            let input_cost_per_char = model_info.input_cost_per_character.unwrap_or(0.0);
            let output_cost_per_char = model_info.output_cost_per_character.unwrap_or(0.0);

            let input_chars = prompt.map(|p| p.len()).unwrap_or(0) as f64;
            let output_chars = completion.map(|c| c.len()).unwrap_or(0) as f64;

            let input_cost = input_chars * input_cost_per_char;
            let output_cost = output_chars * output_cost_per_char;

            Ok(CostResult {
                input_cost,
                output_cost,
                total_cost: input_cost + output_cost,
                input_tokens,
                output_tokens,
                model: model.to_string(),
                provider: model_info.litellm_provider.clone(),
                cost_type: CostType::CharacterBased,
            })
        } else {
            // Fall back to token-based
            self.calculate_token_based_cost(model, model_info, input_tokens, output_tokens)
        }
    }

    /// Calculate time-based cost (for deployment providers)
    fn calculate_time_based_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        total_time_seconds: f64,
    ) -> Result<CostResult> {
        let cost_per_second = model_info.cost_per_second.unwrap_or(0.0);
        let total_cost = total_time_seconds * cost_per_second;

        Ok(CostResult {
            input_cost: 0.0,
            output_cost: 0.0,
            total_cost,
            input_tokens: 0,
            output_tokens: 0,
            model: model.to_string(),
            provider: model_info.litellm_provider.clone(),
            cost_type: CostType::TimeBased,
        })
    }

    /// Get cost per token for a model
    pub fn get_cost_per_token(&self, model: &str) -> Option<(f64, f64)> {
        let model_info = self.get_model_info(model)?;
        Some((
            model_info.input_cost_per_token.unwrap_or(0.0),
            model_info.output_cost_per_token.unwrap_or(0.0),
        ))
    }

    /// Check if model supports a feature
    pub fn supports_feature(&self, model: &str, feature: &str) -> bool {
        let model_info = match self.get_model_info(model) {
            Some(info) => info,
            None => return false,
        };

        match feature {
            "function_calling" => model_info.supports_function_calling.unwrap_or(false),
            "vision" => model_info.supports_vision.unwrap_or(false),
            "streaming" => model_info.supports_streaming.unwrap_or(true), // Default to true
            "parallel_function_calling" => model_info
                .supports_parallel_function_calling
                .unwrap_or(false),
            "system_message" => model_info.supports_system_message.unwrap_or(true),
            _ => false,
        }
    }

    /// Get all available models for a provider
    pub fn get_models_by_provider(&self, provider: &str) -> Vec<String> {
        let data = self.pricing_data.read();
        data.models
            .iter()
            .filter(|(_, info)| info.litellm_provider == provider)
            .map(|(model, _)| model.clone())
            .collect()
    }

    /// Get all available providers
    pub fn get_providers(&self) -> Vec<String> {
        let data = self.pricing_data.read();
        let mut providers: Vec<String> = data
            .models
            .values()
            .map(|info| info.litellm_provider.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        providers.sort();
        providers
    }

    /// Add custom model pricing
    pub fn add_custom_model(&self, model: String, model_info: ModelInfo) {
        let provider = model_info.litellm_provider.clone();
        {
            let mut data = self.pricing_data.write();
            data.models.insert(model.clone(), model_info);
        }

        // Send update event
        let _ = self.event_sender.send(PricingUpdateEvent {
            event_type: PricingEventType::ModelAdded,
            model,
            provider,
            timestamp: SystemTime::now(),
        });
    }

    /// Get pricing statistics
    pub fn get_statistics(&self) -> PricingStatistics {
        let data = self.pricing_data.read();
        let total_models = data.models.len();

        let mut provider_stats = HashMap::new();
        let mut cost_ranges = HashMap::new();

        for (_, model_info) in data.models.iter() {
            let provider = &model_info.litellm_provider;
            *provider_stats.entry(provider.clone()).or_insert(0) += 1;

            // Track cost ranges
            if let (Some(input_cost), Some(output_cost)) = (
                model_info.input_cost_per_token,
                model_info.output_cost_per_token,
            ) {
                let range = cost_ranges.entry(provider.clone()).or_insert(CostRange {
                    input_min: f64::MAX,
                    input_max: f64::MIN,
                    output_min: f64::MAX,
                    output_max: f64::MIN,
                });

                range.input_min = range.input_min.min(input_cost);
                range.input_max = range.input_max.max(input_cost);
                range.output_min = range.output_min.min(output_cost);
                range.output_max = range.output_max.max(output_cost);
            }
        }

        PricingStatistics {
            total_models,
            provider_stats,
            cost_ranges,
            last_updated: data.last_updated,
        }
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
