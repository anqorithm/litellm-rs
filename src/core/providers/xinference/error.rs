//! Error types for Xinference provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Xinference error type (alias to unified ProviderError)
pub type XinferenceError = ProviderError;

crate::define_standard_error_mapper!("xinference", Xinference);
