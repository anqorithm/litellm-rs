//! Unified pricing calculation system
//!
//! Uses `data/model_prices.json` as the single source of truth.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use tracing::warn;

pub const DEFAULT_PRICING_FILE: &str = "data/model_prices.json";

/// Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per input token
    #[serde(default)]
    pub input_cost_per_token: f64,

    /// Cost per output token
    #[serde(default)]
    pub output_cost_per_token: f64,

    /// Model
    #[serde(default)]
    pub output_cost_per_reasoning_token: f64,

    /// Maximum token count (compatible with legacy field)
    #[serde(default)]
    pub max_tokens: Option<u32>,

    /// Maximum input token count
    #[serde(default)]
    pub max_input_tokens: Option<u32>,

    /// Maximum output token count
    #[serde(default)]
    pub max_output_tokens: Option<u32>,

    /// Provider name
    #[serde(default)]
    pub litellm_provider: Option<String>,

    /// Mode (chat, embedding, completion, etc.)
    #[serde(default)]
    pub mode: Option<String>,

    /// Whether function calling is supported
    #[serde(default)]
    pub supports_function_calling: Option<bool>,

    /// Whether vision is supported
    #[serde(default)]
    pub supports_vision: Option<bool>,
}

/// Usage information
#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub reasoning_tokens: Option<u32>,
}

/// Pricing database
#[derive(Debug, Clone)]
pub struct PricingDatabase {
    models: HashMap<String, ModelPricing>,
}

impl PricingDatabase {
    pub fn empty() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Load pricing data from JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read pricing file: {}", e))?;

        let all_data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse pricing JSON: {}", e))?;

        // Filter out entries that are not actual models (e.g., sample_spec)
        let mut models = HashMap::new();
        for (key, value) in all_data {
            // Skip documentation and sample entries
            if key == "sample_spec" || key.starts_with("_") || key.contains("example") {
                continue;
            }

            // Try to parse value as ModelPricing
            match serde_json::from_value::<ModelPricing>(value) {
                Ok(pricing) => {
                    models.insert(key, pricing);
                }
                Err(e) => {
                    warn!(
                        model = %key,
                        error = %e,
                        "Failed to parse model pricing data, skipping model"
                    );
                }
            }
        }

        Ok(Self { models })
    }

    /// Load from the repository canonical pricing file.
    pub fn from_default_file() -> Result<Self, String> {
        Self::from_json_file(DEFAULT_PRICING_FILE)
    }

    /// Calculate cost
    pub fn calculate(&self, model: &str, usage: &Usage) -> f64 {
        if let Some(pricing) = self.models.get(model) {
            return self.calculate_with_pricing(pricing, usage);
        }

        0.0
    }

    /// Calculate cost using specified pricing information from usage
    fn calculate_with_pricing(&self, pricing: &ModelPricing, usage: &Usage) -> f64 {
        let mut cost = 0.0;

        // Input token cost
        cost += usage.prompt_tokens as f64 * pricing.input_cost_per_token;

        // Output token cost
        cost += usage.completion_tokens as f64 * pricing.output_cost_per_token;

        // Reasoning token cost (if available)
        if let Some(reasoning_tokens) = usage.reasoning_tokens {
            cost += reasoning_tokens as f64 * pricing.output_cost_per_reasoning_token;
        }

        cost
    }

    /// Model
    pub fn get_model_info(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model)
    }

    /// Model
    pub fn get_max_tokens(&self, model: &str) -> Option<u32> {
        self.get_model_info(model).and_then(|info| {
            info.max_tokens
                .or(info.max_input_tokens)
                .or(info.max_output_tokens)
        })
    }

    /// Model
    pub fn get_provider_models(&self, provider: &str) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(model_id, pricing)| {
                if pricing.litellm_provider.as_deref() == Some(provider) {
                    Some(model_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create
    pub fn to_model_info(
        &self,
        model_id: &str,
        provider: &str,
    ) -> Option<crate::core::types::model::ModelInfo> {
        use crate::core::types::model::ModelInfo;
        use std::collections::HashMap;

        let pricing = self.get_model_info(model_id)?;

        Some(ModelInfo {
            id: model_id.to_string(),
            name: model_id
                .rsplit('/')
                .next()
                .unwrap_or(model_id)
                .replace(['-', '_'], " "),
            provider: provider.to_string(),
            max_context_length: pricing
                .max_input_tokens
                .unwrap_or_else(|| pricing.max_tokens.unwrap_or(4096)),
            max_output_length: pricing.max_output_tokens,
            supports_streaming: true, // Most modern models support streaming
            supports_tools: pricing.supports_function_calling.unwrap_or(false),
            supports_multimodal: pricing.supports_vision.unwrap_or(false),
            input_cost_per_1k_tokens: Some(pricing.input_cost_per_token * 1000.0),
            output_cost_per_1k_tokens: Some(pricing.output_cost_per_token * 1000.0),
            currency: "USD".to_string(),
            capabilities: vec![], // Can be extended later
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        })
    }

    /// Check
    pub fn supports_feature(&self, model: &str, feature: &str) -> bool {
        self.get_model_info(model)
            .map(|info| match feature {
                "function_calling" => info.supports_function_calling.unwrap_or(false),
                "vision" => info.supports_vision.unwrap_or(false),
                _ => false,
            })
            .unwrap_or(false)
    }
}

impl Default for PricingDatabase {
    fn default() -> Self {
        // Built-in pricing for some common models as backup
        let mut models = HashMap::new();

        // OpenAI models
        models.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00003,
                output_cost_per_token: 0.00006,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(8192),
                max_input_tokens: Some(8192),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        models.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00001,
                output_cost_per_token: 0.00003,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_cost_per_token: 0.0000005,
                output_cost_per_token: 0.0000015,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(16385),
                max_input_tokens: Some(16385),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        // Anthropic models
        models.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_cost_per_token: 0.000015,
                output_cost_per_token: 0.000075,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(200000),
                max_input_tokens: Some(200000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("anthropic".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        models.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                input_cost_per_token: 0.000003,
                output_cost_per_token: 0.000015,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(200000),
                max_input_tokens: Some(200000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("anthropic".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        // DeepSeek models - updated pricing
        models.insert(
            "deepseek-chat".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00000056,  // $0.56 per 1M tokens
                output_cost_per_token: 0.00000168, // $1.68 per 1M tokens
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(8192),
                litellm_provider: Some("deepseek".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        models.insert(
            "deepseek-reasoner".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00000056,  // $0.56 per 1M tokens
                output_cost_per_token: 0.00000168, // $1.68 per 1M tokens
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(8192),
                litellm_provider: Some("deepseek".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        Self { models }
    }
}

// Global pricing database (lazy loading)
pub static GLOBAL_PRICING_DB: LazyLock<PricingDatabase> = LazyLock::new(|| {
    PricingDatabase::from_default_file().unwrap_or_else(|e| {
        warn!(
            error = %e,
            file = DEFAULT_PRICING_FILE,
            "Failed to load pricing data from canonical file"
        );
        PricingDatabase::empty()
    })
});

/// Get
pub fn get_pricing_db() -> &'static PricingDatabase {
    &GLOBAL_PRICING_DB
}

/// Quick cost calculation
pub fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    let usage = Usage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        reasoning_tokens: None,
    };

    GLOBAL_PRICING_DB.calculate(model, &usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pricing() {
        let db = PricingDatabase::default();

        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            reasoning_tokens: None,
        };

        // Test GPT-4 pricing
        let cost = db.calculate("gpt-4", &usage);
        assert!(cost > 0.0);
        assert_eq!(cost, 1000.0 * 0.00003 + 500.0 * 0.00006);

        // Test Claude pricing
        let cost = db.calculate("claude-3-opus", &usage);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_model_info() {
        let db = PricingDatabase::default();

        assert!(db.get_model_info("gpt-4").is_some());
        assert!(db.get_model_info("non-existent-model").is_none());

        assert_eq!(db.get_max_tokens("gpt-4"), Some(8192));
        assert!(db.supports_feature("gpt-4", "function_calling"));
        assert!(!db.supports_feature("gpt-4", "vision"));
        assert!(db.supports_feature("gpt-4-turbo", "vision"));
    }

    #[test]
    fn test_quick_calculate() {
        let cost = calculate_cost("openai/gpt-3.5-turbo", 1000, 500);
        assert!(cost > 0.0);
    }
}
