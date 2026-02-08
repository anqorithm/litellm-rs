//! Analytics engine for processing usage data

use super::collector::MetricsCollector;
use super::optimizer::{CostOptimizer, OptimizationSuggestion};
use super::reports::{GeneratedReport, ReportGenerator};
use super::types::UserMetrics;
use crate::storage::database::Database;
use crate::utils::error::error::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tracing::info;

/// Analytics engine for processing usage data and generating insights
pub struct AnalyticsEngine {
    /// Database connection
    database: Arc<Database>,
    /// Metrics collector
    metrics_collector: MetricsCollector,
    /// Cost optimizer
    cost_optimizer: CostOptimizer,
    /// Report generator
    report_generator: ReportGenerator,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            metrics_collector: MetricsCollector::new(),
            cost_optimizer: CostOptimizer::new(),
            report_generator: ReportGenerator::new(),
        }
    }

    /// Generate usage analytics for a user
    pub async fn generate_user_analytics(
        &self,
        user_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<UserMetrics> {
        info!("Generating analytics for user: {}", user_id);

        // Fetch usage data from database
        let usage_data = self
            .database
            .get_user_usage(user_id, start_date, end_date)
            .await?;

        // Process and analyze the data
        let metrics = self
            .metrics_collector
            .process_user_data(user_id, &usage_data)
            .await?;

        Ok(metrics)
    }

    /// Generate cost optimization suggestions
    pub async fn generate_cost_suggestions(
        &self,
        user_id: &str,
        period_days: u32,
    ) -> Result<Vec<OptimizationSuggestion>> {
        info!(
            "Generating cost optimization suggestions for user: {}",
            user_id
        );

        let end_date = Utc::now();
        let start_date = end_date - Duration::days(period_days as i64);

        // Get user metrics
        let metrics = self
            .generate_user_analytics(user_id, start_date, end_date)
            .await?;

        // Generate suggestions
        let suggestions = self.cost_optimizer.analyze_and_suggest(&metrics).await?;

        Ok(suggestions)
    }

    /// Generate comprehensive report
    pub async fn generate_report(
        &self,
        template_name: &str,
        user_id: Option<&str>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<GeneratedReport> {
        info!(
            "Generating report: {} for period {} to {}",
            template_name, start_date, end_date
        );

        let report = self
            .report_generator
            .generate(template_name, user_id, start_date, end_date, &self.database)
            .await?;

        Ok(report)
    }

    /// Get metrics collector reference
    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// Get cost optimizer reference
    pub fn cost_optimizer(&self) -> &CostOptimizer {
        &self.cost_optimizer
    }

    /// Get report generator reference
    pub fn report_generator(&self) -> &ReportGenerator {
        &self.report_generator
    }
}
