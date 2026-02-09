//! Analytics types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// P95 response time
    pub p95_response_time_ms: f64,
    /// P99 response time
    pub p99_response_time_ms: f64,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Time period
    pub period_start: DateTime<Utc>,
    /// End of analysis period
    pub period_end: DateTime<Utc>,
}

/// Provider-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Provider name
    pub provider_name: String,
    /// Request count
    pub request_count: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// Error rate
    pub error_rate: f64,
    /// Cost efficiency (tokens per dollar)
    pub cost_efficiency: f64,
    /// Uptime percentage
    pub uptime_percentage: f64,
    /// Rate limit hits
    pub rate_limit_hits: u64,
}

/// User-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetrics {
    /// User ID
    pub user_id: String,
    /// Request count
    pub request_count: u64,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Cost breakdown
    pub cost_breakdown: CostBreakdown,
    /// Most used models
    pub top_models: Vec<ModelUsage>,
    /// Usage patterns
    pub usage_patterns: UsagePatterns,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
}

/// Cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Total cost
    pub total_cost: f64,
    /// Cost by provider
    pub by_provider: HashMap<String, f64>,
    /// Cost by model
    pub by_model: HashMap<String, f64>,
    /// Cost by operation type
    pub by_operation: HashMap<String, f64>,
    /// Daily costs
    pub daily_costs: Vec<DailyCost>,
}

/// Daily cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    /// Date
    pub date: DateTime<Utc>,
    /// Cost amount
    pub cost: f64,
    /// Request count
    pub requests: u64,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model name
    pub model: String,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Success rate
    pub success_rate: f64,
}

/// Usage patterns analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    /// Peak usage hours
    pub peak_hours: Vec<u8>,
    /// Usage by day of week
    pub usage_by_weekday: HashMap<String, u64>,
    /// Request size distribution
    pub request_size_distribution: RequestSizeDistribution,
    /// Seasonal trends
    pub seasonal_trends: Vec<SeasonalTrend>,
}

/// Request size distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSizeDistribution {
    /// Small requests (< 100 tokens)
    pub small: u64,
    /// Medium requests (100-1000 tokens)
    pub medium: u64,
    /// Large requests (1000-10000 tokens)
    pub large: u64,
    /// Extra large requests (> 10000 tokens)
    pub extra_large: u64,
}

/// Seasonal trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalTrend {
    /// Period (week, month, quarter)
    pub period: String,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// End date
    pub end_date: DateTime<Utc>,
    /// Usage count
    pub usage: u64,
    /// Growth rate compared to previous period
    pub growth_rate: f64,
}

/// Overall cost metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Total cost across all users
    pub total_cost: f64,
    /// Cost by time period
    pub cost_by_period: HashMap<String, f64>,
    /// Cost trends
    pub cost_trends: Vec<CostTrend>,
    /// Budget utilization
    pub budget_utilization: HashMap<String, BudgetUtilization>,
}

/// Cost trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    /// Period
    pub period: DateTime<Utc>,
    /// Cost amount
    pub cost: f64,
    /// Change from previous period
    pub change_percentage: f64,
    /// Projected cost for next period
    pub projected_cost: f64,
}

/// Budget utilization tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUtilization {
    /// Budget limit
    pub budget_limit: f64,
    /// Current usage
    pub current_usage: f64,
    /// Utilization percentage
    pub utilization_percentage: f64,
    /// Projected end-of-period usage
    pub projected_usage: f64,
    /// Days remaining in period
    pub days_remaining: u32,
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
