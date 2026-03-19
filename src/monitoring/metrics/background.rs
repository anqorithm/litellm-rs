//! Background tasks for metrics collection

use super::bounded::BoundedPush;
use super::collector::MetricsCollector;
use super::system::{
    get_active_connections, get_cpu_usage, get_disk_usage, get_memory_usage, get_network_bytes_in,
    get_network_bytes_out,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// Start system metrics collection
pub(super) async fn start_system_metrics_collection(collector: &MetricsCollector) {
    let storage = collector.storage.clone();
    let active = collector.active.load(Ordering::Acquire);

    if !active {
        return;
    }

    // Clone Arc for the spawned task
    let storage_clone = storage.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            // Collect system metrics
            {
                let mut storage = storage_clone.write();
                let metrics = &mut storage.system;

                // NOTE: actual system metrics collection not yet implemented
                // For now, use placeholder values
                // Using push_bounded for automatic size limiting (1 hour at 10-second intervals = 360 samples)
                const SYSTEM_MAX_SAMPLES: usize = 360;
                metrics
                    .cpu_samples
                    .push_bounded(get_cpu_usage(), SYSTEM_MAX_SAMPLES);
                metrics
                    .memory_samples
                    .push_bounded(get_memory_usage(), SYSTEM_MAX_SAMPLES);
                metrics
                    .disk_samples
                    .push_bounded(get_disk_usage(), SYSTEM_MAX_SAMPLES);
                metrics
                    .network_in_samples
                    .push_bounded(get_network_bytes_in(), SYSTEM_MAX_SAMPLES);
                metrics
                    .network_out_samples
                    .push_bounded(get_network_bytes_out(), SYSTEM_MAX_SAMPLES);
                metrics
                    .connection_samples
                    .push_bounded(get_active_connections(), SYSTEM_MAX_SAMPLES);
            }
        }
    });
}

/// Start cleanup task for old metrics
pub(super) async fn start_cleanup_task(collector: &MetricsCollector) {
    let storage = collector.storage.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            let now = Instant::now();

            // Clean up old timestamps in a single lock
            {
                let mut storage = storage.write();

                // Clean up old request timestamps
                storage
                    .request
                    .last_minute_requests
                    .retain(|&time| now.duration_since(time) <= Duration::from_secs(300));

                // Clean up old error timestamps
                storage
                    .error
                    .last_minute_errors
                    .retain(|&time| now.duration_since(time) <= Duration::from_secs(300));
            }
        }
    });
}
