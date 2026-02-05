//! Shared HTTP client utilities for providers.

use std::time::Duration;

use reqwest::Client;

use crate::core::providers::unified_provider::ProviderError;

/// Create a provider-scoped HTTP client with a configurable timeout.
pub fn create_http_client(
    provider: &'static str,
    timeout: Duration,
) -> Result<Client, ProviderError> {
    Client::builder().timeout(timeout).build().map_err(|e| {
        ProviderError::initialization(provider, format!("Failed to create HTTP client: {}", e))
    })
}
