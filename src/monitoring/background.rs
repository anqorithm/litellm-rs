//! Background task implementations for MonitoringSystem

use crate::utils::error::gateway_error::Result;
use std::time::Duration;

use tracing::{debug, warn};

use super::system::MonitoringSystem;

impl MonitoringSystem {
    /// Start background monitoring tasks
    pub(super) async fn start_background_tasks(&self) -> Result<()> {
        let monitoring = self.clone();

        // Metrics aggregation task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = monitoring.aggregate_metrics().await {
                    warn!("Failed to aggregate metrics: {}", e);
                }
            }
        });

        // Health check task
        let monitoring = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = monitoring.run_health_checks().await {
                    warn!("Health check failed: {}", e);
                }
            }
        });

        // Alert processing task
        if self.alerts.is_some() {
            let monitoring = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    if let Err(e) = monitoring.process_alerts().await {
                        warn!("Failed to process alerts: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Aggregate metrics for storage
    pub(super) async fn aggregate_metrics(&self) -> Result<()> {
        debug!("Aggregating metrics");

        let _metrics = self.get_metrics().await?;

        // Store metrics in database
        // NOTE: SystemMetrics and RequestMetrics type mismatch; conversion needed before storage.
        // self.storage.db().store_metrics(&metrics).await?;

        // Store metrics in time series database (if configured)
        // NOTE: time series storage not yet implemented

        Ok(())
    }

    /// Run health checks
    pub(super) async fn run_health_checks(&self) -> Result<()> {
        debug!("Running health checks");
        // NOTE: health check integration pending monitoring system activation
        Ok(())
    }

    /// Process pending alerts
    pub(super) async fn process_alerts(&self) -> Result<()> {
        if let Some(alerts) = &self.alerts {
            alerts.process_pending().await?;
        }
        Ok(())
    }
}
