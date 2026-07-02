//! Metrics collection and export

use super::histogram::BoundedHistogram;
use super::types::TokenUsage;
use crate::core::http::outbound::default_outbound_client;
use crate::utils::error::gateway_error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

/// Prometheus metrics structure
#[derive(Debug, Default)]
pub struct PrometheusMetrics {
    /// Request counters
    pub request_total: HashMap<String, u64>,
    /// Request duration histograms (bounded to prevent memory leaks)
    pub request_duration: HashMap<String, BoundedHistogram>,
    /// Error counters
    pub error_total: HashMap<String, u64>,
    /// Token usage counters
    pub token_usage: HashMap<String, u64>,
    /// Cost tracking
    pub cost_total: HashMap<String, f64>,
    /// Provider health status
    pub provider_health: HashMap<String, f64>,
    /// Cache hit/miss ratios
    pub cache_hits: u64,
    pub cache_misses: u64,
    /// Active connections
    pub active_connections: u64,
    /// Queue sizes
    pub queue_size: HashMap<String, u64>,
}

/// DataDog client for metrics
pub struct DataDogClient {
    /// API key
    pub api_key: String,
    /// Base URL
    pub base_url: String,
    /// HTTP client
    pub client: reqwest::Client,
    /// Tags to add to all metrics
    pub default_tags: Vec<String>,
}

/// OpenTelemetry exporter
pub struct OtelExporter {
    /// Endpoint URL
    pub endpoint: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// HTTP client
    pub client: reqwest::Client,
}

/// Metrics collector and exporter
pub struct MetricsCollector {
    /// Prometheus metrics
    pub prometheus_metrics: Arc<RwLock<PrometheusMetrics>>,
    /// DataDog metrics
    datadog_client: Option<DataDogClient>,
    /// OpenTelemetry exporter
    otel_exporter: Option<OtelExporter>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            prometheus_metrics: Arc::new(RwLock::new(PrometheusMetrics::default())),
            datadog_client: None,
            otel_exporter: None,
        }
    }

    /// Configure DataDog integration
    pub fn with_datadog(mut self, api_key: String, site: String) -> Self {
        self.datadog_client = Some(DataDogClient {
            api_key,
            base_url: format!("https://api.{}", site),
            client: default_outbound_client().clone(),
            default_tags: vec![
                "service:litellm-gateway".to_string(),
                "env:production".to_string(),
            ],
        });
        self
    }

    /// Configure OpenTelemetry integration
    pub fn with_otel(mut self, endpoint: String, headers: HashMap<String, String>) -> Self {
        self.otel_exporter = Some(OtelExporter {
            endpoint,
            headers,
            client: default_outbound_client().clone(),
        });
        self
    }

    /// Record request metrics
    pub async fn record_request(
        &self,
        provider: &str,
        model: &str,
        duration: Duration,
        tokens: Option<TokenUsage>,
        cost: Option<f64>,
        success: bool,
    ) {
        let key = format!("{}:{}", provider, model);
        let duration_secs = duration.as_secs_f64();

        {
            let mut metrics = self.prometheus_metrics.write().await;

            // Request counter
            *metrics.request_total.entry(key.clone()).or_insert(0) += 1;

            // Duration histogram (bounded to prevent memory leaks)
            metrics
                .request_duration
                .entry(key.clone())
                .or_insert_with(BoundedHistogram::default)
                .record(duration_secs);

            // Error counter
            if !success {
                *metrics.error_total.entry(key.clone()).or_insert(0) += 1;
            }
        }

        // Token usage
        if let Some(token_usage) = tokens {
            let prompt_key = format!("{}:prompt", key);
            let completion_key = format!("{}:completion", key);
            let prompt_tokens = token_usage.prompt_tokens as u64;
            let completion_tokens = token_usage.completion_tokens as u64;

            let mut metrics = self.prometheus_metrics.write().await;
            *metrics.token_usage.entry(prompt_key).or_insert(0) += prompt_tokens;
            *metrics.token_usage.entry(completion_key).or_insert(0) += completion_tokens;
        }

        // Cost tracking
        if let Some(request_cost) = cost {
            let mut metrics = self.prometheus_metrics.write().await;
            *metrics.cost_total.entry(key).or_insert(0.0) += request_cost;
        }
    }

    /// Record cache metrics
    pub async fn record_cache_hit(&self, hit: bool) {
        let mut metrics = self.prometheus_metrics.write().await;
        if hit {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }
    }

    /// Update provider health
    pub async fn update_provider_health(&self, provider: &str, health_score: f64) {
        let mut metrics = self.prometheus_metrics.write().await;
        metrics
            .provider_health
            .insert(provider.to_string(), health_score);
    }

    /// Export metrics to Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.prometheus_metrics.read().await;
        let mut output = String::new();

        // Request total
        output.push_str("# HELP litellm_requests_total Total number of requests\n");
        output.push_str("# TYPE litellm_requests_total counter\n");
        for (key, value) in &metrics.request_total {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                output.push_str(&format!(
                    "litellm_requests_total{{provider=\"{}\",model=\"{}\"}} {}\n",
                    parts[0], parts[1], value
                ));
            }
        }

        // Error total
        output.push_str("# HELP litellm_errors_total Total number of errors\n");
        output.push_str("# TYPE litellm_errors_total counter\n");
        for (key, value) in &metrics.error_total {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                output.push_str(&format!(
                    "litellm_errors_total{{provider=\"{}\",model=\"{}\"}} {}\n",
                    parts[0], parts[1], value
                ));
            }
        }

        // Cache metrics
        output.push_str("# HELP litellm_cache_hits_total Total cache hits\n");
        output.push_str("# TYPE litellm_cache_hits_total counter\n");
        output.push_str(&format!(
            "litellm_cache_hits_total {}\n",
            metrics.cache_hits
        ));

        output.push_str("# HELP litellm_cache_misses_total Total cache misses\n");
        output.push_str("# TYPE litellm_cache_misses_total counter\n");
        output.push_str(&format!(
            "litellm_cache_misses_total {}\n",
            metrics.cache_misses
        ));

        // Provider health
        output.push_str("# HELP litellm_provider_health Provider health score\n");
        output.push_str("# TYPE litellm_provider_health gauge\n");
        for (provider, health) in &metrics.provider_health {
            output.push_str(&format!(
                "litellm_provider_health{{provider=\"{}\"}} {}\n",
                provider, health
            ));
        }

        output
    }

    /// Send metrics to DataDog
    pub async fn send_to_datadog(&self) -> Result<()> {
        if let Some(_client) = &self.datadog_client {
            let _metrics = self.prometheus_metrics.read().await;

            // Convert metrics to DataDog format and send
            // Implementation would depend on DataDog API format
            debug!("Sending metrics to DataDog");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
