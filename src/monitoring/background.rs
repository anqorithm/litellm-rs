//! Background task implementations for MonitoringSystem

use crate::utils::error::gateway_error::Result;
use std::time::Duration;

use tracing::{debug, warn};

use super::system::MonitoringSystem;
use super::types::{Alert, AlertSeverity};

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
        // TODO: SystemMetrics and RequestMetrics are different types, need to convert or use different method
        // self.storage.db().store_metrics(&metrics).await?;

        // Store metrics in time series database (if configured)
        // TODO: Implement time series storage

        Ok(())
    }

    /// Run health checks
    pub(super) async fn run_health_checks(&self) -> Result<()> {
        debug!("Running health checks");

        let health_status = self.health.check_all().await?;

        // Check for unhealthy components and send alerts
        if !health_status.overall_healthy {
            let alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Critical,
                title: "System Health Check Failed".to_string(),
                description: format!(
                    "One or more system components are unhealthy: {:?}",
                    health_status
                ),
                timestamp: chrono::Utc::now(),
                source: "health_checker".to_string(),
                metadata: serde_json::to_value(&health_status).unwrap_or_default(),
                resolved: false,
            };

            self.send_alert(alert).await?;
        }

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
