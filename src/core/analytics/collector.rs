//! Metrics collector for analytics

use super::types::{
    CostBreakdown, CostMetrics, ProviderMetrics, RequestMetrics, RequestSizeDistribution,
    TokenUsage, UsagePatterns, UserMetrics,
};
use crate::utils::error::error::Result;
use std::collections::HashMap;

/// Metrics collector for gathering usage statistics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Request metrics
    request_metrics: HashMap<String, RequestMetrics>,
    /// Provider metrics
    provider_metrics: HashMap<String, ProviderMetrics>,
    /// User metrics
    user_metrics: HashMap<String, UserMetrics>,
    /// Cost metrics
    cost_metrics: CostMetrics,
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
            request_metrics: HashMap::new(),
            provider_metrics: HashMap::new(),
            user_metrics: HashMap::new(),
            cost_metrics: CostMetrics {
                total_cost: 0.0,
                cost_by_period: HashMap::new(),
                cost_trends: Vec::new(),
                budget_utilization: HashMap::new(),
            },
        }
    }

    /// Process user data and generate metrics
    pub async fn process_user_data(
        &self,
        user_id: &str,
        usage_data: &[serde_json::Value],
    ) -> Result<UserMetrics> {
        // Process the usage data and calculate metrics
        let request_count = usage_data.len() as u64;
        let total_tokens = usage_data
            .iter()
            .filter_map(|data| data.get("total_tokens")?.as_u64())
            .sum();

        let total_cost = usage_data
            .iter()
            .filter_map(|data| data.get("cost")?.as_f64())
            .sum();

        Ok(UserMetrics {
            user_id: user_id.to_string(),
            request_count,
            token_usage: TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens,
                avg_tokens_per_request: if request_count > 0 {
                    total_tokens as f64 / request_count as f64
                } else {
                    0.0
                },
            },
            cost_breakdown: CostBreakdown {
                total_cost,
                by_provider: HashMap::new(),
                by_model: HashMap::new(),
                by_operation: HashMap::new(),
                daily_costs: Vec::new(),
            },
            top_models: Vec::new(),
            usage_patterns: UsagePatterns {
                peak_hours: Vec::new(),
                usage_by_weekday: HashMap::new(),
                request_size_distribution: RequestSizeDistribution {
                    small: 0,
                    medium: 0,
                    large: 0,
                    extra_large: 0,
                },
                seasonal_trends: Vec::new(),
            },
        })
    }

    /// Get request metrics
    pub fn request_metrics(&self) -> &HashMap<String, RequestMetrics> {
        &self.request_metrics
    }

    /// Get provider metrics
    pub fn provider_metrics(&self) -> &HashMap<String, ProviderMetrics> {
        &self.provider_metrics
    }

    /// Get user metrics
    pub fn user_metrics(&self) -> &HashMap<String, UserMetrics> {
        &self.user_metrics
    }

    /// Get cost metrics
    pub fn cost_metrics(&self) -> &CostMetrics {
        &self.cost_metrics
    }
}
