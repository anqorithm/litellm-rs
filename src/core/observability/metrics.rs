//! Metrics collection and export

use super::histogram::BoundedHistogram;
use super::types::{MetricValue, TokenUsage};
use crate::utils::error::error::Result;
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
    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
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
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure DataDog integration
    pub fn with_datadog(mut self, api_key: String, site: String) -> Self {
        self.datadog_client = Some(DataDogClient {
            api_key,
            base_url: format!("https://api.{}", site),
            client: reqwest::Client::new(),
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
            client: reqwest::Client::new(),
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
        let mut metrics = self.prometheus_metrics.write().await;

        // Request counter
        let key = format!("{}:{}", provider, model);
        *metrics.request_total.entry(key.clone()).or_insert(0) += 1;

        // Duration histogram (bounded to prevent memory leaks)
        metrics
            .request_duration
            .entry(key.clone())
            .or_insert_with(BoundedHistogram::default)
            .record(duration.as_secs_f64());

        // Error counter
        if !success {
            *metrics.error_total.entry(key.clone()).or_insert(0) += 1;
        }

        // Token usage
        if let Some(token_usage) = tokens {
            *metrics
                .token_usage
                .entry(format!("{}:prompt", key))
                .or_insert(0) += token_usage.prompt_tokens as u64;
            *metrics
                .token_usage
                .entry(format!("{}:completion", key))
                .or_insert(0) += token_usage.completion_tokens as u64;
        }

        // Cost tracking
        if let Some(request_cost) = cost {
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

    /// Set a custom metric value.
    pub async fn set_custom_metric(&self, name: impl Into<String>, value: MetricValue) {
        self.custom_metrics.write().await.insert(name.into(), value);
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
        drop(metrics);

        // Custom metrics
        let custom_metrics = self.custom_metrics.read().await;
        for (name, metric) in custom_metrics.iter() {
            match metric {
                MetricValue::Counter(v) => {
                    output.push_str(&format!(
                        "litellm_custom_counter{{name=\"{}\"}} {}\n",
                        name, v
                    ));
                }
                MetricValue::Gauge(v) => {
                    output.push_str(&format!(
                        "litellm_custom_gauge{{name=\"{}\"}} {}\n",
                        name, v
                    ));
                }
                MetricValue::Summary { sum, count } => {
                    output.push_str(&format!(
                        "litellm_custom_summary_sum{{name=\"{}\"}} {}\n",
                        name, sum
                    ));
                    output.push_str(&format!(
                        "litellm_custom_summary_count{{name=\"{}\"}} {}\n",
                        name, count
                    ));
                }
                MetricValue::Histogram(values) => {
                    output.push_str(&format!(
                        "litellm_custom_histogram_count{{name=\"{}\"}} {}\n",
                        name,
                        values.len()
                    ));
                    let hist_sum: f64 = values.iter().sum();
                    output.push_str(&format!(
                        "litellm_custom_histogram_sum{{name=\"{}\"}} {}\n",
                        name, hist_sum
                    ));
                }
            }
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
