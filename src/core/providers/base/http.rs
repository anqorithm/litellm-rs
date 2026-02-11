//! Shared HTTP client utilities for providers.

use std::time::Duration;

use reqwest::Client;

use crate::core::providers::unified_provider::ProviderError;
use crate::utils::net::http::get_client_with_timeout_fallible;

/// Create a provider-scoped HTTP client with a configurable timeout.
pub fn create_http_client(
    provider: &'static str,
    timeout: Duration,
) -> Result<Client, ProviderError> {
    get_client_with_timeout_fallible(timeout)
        .map(|shared_client| (*shared_client).clone())
        .map_err(|e| {
            ProviderError::initialization(provider, format!("Failed to create HTTP client: {}", e))
        })
}
