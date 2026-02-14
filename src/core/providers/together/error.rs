//! Error types for Together AI provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Together AI error type (alias to unified ProviderError)
pub type TogetherError = ProviderError;

crate::define_standard_error_mapper!("together", Together);
