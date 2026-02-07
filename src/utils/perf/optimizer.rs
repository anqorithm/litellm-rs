//! Performance optimization utilities
//!
//! This module provides utilities to optimize runtime performance,
//! reduce allocations, and improve overall system efficiency.

#![allow(dead_code)] // Tool module - functions may be used in the future

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
                let max = times.iter().max().unwrap();
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
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_metrics_new() {
        let metrics = PerformanceMetrics::new();
        let report = metrics.get_report();

        assert!(report.call_counts.is_empty());
        assert!(report.execution_times.is_empty());
        assert!(report.allocation_counts.is_empty());
        assert!(report.cache_stats.is_empty());
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        let report = metrics.get_report();

        assert!(report.call_counts.is_empty());
        assert!(report.execution_times.is_empty());
        assert!(report.allocation_counts.is_empty());
        assert!(report.cache_stats.is_empty());
    }

    #[test]
    fn test_record_call_single() {
        let metrics = PerformanceMetrics::new();
        metrics.record_call("test_function");

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("test_function"), Some(&1));
    }

    #[test]
    fn test_record_call_multiple() {
        let metrics = PerformanceMetrics::new();

        for _ in 0..5 {
            metrics.record_call("test_function");
        }

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("test_function"), Some(&5));
    }

    #[test]
    fn test_record_call_different_functions() {
        let metrics = PerformanceMetrics::new();

        metrics.record_call("function_a");
        metrics.record_call("function_b");
        metrics.record_call("function_a");

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("function_a"), Some(&2));
        assert_eq!(report.call_counts.get("function_b"), Some(&1));
    }

    #[test]
    fn test_record_execution_time_single() {
        let metrics = PerformanceMetrics::new();
        let duration = Duration::from_millis(50);

        metrics.record_execution_time("test_function", duration);

        let report = metrics.get_report();
        let times = report.execution_times.get("test_function").unwrap();
        assert_eq!(times.len(), 1);
        assert_eq!(times[0], duration);
    }

    #[test]
    fn test_record_execution_time_multiple() {
        let metrics = PerformanceMetrics::new();

        metrics.record_execution_time("test_function", Duration::from_millis(10));
        metrics.record_execution_time("test_function", Duration::from_millis(20));
        metrics.record_execution_time("test_function", Duration::from_millis(30));

        let report = metrics.get_report();
        let times = report.execution_times.get("test_function").unwrap();
        assert_eq!(times.len(), 3);
        assert_eq!(times[0], Duration::from_millis(10));
        assert_eq!(times[1], Duration::from_millis(20));
        assert_eq!(times[2], Duration::from_millis(30));
    }

    #[test]
    fn test_record_allocation_single() {
        let metrics = PerformanceMetrics::new();
        metrics.record_allocation("test_context");

        let report = metrics.get_report();
        assert_eq!(report.allocation_counts.get("test_context"), Some(&1));
    }

    #[test]
    fn test_record_allocation_multiple() {
        let metrics = PerformanceMetrics::new();

        for _ in 0..10 {
            metrics.record_allocation("test_context");
        }

        let report = metrics.get_report();
        assert_eq!(report.allocation_counts.get("test_context"), Some(&10));
    }

    #[test]
    fn test_record_allocation_different_contexts() {
        let metrics = PerformanceMetrics::new();

        metrics.record_allocation("context_a");
        metrics.record_allocation("context_b");
        metrics.record_allocation("context_a");
        metrics.record_allocation("context_a");

        let report = metrics.get_report();
        assert_eq!(report.allocation_counts.get("context_a"), Some(&3));
        assert_eq!(report.allocation_counts.get("context_b"), Some(&1));
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_zero_requests() {
        let stats = CacheStats {
            hits: 0,
            misses: 0,
            total_requests: 0,
        };
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_all_hits() {
        let stats = CacheStats {
            hits: 100,
            misses: 0,
            total_requests: 100,
        };
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_all_misses() {
        let stats = CacheStats {
            hits: 0,
            misses: 100,
            total_requests: 100,
        };
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_mixed() {
        let stats = CacheStats {
            hits: 75,
            misses: 25,
            total_requests: 100,
        };
        assert_eq!(stats.hit_rate(), 0.75);
    }

    #[test]
    fn test_record_cache_hit() {
        let metrics = PerformanceMetrics::new();
        metrics.record_cache_hit("test_cache");

        let report = metrics.get_report();
        let stats = report.cache_stats.get("test_cache").unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_record_cache_miss() {
        let metrics = PerformanceMetrics::new();
        metrics.record_cache_miss("test_cache");

        let report = metrics.get_report();
        let stats = report.cache_stats.get("test_cache").unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_record_cache_mixed() {
        let metrics = PerformanceMetrics::new();

        metrics.record_cache_hit("test_cache");
        metrics.record_cache_hit("test_cache");
        metrics.record_cache_miss("test_cache");
        metrics.record_cache_hit("test_cache");

        let report = metrics.get_report();
        let stats = report.cache_stats.get("test_cache").unwrap();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests, 4);
        assert_eq!(stats.hit_rate(), 0.75);
    }

    #[test]
    fn test_record_cache_multiple_caches() {
        let metrics = PerformanceMetrics::new();

        metrics.record_cache_hit("cache_a");
        metrics.record_cache_hit("cache_a");
        metrics.record_cache_miss("cache_b");
        metrics.record_cache_hit("cache_b");

        let report = metrics.get_report();

        let stats_a = report.cache_stats.get("cache_a").unwrap();
        assert_eq!(stats_a.hits, 2);
        assert_eq!(stats_a.misses, 0);
        assert_eq!(stats_a.hit_rate(), 1.0);

        let stats_b = report.cache_stats.get("cache_b").unwrap();
        assert_eq!(stats_b.hits, 1);
        assert_eq!(stats_b.misses, 1);
        assert_eq!(stats_b.hit_rate(), 0.5);
    }

    #[test]
    fn test_reset_clears_all_metrics() {
        let metrics = PerformanceMetrics::new();

        // Add some data
        metrics.record_call("test_function");
        metrics.record_execution_time("test_function", Duration::from_millis(50));
        metrics.record_allocation("test_context");
        metrics.record_cache_hit("test_cache");

        // Verify data exists
        let report = metrics.get_report();
        assert!(!report.call_counts.is_empty());
        assert!(!report.execution_times.is_empty());
        assert!(!report.allocation_counts.is_empty());
        assert!(!report.cache_stats.is_empty());

        // Reset and verify empty
        metrics.reset();
        let report = metrics.get_report();
        assert!(report.call_counts.is_empty());
        assert!(report.execution_times.is_empty());
        assert!(report.allocation_counts.is_empty());
        assert!(report.cache_stats.is_empty());
    }

    #[test]
    fn test_performance_metrics_comprehensive() {
        let metrics = PerformanceMetrics::new();

        metrics.record_call("test_function");
        metrics.record_execution_time("test_function", Duration::from_millis(50));
        metrics.record_allocation("test_context");
        metrics.record_cache_hit("test_cache");
        metrics.record_cache_miss("test_cache");

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("test_function"), Some(&1));
        assert_eq!(report.allocation_counts.get("test_context"), Some(&1));

        let cache_stats = report.cache_stats.get("test_cache").unwrap();
        assert_eq!(cache_stats.hits, 1);
        assert_eq!(cache_stats.misses, 1);
        assert_eq!(cache_stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_performance_report_recommendations_no_issues() {
        let metrics = PerformanceMetrics::new();

        // Add reasonable metrics
        metrics.record_call("normal_function");
        metrics.record_execution_time("normal_function", Duration::from_millis(10));
        metrics.record_allocation("normal_context");

        for _ in 0..90 {
            metrics.record_cache_hit("good_cache");
        }
        for _ in 0..10 {
            metrics.record_cache_miss("good_cache");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_performance_report_recommendations_frequent_calls() {
        let metrics = PerformanceMetrics::new();

        // Record more than 10000 calls
        for _ in 0..10001 {
            metrics.record_call("hot_function");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("hot_function") && r.contains("10001"))
        );
    }

    #[test]
    fn test_performance_report_recommendations_slow_function() {
        let metrics = PerformanceMetrics::new();

        // Record slow execution time
        metrics.record_execution_time("slow_function", Duration::from_millis(150));

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("slow_function") && r.contains("slow"))
        );
    }

    #[test]
    fn test_performance_report_recommendations_low_cache_hit_rate() {
        let metrics = PerformanceMetrics::new();

        // Record low cache hit rate with > 100 total requests
        // Need total_requests > 100 for recommendation to trigger
        for _ in 0..21 {
            metrics.record_cache_hit("poor_cache");
        }
        for _ in 0..81 {
            metrics.record_cache_miss("poor_cache");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("poor_cache") && r.contains("hit rate"))
        );
    }

    #[test]
    fn test_performance_report_recommendations_low_cache_hit_rate_insufficient_requests() {
        let metrics = PerformanceMetrics::new();

        // Record low cache hit rate but with too few requests (< 100)
        for _ in 0..10 {
            metrics.record_cache_hit("new_cache");
        }
        for _ in 0..40 {
            metrics.record_cache_miss("new_cache");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        // Should not recommend because total_requests (50) < 100
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_performance_report_recommendations_high_allocations() {
        let metrics = PerformanceMetrics::new();

        // Record more than 50000 allocations
        for _ in 0..50001 {
            metrics.record_allocation("allocation_heavy_context");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("allocation_heavy_context") && r.contains("pooling"))
        );
    }

    #[test]
    fn test_performance_report_recommendations_multiple_issues() {
        let metrics = PerformanceMetrics::new();

        // Add multiple issues
        for _ in 0..10001 {
            metrics.record_call("hot_function");
        }

        metrics.record_execution_time("slow_function", Duration::from_millis(200));

        for _ in 0..50001 {
            metrics.record_allocation("heavy_allocations");
        }

        // Need total_requests > 100 for cache recommendation to trigger
        for _ in 0..21 {
            metrics.record_cache_hit("poor_cache");
        }
        for _ in 0..81 {
            metrics.record_cache_miss("poor_cache");
        }

        let report = metrics.get_report();
        let recommendations = report.get_recommendations();

        // Should have at least 4 recommendations
        assert!(recommendations.len() >= 4);
    }

    #[test]
    fn test_performance_report_print_summary() {
        let metrics = PerformanceMetrics::new();

        // Add various metrics
        metrics.record_call("function_a");
        metrics.record_call("function_b");
        metrics.record_execution_time("function_a", Duration::from_millis(10));
        metrics.record_execution_time("function_a", Duration::from_millis(20));
        metrics.record_allocation("context_a");
        metrics.record_cache_hit("cache_a");
        metrics.record_cache_miss("cache_a");

        let report = metrics.get_report();

        // Just verify it doesn't panic
        report.print_summary();
    }

    #[test]
    fn test_performance_report_print_summary_empty() {
        let metrics = PerformanceMetrics::new();
        let report = metrics.get_report();

        // Should handle empty metrics gracefully
        report.print_summary();
    }

    #[test]
    fn test_performance_timer_basic() {
        let metrics = Arc::new(PerformanceMetrics::new());

        {
            let _timer = PerformanceTimer::start("test_function", metrics.clone());
            // Simulate some work
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("test_function"), Some(&1));

        let times = report.execution_times.get("test_function").unwrap();
        assert_eq!(times.len(), 1);
        assert!(times[0] >= Duration::from_millis(10));
    }

    #[test]
    fn test_performance_timer_multiple() {
        let metrics = Arc::new(PerformanceMetrics::new());

        for _ in 0..3 {
            let _timer = PerformanceTimer::start("test_function", metrics.clone());
            std::thread::sleep(Duration::from_millis(5));
        }

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("test_function"), Some(&3));

        let times = report.execution_times.get("test_function").unwrap();
        assert_eq!(times.len(), 3);
    }

    #[test]
    fn test_performance_timer_slow_warning() {
        let metrics = Arc::new(PerformanceMetrics::new());

        {
            let _timer = PerformanceTimer::start("slow_function", metrics.clone());
            // Sleep for more than 100ms to trigger warning
            std::thread::sleep(Duration::from_millis(105));
        }

        let report = metrics.get_report();
        let times = report.execution_times.get("slow_function").unwrap();
        assert_eq!(times.len(), 1);
        assert!(times[0] >= Duration::from_millis(100));
    }

    #[test]
    fn test_performance_timer_different_functions() {
        let metrics = Arc::new(PerformanceMetrics::new());

        {
            let _timer1 = PerformanceTimer::start("function_a", metrics.clone());
            std::thread::sleep(Duration::from_millis(5));
        }

        {
            let _timer2 = PerformanceTimer::start("function_b", metrics.clone());
            std::thread::sleep(Duration::from_millis(5));
        }

        let report = metrics.get_report();
        assert_eq!(report.call_counts.get("function_a"), Some(&1));
        assert_eq!(report.call_counts.get("function_b"), Some(&1));

        assert!(report.execution_times.contains_key("function_a"));
        assert!(report.execution_times.contains_key("function_b"));
    }

    #[test]
    fn test_global_metrics_singleton() {
        let metrics1 = global_metrics();
        let metrics2 = global_metrics();

        // Should be the same instance
        metrics1.record_call("test");

        let report = metrics2.get_report();
        assert_eq!(report.call_counts.get("test"), Some(&1));
    }

    #[test]
    fn test_global_metrics_shared_state() {
        let metrics = global_metrics();

        // Clear any previous state
        metrics.reset();

        metrics.record_call("shared_function");

        let metrics2 = global_metrics();
        let report = metrics2.get_report();

        assert_eq!(report.call_counts.get("shared_function"), Some(&1));

        // Clean up
        metrics.reset();
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(PerformanceMetrics::new());
        let mut handles = vec![];

        // Spawn multiple threads recording metrics concurrently
        for i in 0..10 {
            let metrics_clone = metrics.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.record_call(&format!("function_{}", i));
                    metrics_clone.record_allocation(&format!("context_{}", i));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let report = metrics.get_report();

        // Each function should have been called 100 times
        for i in 0..10 {
            assert_eq!(
                report.call_counts.get(&format!("function_{}", i)),
                Some(&100)
            );
            assert_eq!(
                report.allocation_counts.get(&format!("context_{}", i)),
                Some(&100)
            );
        }
    }

    #[test]
    fn test_concurrent_cache_access() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(PerformanceMetrics::new());
        let mut handles = vec![];

        // Spawn threads recording cache hits and misses
        for _ in 0..5 {
            let metrics_clone = metrics.clone();
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    metrics_clone.record_cache_hit("shared_cache");
                }
            });
            handles.push(handle);
        }

        for _ in 0..5 {
            let metrics_clone = metrics.clone();
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    metrics_clone.record_cache_miss("shared_cache");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let report = metrics.get_report();
        let stats = report.cache_stats.get("shared_cache").unwrap();

        assert_eq!(stats.hits, 250);
        assert_eq!(stats.misses, 250);
        assert_eq!(stats.total_requests, 500);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_start_performance_monitoring() {
        // Just verify the function can be called without panicking
        // The actual monitoring runs in background
        start_performance_monitoring().await;

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[test]
    fn test_execution_time_average_calculation() {
        let metrics = PerformanceMetrics::new();

        metrics.record_execution_time("avg_function", Duration::from_millis(10));
        metrics.record_execution_time("avg_function", Duration::from_millis(20));
        metrics.record_execution_time("avg_function", Duration::from_millis(30));

        let report = metrics.get_report();
        let times = report.execution_times.get("avg_function").unwrap();

        let sum: Duration = times.iter().sum();
        let avg = sum / times.len() as u32;

        assert_eq!(avg, Duration::from_millis(20));
    }

    #[test]
    fn test_execution_time_max_calculation() {
        let metrics = PerformanceMetrics::new();

        metrics.record_execution_time("max_function", Duration::from_millis(10));
        metrics.record_execution_time("max_function", Duration::from_millis(50));
        metrics.record_execution_time("max_function", Duration::from_millis(30));

        let report = metrics.get_report();
        let times = report.execution_times.get("max_function").unwrap();

        let max = times.iter().max().unwrap();

        assert_eq!(*max, Duration::from_millis(50));
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats {
            hits: 100,
            misses: 50,
            total_requests: 150,
        };

        let cloned = stats.clone();

        assert_eq!(cloned.hits, 100);
        assert_eq!(cloned.misses, 50);
        assert_eq!(cloned.total_requests, 150);
        assert_eq!(cloned.hit_rate(), stats.hit_rate());
    }

    #[test]
    fn test_performance_metrics_clone() {
        let metrics = PerformanceMetrics::new();

        metrics.record_call("test");
        metrics.record_allocation("context");

        let cloned = metrics.clone();

        // Both should share the same underlying data
        let report1 = metrics.get_report();
        let report2 = cloned.get_report();

        assert_eq!(
            report1.call_counts.get("test"),
            report2.call_counts.get("test")
        );
        assert_eq!(
            report1.allocation_counts.get("context"),
            report2.allocation_counts.get("context")
        );
    }
}
