//! Statistics and metrics methods

use super::llm_client::LLMClient;
use super::types::ProviderStats;
use crate::sdk::errors::Result;
use crate::sdk::types::ChatResponse;
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::debug;

impl LLMClient {
    /// Update provider statistics after a request
    pub(crate) async fn update_provider_stats(
        &self,
        provider_id: &str,
        start_time: SystemTime,
        result: &Result<ChatResponse>,
    ) {
        let mut stats = self.provider_stats.write().await;
        let provider_stats = stats.entry(provider_id.to_string()).or_default();

        provider_stats.requests += 1;
        provider_stats.last_used = Some(SystemTime::now());

        if let Ok(elapsed) = start_time.elapsed() {
            let latency_ms = elapsed.as_millis() as f64;
            provider_stats.avg_latency_ms = if provider_stats.requests == 1 {
                latency_ms
            } else {
                (provider_stats.avg_latency_ms * (provider_stats.requests - 1) as f64 + latency_ms)
                    / provider_stats.requests as f64
            };
        }

        match result {
            Ok(response) => {
                provider_stats.total_tokens += response.usage.total_tokens as u64;
                provider_stats.health_score = (provider_stats.health_score * 0.9 + 0.1).min(1.0);
            }
            Err(_) => {
                provider_stats.errors += 1;
                provider_stats.health_score = (provider_stats.health_score * 0.9).max(0.1);
            }
        }

        debug!(
            "Updated stats for provider {}: requests={}, errors={}, health={:.2}",
            provider_id,
            provider_stats.requests,
            provider_stats.errors,
            provider_stats.health_score
        );
    }

    /// Get provider statistics
    pub async fn get_provider_stats(&self) -> HashMap<String, ProviderStats> {
        self.provider_stats.read().await.clone()
    }
}
