//! Performance optimization utilities
//!
//! This module provides utilities to optimize runtime performance,
//! reduce allocations, and improve overall system efficiency.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Function call counts
    pub call_counts: Arc<RwLock<HashMap<String, u64>>>,
    /// Function execution times
    pub execution_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    /// Memory allocation tracking
    pub allocation_counts: Arc<RwLock<HashMap<String, u64>>>,
    /// Cache hit rates
    pub cache_stats: Arc<RwLock<HashMap<String, CacheStats>>>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total_requests: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    pub fn new() -> Self {
        Self {
            call_counts: Arc::new(RwLock::new(HashMap::new())),
            execution_times: Arc::new(RwLock::new(HashMap::new())),
            allocation_counts: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a function call
    pub fn record_call(&self, function_name: &str) {
        let mut counts = self.call_counts.write();
        *counts.entry(function_name.to_string()).or_insert(0) += 1;
    }

    /// Record function execution time
    pub fn record_execution_time(&self, function_name: &str, duration: Duration) {
        let mut times = self.execution_times.write();
        times
            .entry(function_name.to_string())
            .or_default()
            .push(duration);
    }

    /// Record memory allocation
    pub fn record_allocation(&self, context: &str) {
        let mut counts = self.allocation_counts.write();
        *counts.entry(context.to_string()).or_insert(0) += 1;
    }

    /// Record cache hit
    pub fn record_cache_hit(&self, cache_name: &str) {
        let mut stats = self.cache_stats.write();
        let cache_stat = stats.entry(cache_name.to_string()).or_default();
        cache_stat.hits += 1;
        cache_stat.total_requests += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&self, cache_name: &str) {
        let mut stats = self.cache_stats.write();
        let cache_stat = stats.entry(cache_name.to_string()).or_default();
        cache_stat.misses += 1;
        cache_stat.total_requests += 1;
    }

    /// Get performance report
    pub fn get_report(&self) -> PerformanceReport {
        let call_counts = self.call_counts.read().clone();
        let execution_times = self.execution_times.read().clone();
        let allocation_counts = self.allocation_counts.read().clone();
        let cache_stats = self.cache_stats.read().clone();

        PerformanceReport {
            call_counts,
            execution_times,
            allocation_counts,
            cache_stats,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.call_counts.write().clear();
        self.execution_times.write().clear();
        self.allocation_counts.write().clear();
        self.cache_stats.write().clear();
    }
}

/// Performance report
#[derive(Debug)]
pub struct PerformanceReport {
    pub call_counts: HashMap<String, u64>,
    pub execution_times: HashMap<String, Vec<Duration>>,
    pub allocation_counts: HashMap<String, u64>,
    pub cache_stats: HashMap<String, CacheStats>,
}

impl PerformanceReport {
    /// Print performance summary
    pub fn print_summary(&self) {
        info!("=== Performance Report ===");

        // Top called functions
        let mut sorted_calls: Vec<_> = self.call_counts.iter().collect();
        sorted_calls.sort_by(|a, b| b.1.cmp(a.1));

        info!("Top 10 Most Called Functions:");
        for (func, count) in sorted_calls.iter().take(10) {
            info!("  {}: {} calls", func, count);
        }

        // Slowest functions
        info!("Function Execution Times:");
        for (func, times) in &self.execution_times {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                let Some(max) = times.iter().max() else {
                    continue;
                };
                info!(
                    "  {}: avg={:?}, max={:?}, calls={}",
                    func,
                    avg,
                    max,
                    times.len()
                );
            }
        }

        // Memory allocations
        let mut sorted_allocs: Vec<_> = self.allocation_counts.iter().collect();
        sorted_allocs.sort_by(|a, b| b.1.cmp(a.1));

        info!("Top Memory Allocation Sources:");
        for (context, count) in sorted_allocs.iter().take(10) {
            info!("  {}: {} allocations", context, count);
        }

        // Cache performance
        info!("Cache Performance:");
        for (cache, stats) in &self.cache_stats {
            info!(
                "  {}: hit_rate={:.2}%, hits={}, misses={}",
                cache,
                stats.hit_rate() * 100.0,
                stats.hits,
                stats.misses
            );
        }
    }

    /// Get optimization recommendations
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for frequently called functions
        for (func, count) in &self.call_counts {
            if *count > 10000 {
                recommendations.push(format!(
                    "Consider optimizing '{}' - called {} times",
                    func, count
                ));
            }
        }

        // Check for slow functions
        for (func, times) in &self.execution_times {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                if avg > Duration::from_millis(100) {
                    recommendations.push(format!(
                        "Function '{}' is slow - average execution time: {:?}",
                        func, avg
                    ));
                }
            }
        }

        // Check cache hit rates
        for (cache, stats) in &self.cache_stats {
            if stats.hit_rate() < 0.8 && stats.total_requests > 100 {
                recommendations.push(format!(
                    "Cache '{}' has low hit rate: {:.2}% - consider tuning cache size or TTL",
                    cache,
                    stats.hit_rate() * 100.0
                ));
            }
        }

        // Check for excessive allocations
        for (context, count) in &self.allocation_counts {
            if *count > 50000 {
                recommendations.push(format!(
                    "High allocation count in '{}': {} - consider object pooling",
                    context, count
                ));
            }
        }

        recommendations
    }
}

/// Performance timer for measuring execution time
pub struct PerformanceTimer {
    start: Instant,
    function_name: String,
    metrics: Arc<PerformanceMetrics>,
}

impl PerformanceTimer {
    /// Start timing a function
    pub fn start(function_name: &str, metrics: Arc<PerformanceMetrics>) -> Self {
        metrics.record_call(function_name);
        Self {
            start: Instant::now(),
            function_name: function_name.to_string(),
            metrics,
        }
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics
            .record_execution_time(&self.function_name, duration);

        if duration > Duration::from_millis(100) {
            warn!(
                "Slow function detected: {} took {:?}",
                self.function_name, duration
            );
        }
    }
}

/// Macro for easy performance timing
#[macro_export]
macro_rules! time_function {
    ($metrics:expr, $func_name:expr, $body:block) => {{
        let _timer = $crate::utils::performance_optimizer::PerformanceTimer::start(
            $func_name,
            $metrics.clone(),
        );
        $body
    }};
}

/// Global performance metrics instance
use std::sync::LazyLock;
pub static GLOBAL_METRICS: LazyLock<Arc<PerformanceMetrics>> =
    LazyLock::new(|| Arc::new(PerformanceMetrics::new()));

/// Convenience function to get global metrics
pub fn global_metrics() -> Arc<PerformanceMetrics> {
    GLOBAL_METRICS.clone()
}

/// Start performance monitoring background task
pub async fn start_performance_monitoring() {
    let metrics = global_metrics();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            let report = metrics.get_report();
            debug!("Performance monitoring tick");

            // Print recommendations if any
            let recommendations = report.get_recommendations();
            if !recommendations.is_empty() {
                warn!("Performance recommendations:");
                for rec in recommendations {
                    warn!("  - {}", rec);
                }
            }
        }
    });
}

#[cfg(test)]
#[path = "optimizer_tests.rs"]
mod tests;
