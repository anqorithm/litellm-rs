//! Metrics collector implementation for recording metrics

use super::background::{start_cleanup_task, start_system_metrics_collection};
use super::bounded::{BoundedPush, MAX_METRIC_SAMPLES, MAX_RECENT_EVENTS};
use super::types::MetricsStorage;
use crate::config::models::monitoring::MonitoringConfig;
use crate::utils::error::error::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::debug;

/// Metrics collector for gathering and aggregating system metrics
#[derive(Debug)]
pub struct MetricsCollector {
    /// Configuration
    pub(super) config: Arc<MonitoringConfig>,
    /// All metrics storage consolidated into a single lock
    /// This reduces lock contention and simplifies the code
    pub(super) storage: Arc<RwLock<MetricsStorage>>,
    /// Collection start time
    pub(super) start_time: Instant,
    /// Whether collection is active - using AtomicBool for lock-free access
    pub(super) active: AtomicBool,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub async fn new(config: &MonitoringConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clone()),
            storage: Arc::new(RwLock::new(MetricsStorage::default())),
            start_time: Instant::now(),
            active: AtomicBool::new(false),
        })
    }

    /// Start metrics collection
    pub async fn start(&self) -> Result<()> {
        debug!("Starting metrics collection");

        self.active.store(true, Ordering::Release);

        // Start background collection tasks
        start_system_metrics_collection(self).await;
        start_cleanup_task(self).await;

        Ok(())
    }

    /// Stop metrics collection
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping metrics collection");
        self.active.store(false, Ordering::Release);
        Ok(())
    }

    /// Check if metrics collection is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Record a request metric
    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        response_time: Duration,
        _user_id: Option<uuid::Uuid>,
        _api_key_id: Option<uuid::Uuid>,
    ) -> Result<()> {
        let mut storage = self.storage.write();
        let metrics = &mut storage.request;

        metrics.total_requests += 1;
        metrics
            .response_times
            .push_bounded(response_time.as_millis() as f64, MAX_METRIC_SAMPLES);
        *metrics.status_codes.entry(status_code).or_insert(0) += 1;

        let endpoint_key = format!("{} {}", method, path);
        *metrics.endpoints.entry(endpoint_key).or_insert(0) += 1;

        metrics
            .last_minute_requests
            .push_bounded(Instant::now(), MAX_RECENT_EVENTS);

        Ok(())
    }

    /// Record a provider request metric
    pub async fn record_provider_request(
        &self,
        provider: &str,
        _model: &str,
        tokens_used: u32,
        cost: f64,
        response_time: Duration,
        success: bool,
    ) -> Result<()> {
        let mut storage = self.storage.write();
        let metrics = &mut storage.provider;

        metrics.total_requests += 1;
        *metrics
            .provider_requests
            .entry(provider.to_string())
            .or_insert(0) += 1;

        metrics
            .provider_response_times
            .entry(provider.to_string())
            .or_default()
            .push_bounded(response_time.as_millis() as f64, MAX_METRIC_SAMPLES);

        if !success {
            *metrics
                .provider_errors
                .entry(provider.to_string())
                .or_insert(0) += 1;
        }

        *metrics.token_usage.entry(provider.to_string()).or_insert(0) += tokens_used as u64;
        *metrics.costs.entry(provider.to_string()).or_insert(0.0) += cost;

        Ok(())
    }

    /// Record an error metric
    pub async fn record_error(
        &self,
        error_type: &str,
        _error_message: &str,
        _context: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut storage = self.storage.write();
        let metrics = &mut storage.error;

        metrics.total_errors += 1;
        *metrics
            .error_types
            .entry(error_type.to_string())
            .or_insert(0) += 1;

        // Classify error severity
        if error_type.contains("critical") || error_type.contains("fatal") {
            metrics.critical_errors += 1;
        } else if error_type.contains("warning") || error_type.contains("warn") {
            metrics.warnings += 1;
        }

        metrics
            .last_minute_errors
            .push_bounded(Instant::now(), MAX_RECENT_EVENTS);

        Ok(())
    }

    /// Record cache hit
    pub async fn record_cache_hit(&self) -> Result<()> {
        self.storage.write().performance.cache_hits += 1;
        Ok(())
    }

    /// Record cache miss
    pub async fn record_cache_miss(&self) -> Result<()> {
        self.storage.write().performance.cache_misses += 1;
        Ok(())
    }

    /// Record database query time
    pub async fn record_db_query_time(&self, duration: Duration) -> Result<()> {
        self.storage
            .write()
            .performance
            .db_query_times
            .push_bounded(duration.as_millis() as f64, MAX_METRIC_SAMPLES);
        Ok(())
    }
}
