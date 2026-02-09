//! Type definitions for the pricing service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// LiteLLM compatible model pricing data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Maximum total tokens
    pub max_tokens: Option<u32>,
    /// Maximum input tokens
    pub max_input_tokens: Option<u32>,
    /// Maximum output tokens
    pub max_output_tokens: Option<u32>,
    /// Input cost per token
    pub input_cost_per_token: Option<f64>,
    /// Output cost per token
    pub output_cost_per_token: Option<f64>,
    /// Input cost per character (for some providers)
    pub input_cost_per_character: Option<f64>,
    /// Output cost per character (for some providers)
    pub output_cost_per_character: Option<f64>,
    /// Cost per second (for time-based providers)
    pub cost_per_second: Option<f64>,
    /// LiteLLM provider name
    pub litellm_provider: String,
    /// Model mode (chat, completion, embedding, etc.)
    pub mode: String,
    /// Supports function calling
    pub supports_function_calling: Option<bool>,
    /// Supports vision
    pub supports_vision: Option<bool>,
    /// Supports streaming
    pub supports_streaming: Option<bool>,
    /// Supports parallel function calling
    pub supports_parallel_function_calling: Option<bool>,
    /// Supports system message
    pub supports_system_message: Option<bool>,
    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Consolidated pricing data - single lock for all pricing state
#[derive(Debug)]
pub(super) struct PricingData {
    /// Model pricing data (model_name -> ModelInfo)
    pub models: HashMap<String, ModelInfo>,
    /// Last update time
    pub last_updated: SystemTime,
}

impl Default for PricingData {
    fn default() -> Self {
        Self {
            models: HashMap::new(),
            last_updated: SystemTime::UNIX_EPOCH,
        }
    }
}

/// Pricing update event
/// Event for pricing updates
#[derive(Debug, Clone)]
pub struct PricingUpdateEvent {
    /// Type of pricing event that occurred
    pub event_type: PricingEventType,
    /// Model name that was affected
    pub model: String,
    /// Provider name that was affected
    pub provider: String,
    /// When the event occurred
    pub timestamp: SystemTime,
}

/// Types of pricing events that can occur
#[derive(Debug, Clone)]
pub enum PricingEventType {
    /// A new model was added to the pricing data
    ModelAdded,
    /// An existing model's pricing was updated
    ModelUpdated,
    /// A model was removed from the pricing data
    ModelRemoved,
    /// The entire pricing dataset was refreshed
    DataRefreshed,
}

/// Cost calculation result
#[derive(Debug, Clone, Serialize)]
pub struct CostResult {
    /// Cost for input tokens/characters
    pub input_cost: f64,
    /// Cost for output tokens/characters
    pub output_cost: f64,
    /// Total cost (input + output)
    pub total_cost: f64,
    /// Number of input tokens used
    pub input_tokens: u32,
    /// Number of output tokens used
    pub output_tokens: u32,
    /// The model name used for pricing calculation
    pub model: String,
    /// The provider name (e.g., "openai", "anthropic")
    pub provider: String,
    /// The type of cost calculation used
    pub cost_type: CostType,
}

/// Type of cost calculation method
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum CostType {
    /// Cost calculated based on token count
    TokenBased,
    /// Cost calculated based on character count
    CharacterBased,
    /// Cost calculated based on time duration
    TimeBased,
    /// Custom cost calculation method
    Custom,
}

/// Pricing statistics
#[derive(Debug, Clone)]
pub struct PricingStatistics {
    /// Total number of models in the pricing database
    pub total_models: usize,
    /// Number of models per provider
    pub provider_stats: HashMap<String, usize>,
    /// Cost ranges for each provider
    pub cost_ranges: HashMap<String, CostRange>,
    /// When the pricing data was last updated
    pub last_updated: SystemTime,
}

/// Cost range statistics for a provider
#[derive(Debug, Clone)]
pub struct CostRange {
    /// Minimum input cost per token
    pub input_min: f64,
    /// Maximum input cost per token
    pub input_max: f64,
    /// Minimum output cost per token
    pub output_min: f64,
    /// Maximum output cost per token
    pub output_max: f64,
}

// ====================================================================================
// TESTS
// ====================================================================================

#[cfg(test)]
mod types_tests;
