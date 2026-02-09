//! Router metrics collection and reporting

use crate::utils::error::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Router metrics collector
pub struct RouterMetrics {
    /// Consolidated metrics data - single lock for all metrics
    metrics_data: Arc<RwLock<MetricsData>>,
    /// Start time
    start_time: Instant,
}

/// Consolidated metrics data - single lock for all router metrics
#[derive(Debug, Default)]
struct MetricsData {
    /// Request metrics by provider
    provider: HashMap<String, ProviderMetrics>,
    /// Model metrics
    model: HashMap<String, ModelMetrics>,
    /// Overall metrics
    overall: OverallMetrics,
}

/// Metrics for a specific provider
#[derive(Debug, Clone, Default)]
pub struct ProviderMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total response time
    pub total_response_time: Duration,
    /// Minimum response time
    pub min_response_time: Option<Duration>,
    /// Maximum response time
    pub max_response_time: Option<Duration>,
    /// Last request time
    pub last_request: Option<Instant>,
    /// Error counts by type
    pub error_counts: HashMap<String, u64>,
}

/// Metrics for a specific model
#[derive(Debug, Clone, Default)]
pub struct ModelMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total response time
    pub total_response_time: Duration,
    /// Providers used for this model
    pub providers_used: HashMap<String, u64>,
}

/// Overall router metrics
#[derive(Debug, Clone)]
pub struct OverallMetrics {
    /// Total requests across all providers
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total response time
    pub total_response_time: Duration,
    /// Requests per second (calculated)
    pub requests_per_second: f64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Last calculation time
    pub last_calculation: Instant,
}

impl Default for OverallMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_response_time: Duration::from_secs(0),
            requests_per_second: 0.0,
            avg_response_time: Duration::from_secs(0),
            last_calculation: Instant::now(),
        }
    }
}

impl RouterMetrics {
    /// Create a new router metrics collector
    pub async fn new() -> Result<Self> {
        info!("Creating router metrics collector");

        Ok(Self {
            metrics_data: Arc::new(RwLock::new(MetricsData::default())),
            start_time: Instant::now(),
        })
    }

    /// Record a request - single lock for all updates
    pub async fn record_request(
        &self,
        provider: &str,
        model: &str,
        duration: Duration,
        success: bool,
    ) {
        debug!(
            "Recording request: provider={}, model={}, duration={:?}, success={}",
            provider, model, duration, success
        );

        let mut data = self.metrics_data.write();

        // Update provider metrics
        {
            let metrics = data.provider.entry(provider.to_string()).or_default();

            metrics.total_requests += 1;
            if success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
            }

            metrics.total_response_time += duration;
            metrics.last_request = Some(Instant::now());

            // Update min/max response times
            if metrics.min_response_time.is_none_or(|min| duration < min) {
                metrics.min_response_time = Some(duration);
            }
            if metrics.max_response_time.is_none_or(|max| duration > max) {
                metrics.max_response_time = Some(duration);
            }
        }

        // Update model metrics
        {
            let metrics = data.model.entry(model.to_string()).or_default();

            metrics.total_requests += 1;
            if success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
            }

            metrics.total_response_time += duration;

            // Track provider usage for this model
            *metrics
                .providers_used
                .entry(provider.to_string())
                .or_insert(0) += 1;
        }

        // Update overall metrics
        {
            data.overall.total_requests += 1;
            if success {
                data.overall.successful_requests += 1;
            } else {
                data.overall.failed_requests += 1;
            }
            data.overall.total_response_time += duration;
        }
    }

    /// Record an error
    pub async fn record_error(&self, provider: &str, error_type: &str) {
        debug!(
            "Recording error: provider={}, error_type={}",
            provider, error_type
        );

        let mut data = self.metrics_data.write();
        let metrics = data.provider.entry(provider.to_string()).or_default();
        *metrics
            .error_counts
            .entry(error_type.to_string())
            .or_insert(0) += 1;
    }

    /// Get metrics snapshot
    pub async fn get_snapshot(&self) -> Result<RouterMetricsSnapshot> {
        let mut data = self.metrics_data.write();

        // Calculate derived metrics
        let uptime = self.start_time.elapsed();
        let total_requests = data.overall.total_requests;

        data.overall.requests_per_second = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        data.overall.avg_response_time = if total_requests > 0 {
            data.overall.total_response_time / total_requests as u32
        } else {
            Duration::ZERO
        };

        data.overall.last_calculation = Instant::now();

        Ok(RouterMetricsSnapshot {
            provider_metrics: data.provider.clone(),
            model_metrics: data.model.clone(),
            overall_metrics: data.overall.clone(),
            uptime,
            timestamp: Instant::now(),
        })
    }

    /// Get provider metrics
    pub async fn get_provider_metrics(&self, provider: &str) -> Result<Option<ProviderMetrics>> {
        let data = self.metrics_data.read();
        Ok(data.provider.get(provider).cloned())
    }

    /// Get model metrics
    pub async fn get_model_metrics(&self, model: &str) -> Result<Option<ModelMetrics>> {
        let data = self.metrics_data.read();
        Ok(data.model.get(model).cloned())
    }

    /// Get top providers by request count
    pub async fn get_top_providers(&self, limit: usize) -> Result<Vec<(String, u64)>> {
        let data = self.metrics_data.read();
        let mut providers: Vec<(String, u64)> = data
            .provider
            .iter()
            .map(|(name, metrics)| (name.clone(), metrics.total_requests))
            .collect();

        providers.sort_by(|a, b| b.1.cmp(&a.1));
        providers.truncate(limit);

        Ok(providers)
    }

    /// Get top models by request count
    pub async fn get_top_models(&self, limit: usize) -> Result<Vec<(String, u64)>> {
        let data = self.metrics_data.read();
        let mut models: Vec<(String, u64)> = data
            .model
            .iter()
            .map(|(name, metrics)| (name.clone(), metrics.total_requests))
            .collect();

        models.sort_by(|a, b| b.1.cmp(&a.1));
        models.truncate(limit);

        Ok(models)
    }

    /// Reset all metrics
    pub async fn reset(&self) -> Result<()> {
        info!("Resetting router metrics");

        let mut data = self.metrics_data.write();
        data.provider.clear();
        data.model.clear();
        data.overall = OverallMetrics::default();

        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        use std::fmt::Write;

        let snapshot = self.get_snapshot().await?;
        // Pre-allocate buffer with estimated size to avoid reallocations
        let estimated_size = 1024 + snapshot.provider_metrics.len() * 256;
        let mut output = String::with_capacity(estimated_size);

        // Overall metrics - use write! macro for efficient formatting
        output.push_str("# HELP router_requests_total Total number of requests\n");
        output.push_str("# TYPE router_requests_total counter\n");
        let _ = writeln!(
            output,
            "router_requests_total {}",
            snapshot.overall_metrics.total_requests
        );

        output.push_str(
            "# HELP router_requests_successful_total Total number of successful requests\n",
        );
        output.push_str("# TYPE router_requests_successful_total counter\n");
        let _ = writeln!(
            output,
            "router_requests_successful_total {}",
            snapshot.overall_metrics.successful_requests
        );

        output.push_str("# HELP router_requests_failed_total Total number of failed requests\n");
        output.push_str("# TYPE router_requests_failed_total counter\n");
        let _ = writeln!(
            output,
            "router_requests_failed_total {}",
            snapshot.overall_metrics.failed_requests
        );

        output.push_str("# HELP router_response_time_seconds Average response time in seconds\n");
        output.push_str("# TYPE router_response_time_seconds gauge\n");
        let _ = writeln!(
            output,
            "router_response_time_seconds {:.6}",
            snapshot.overall_metrics.avg_response_time.as_secs_f64()
        );

        // Provider metrics
        for (provider, metrics) in &snapshot.provider_metrics {
            let _ = writeln!(
                output,
                "router_provider_requests_total{{provider=\"{}\"}} {}",
                provider, metrics.total_requests
            );
            let _ = writeln!(
                output,
                "router_provider_requests_successful_total{{provider=\"{}\"}} {}",
                provider, metrics.successful_requests
            );
            let _ = writeln!(
                output,
                "router_provider_requests_failed_total{{provider=\"{}\"}} {}",
                provider, metrics.failed_requests
            );
        }

        Ok(output)
    }
}

/// Router metrics snapshot
#[derive(Debug, Clone)]
pub struct RouterMetricsSnapshot {
    /// Provider metrics
    pub provider_metrics: HashMap<String, ProviderMetrics>,
    /// Model metrics
    pub model_metrics: HashMap<String, ModelMetrics>,
    /// Overall metrics
    pub overall_metrics: OverallMetrics,
    /// Router uptime
    pub uptime: Duration,
    /// Snapshot timestamp
    pub timestamp: Instant,
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
