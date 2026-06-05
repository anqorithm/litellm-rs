//! Unified pricing service using LiteLLM pricing data format.
//!
//! This service loads pricing data from LiteLLM's JSON format and provides
//! unified cost calculation for all AI providers.

mod cache;
mod events;
mod loader;
mod service;
mod types;

#[cfg(test)]
mod tests;

/// Built-in local pricing source used by gateway defaults.
///
/// Relative user-configured paths remain filesystem paths. This value is an
/// explicit embedded source so the default does not depend on process cwd.
pub const DEFAULT_PRICING_SOURCE: &str = "embedded://model_prices_extended";

// Re-export public types
pub use service::PricingService;
pub use types::{
    CostRange, CostResult, CostType, LiteLLMModelInfo, PricingEventType, PricingStatistics,
    PricingUpdateEvent,
};
