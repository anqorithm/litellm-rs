//! Getter methods for retrieving aggregated metrics

use super::collector::MetricsCollector;
use super::helpers::{
    calculate_average, calculate_average_u32, calculate_average_u64, calculate_percentile,
};
use crate::monitoring::types::{
    ErrorMetrics, LatencyPercentiles, MonitoringRequestMetrics, PerformanceMetrics,
    ProviderMetrics, SystemResourceMetrics,
};
use crate::utils::error::gateway_error::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

impl MetricsCollector {
    /// Get request metrics
    pub async fn get_request_metrics(&self) -> Result<MonitoringRequestMetrics> {
        let storage = self.storage.read();
        let metrics = &storage.request;
        let now = Instant::now();

        // Calculate requests per second (last minute)
        let recent_requests = metrics
            .last_minute_requests
            .iter()
            .filter(|&&time| now.duration_since(time) <= Duration::from_secs(60))
            .count();
        let requests_per_second = recent_requests as f64 / 60.0;

        // Calculate response time percentiles
        // Filter out NaN/Inf values and sort safely
        let mut sorted_times: Vec<f64> = metrics
            .response_times
            .iter()
            .filter(|&&t| t.is_finite())
            .copied()
            .collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg_response_time = if sorted_times.is_empty() {
            0.0
        } else {
            sorted_times.iter().sum::<f64>() / sorted_times.len() as f64
        };

        let p95_response_time = calculate_percentile(&sorted_times, 0.95);
        let p99_response_time = calculate_percentile(&sorted_times, 0.99);

        // Calculate success rate
        let total_requests = metrics.total_requests;
        let error_requests = metrics
            .status_codes
            .iter()
            .filter(|(code, _)| **code >= 400)
            .map(|(_, count)| *count)
            .sum::<u64>();

        let success_rate = if total_requests > 0 {
            ((total_requests - error_requests) as f64 / total_requests as f64) * 100.0
        } else {
            100.0
        };

        Ok(MonitoringRequestMetrics {
            total_requests,
            requests_per_second,
            avg_response_time_ms: avg_response_time,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            success_rate,
            status_codes: metrics.status_codes.clone(),
            endpoints: metrics.endpoints.clone(),
        })
    }

    /// Get provider metrics
    pub async fn get_provider_metrics(&self) -> Result<ProviderMetrics> {
        let storage = self.storage.read();
        let metrics = &storage.provider;

        // Calculate success rates
        let mut provider_success_rates = HashMap::new();
        for (provider, &requests) in &metrics.provider_requests {
            let errors = metrics.provider_errors.get(provider).unwrap_or(&0);
            let success_rate = if requests > 0 {
                ((requests - errors) as f64 / requests as f64) * 100.0
            } else {
                100.0
            };
            provider_success_rates.insert(provider.clone(), success_rate);
        }

        // Calculate average response times
        let mut provider_response_times = HashMap::new();
        for (provider, times) in &metrics.provider_response_times {
            let avg_time = if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<f64>() / times.len() as f64
            };
            provider_response_times.insert(provider.clone(), avg_time);
        }

        Ok(ProviderMetrics {
            total_provider_requests: metrics.total_requests,
            provider_success_rates,
            provider_response_times,
            provider_errors: metrics.provider_errors.clone(),
            provider_usage: metrics.provider_requests.clone(),
            token_usage: metrics.token_usage.clone(),
            costs: metrics.costs.clone(),
        })
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> Result<SystemResourceMetrics> {
        let storage = self.storage.read();
        let metrics = &storage.system;

        // Calculate averages from samples
        let cpu_usage = calculate_average(&metrics.cpu_samples);
        let memory_usage = calculate_average_u64(&metrics.memory_samples);
        let disk_usage = calculate_average_u64(&metrics.disk_samples);
        let network_bytes_in = calculate_average_u64(&metrics.network_in_samples);
        let network_bytes_out = calculate_average_u64(&metrics.network_out_samples);
        let active_connections = calculate_average_u32(&metrics.connection_samples);

        Ok(SystemResourceMetrics {
            cpu_usage,
            memory_usage,
            memory_usage_percent: 0.0, // NOTE: requires total memory to calculate
            disk_usage,
            disk_usage_percent: 0.0, // NOTE: requires total disk to calculate
            network_bytes_in,
            network_bytes_out,
            active_connections,
            database_connections: 0, // NOTE: requires connection pool integration
            redis_connections: 0,    // NOTE: requires Redis pool integration
        })
    }

    /// Get error metrics
    pub async fn get_error_metrics(&self) -> Result<ErrorMetrics> {
        let storage = self.storage.read();
        let metrics = &storage.error;
        let now = Instant::now();

        // Calculate error rate (errors per second in last minute)
        let recent_errors = metrics
            .last_minute_errors
            .iter()
            .filter(|&&time| now.duration_since(time) <= Duration::from_secs(60))
            .count();
        let error_rate = recent_errors as f64 / 60.0;

        Ok(ErrorMetrics {
            total_errors: metrics.total_errors,
            error_rate,
            error_types: metrics.error_types.clone(),
            error_endpoints: metrics.error_endpoints.clone(),
            critical_errors: metrics.critical_errors,
            warnings: metrics.warnings,
        })
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let storage = self.storage.read();
        let metrics = &storage.performance;

        // Calculate cache hit/miss rates
        let total_cache_requests = metrics.cache_hits + metrics.cache_misses;
        let cache_hit_rate = if total_cache_requests > 0 {
            (metrics.cache_hits as f64 / total_cache_requests as f64) * 100.0
        } else {
            0.0
        };
        let cache_miss_rate = 100.0 - cache_hit_rate;

        // Calculate average DB query time
        let avg_db_query_time = calculate_average(&metrics.db_query_times);

        // Calculate throughput
        let throughput = calculate_average(&metrics.throughput_samples);

        // Calculate queue depth
        let queue_depth = calculate_average_u32(&metrics.queue_depths);

        Ok(PerformanceMetrics {
            cache_hit_rate,
            cache_miss_rate,
            avg_db_query_time_ms: avg_db_query_time,
            queue_depth,
            throughput,
            latency_percentiles: LatencyPercentiles {
                p50: 0.0, // NOTE: percentile calculation not yet implemented
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                p999: 0.0,
            },
        })
    }
}
