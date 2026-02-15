//! Analytics types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRequestMetrics {
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
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ==================== RequestMetrics Tests ====================

    #[test]
    fn test_request_metrics_creation() {
        let now = Utc::now();
        let metrics = AnalyticsRequestMetrics {
            total_requests: 1000,
            successful_requests: 950,
            failed_requests: 50,
            avg_response_time_ms: 150.5,
            p95_response_time_ms: 300.0,
            p99_response_time_ms: 500.0,
            total_tokens: 500000,
            total_cost: 25.50,
            period_start: now,
            period_end: now + chrono::Duration::hours(24),
        };

        assert_eq!(metrics.total_requests, 1000);
        assert_eq!(metrics.successful_requests, 950);
        assert_eq!(metrics.failed_requests, 50);
        assert!((metrics.avg_response_time_ms - 150.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_request_metrics_serialization() {
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let metrics = AnalyticsRequestMetrics {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            avg_response_time_ms: 100.0,
            p95_response_time_ms: 200.0,
            p99_response_time_ms: 350.0,
            total_tokens: 10000,
            total_cost: 5.0,
            period_start: now,
            period_end: now + chrono::Duration::hours(1),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: AnalyticsRequestMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_requests, metrics.total_requests);
        assert_eq!(parsed.successful_requests, metrics.successful_requests);
    }

    #[test]
    fn test_request_metrics_success_rate_calculation() {
        let now = Utc::now();
        let metrics = AnalyticsRequestMetrics {
            total_requests: 1000,
            successful_requests: 950,
            failed_requests: 50,
            avg_response_time_ms: 100.0,
            p95_response_time_ms: 200.0,
            p99_response_time_ms: 300.0,
            total_tokens: 10000,
            total_cost: 5.0,
            period_start: now,
            period_end: now,
        };

        let success_rate = metrics.successful_requests as f64 / metrics.total_requests as f64;
        assert!((success_rate - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_request_metrics_clone() {
        let now = Utc::now();
        let metrics = AnalyticsRequestMetrics {
            total_requests: 500,
            successful_requests: 480,
            failed_requests: 20,
            avg_response_time_ms: 75.0,
            p95_response_time_ms: 150.0,
            p99_response_time_ms: 250.0,
            total_tokens: 25000,
            total_cost: 12.50,
            period_start: now,
            period_end: now,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.total_requests, metrics.total_requests);
        assert_eq!(cloned.total_cost, metrics.total_cost);
    }

    // ==================== ProviderMetrics Tests ====================

    #[test]
    fn test_provider_metrics_creation() {
        let metrics = ProviderMetrics {
            provider_name: "openai".to_string(),
            request_count: 5000,
            success_rate: 0.99,
            avg_latency_ms: 120.5,
            error_rate: 0.01,
            cost_efficiency: 1000.0,
            uptime_percentage: 99.95,
            rate_limit_hits: 15,
        };

        assert_eq!(metrics.provider_name, "openai");
        assert_eq!(metrics.request_count, 5000);
        assert!((metrics.success_rate - 0.99).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_metrics_serialization() {
        let metrics = ProviderMetrics {
            provider_name: "anthropic".to_string(),
            request_count: 3000,
            success_rate: 0.985,
            avg_latency_ms: 200.0,
            error_rate: 0.015,
            cost_efficiency: 800.0,
            uptime_percentage: 99.9,
            rate_limit_hits: 5,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("anthropic"));
        assert!(json.contains("3000"));

        let parsed: ProviderMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider_name, "anthropic");
    }

    #[test]
    fn test_provider_metrics_high_performance() {
        let metrics = ProviderMetrics {
            provider_name: "fast-provider".to_string(),
            request_count: 100000,
            success_rate: 0.999,
            avg_latency_ms: 50.0,
            error_rate: 0.001,
            cost_efficiency: 2000.0,
            uptime_percentage: 99.99,
            rate_limit_hits: 0,
        };

        assert!(metrics.success_rate > 0.99);
        assert!(metrics.avg_latency_ms < 100.0);
        assert_eq!(metrics.rate_limit_hits, 0);
    }

    // ==================== TokenUsage Tests ====================

    #[test]
    fn test_token_usage_creation() {
        let usage = TokenUsage {
            input_tokens: 5000,
            output_tokens: 3000,
            total_tokens: 8000,
            avg_tokens_per_request: 80.0,
        };

        assert_eq!(usage.input_tokens, 5000);
        assert_eq!(usage.output_tokens, 3000);
        assert_eq!(usage.total_tokens, 8000);
    }

    #[test]
    fn test_token_usage_total_matches_sum() {
        let usage = TokenUsage {
            input_tokens: 10000,
            output_tokens: 5000,
            total_tokens: 15000,
            avg_tokens_per_request: 150.0,
        };

        assert_eq!(usage.total_tokens, usage.input_tokens + usage.output_tokens);
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            avg_tokens_per_request: 75.0,
        };

        let json = serde_json::to_string(&usage).unwrap();
        let parsed: TokenUsage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.input_tokens, usage.input_tokens);
        assert_eq!(parsed.total_tokens, usage.total_tokens);
    }

    // ==================== CostBreakdown Tests ====================

    #[test]
    fn test_cost_breakdown_creation() {
        let mut by_provider = HashMap::new();
        by_provider.insert("openai".to_string(), 50.0);
        by_provider.insert("anthropic".to_string(), 30.0);

        let breakdown = CostBreakdown {
            total_cost: 80.0,
            by_provider,
            by_model: HashMap::new(),
            by_operation: HashMap::new(),
            daily_costs: vec![],
        };

        assert_eq!(breakdown.total_cost, 80.0);
        assert_eq!(breakdown.by_provider.len(), 2);
    }

    #[test]
    fn test_cost_breakdown_with_daily_costs() {
        let now = Utc::now();
        let daily_costs = vec![
            DailyCost {
                date: now,
                cost: 10.0,
                requests: 100,
            },
            DailyCost {
                date: now + chrono::Duration::days(1),
                cost: 15.0,
                requests: 150,
            },
        ];

        let breakdown = CostBreakdown {
            total_cost: 25.0,
            by_provider: HashMap::new(),
            by_model: HashMap::new(),
            by_operation: HashMap::new(),
            daily_costs,
        };

        assert_eq!(breakdown.daily_costs.len(), 2);
        let sum: f64 = breakdown.daily_costs.iter().map(|d| d.cost).sum();
        assert!((sum - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_breakdown_serialization() {
        let mut by_model = HashMap::new();
        by_model.insert("gpt-4".to_string(), 40.0);
        by_model.insert("gpt-3.5-turbo".to_string(), 10.0);

        let breakdown = CostBreakdown {
            total_cost: 50.0,
            by_provider: HashMap::new(),
            by_model,
            by_operation: HashMap::new(),
            daily_costs: vec![],
        };

        let json = serde_json::to_string(&breakdown).unwrap();
        assert!(json.contains("gpt-4"));

        let parsed: CostBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_cost, 50.0);
    }

    // ==================== DailyCost Tests ====================

    #[test]
    fn test_daily_cost_creation() {
        let date = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
        let daily = DailyCost {
            date,
            cost: 25.50,
            requests: 500,
        };

        assert_eq!(daily.cost, 25.50);
        assert_eq!(daily.requests, 500);
    }

    #[test]
    fn test_daily_cost_average_cost_per_request() {
        let daily = DailyCost {
            date: Utc::now(),
            cost: 100.0,
            requests: 1000,
        };

        let avg_cost = daily.cost / daily.requests as f64;
        assert!((avg_cost - 0.1).abs() < f64::EPSILON);
    }

    // ==================== ModelUsage Tests ====================

    #[test]
    fn test_model_usage_creation() {
        let usage = ModelUsage {
            model: "gpt-4-turbo".to_string(),
            requests: 1000,
            tokens: 500000,
            cost: 50.0,
            success_rate: 0.995,
        };

        assert_eq!(usage.model, "gpt-4-turbo");
        assert_eq!(usage.requests, 1000);
        assert!(usage.success_rate > 0.99);
    }

    #[test]
    fn test_model_usage_cost_per_token() {
        let usage = ModelUsage {
            model: "claude-3-opus".to_string(),
            requests: 500,
            tokens: 100000,
            cost: 75.0,
            success_rate: 0.99,
        };

        let cost_per_token = usage.cost / usage.tokens as f64;
        assert!(cost_per_token > 0.0);
        assert!((cost_per_token - 0.00075).abs() < 0.0001);
    }

    #[test]
    fn test_model_usage_serialization() {
        let usage = ModelUsage {
            model: "gpt-3.5-turbo".to_string(),
            requests: 5000,
            tokens: 1000000,
            cost: 20.0,
            success_rate: 0.998,
        };

        let json = serde_json::to_string(&usage).unwrap();
        let parsed: ModelUsage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.model, "gpt-3.5-turbo");
        assert_eq!(parsed.tokens, 1000000);
    }

    // ==================== UsagePatterns Tests ====================

    #[test]
    fn test_usage_patterns_creation() {
        let mut usage_by_weekday = HashMap::new();
        usage_by_weekday.insert("Monday".to_string(), 1000);
        usage_by_weekday.insert("Tuesday".to_string(), 1200);

        let patterns = UsagePatterns {
            peak_hours: vec![9, 10, 11, 14, 15, 16],
            usage_by_weekday,
            request_size_distribution: RequestSizeDistribution {
                small: 500,
                medium: 300,
                large: 150,
                extra_large: 50,
            },
            seasonal_trends: vec![],
        };

        assert_eq!(patterns.peak_hours.len(), 6);
        assert!(patterns.peak_hours.contains(&9));
    }

    #[test]
    fn test_usage_patterns_serialization() {
        let patterns = UsagePatterns {
            peak_hours: vec![10, 11, 12],
            usage_by_weekday: HashMap::new(),
            request_size_distribution: RequestSizeDistribution {
                small: 100,
                medium: 200,
                large: 50,
                extra_large: 10,
            },
            seasonal_trends: vec![],
        };

        let json = serde_json::to_string(&patterns).unwrap();
        let parsed: UsagePatterns = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.peak_hours.len(), 3);
    }

    // ==================== RequestSizeDistribution Tests ====================

    #[test]
    fn test_request_size_distribution_creation() {
        let dist = RequestSizeDistribution {
            small: 5000,
            medium: 3000,
            large: 1500,
            extra_large: 500,
        };

        assert_eq!(dist.small, 5000);
        assert_eq!(dist.medium, 3000);
        assert_eq!(dist.large, 1500);
        assert_eq!(dist.extra_large, 500);
    }

    #[test]
    fn test_request_size_distribution_total() {
        let dist = RequestSizeDistribution {
            small: 1000,
            medium: 500,
            large: 300,
            extra_large: 200,
        };

        let total = dist.small + dist.medium + dist.large + dist.extra_large;
        assert_eq!(total, 2000);
    }

    #[test]
    fn test_request_size_distribution_percentages() {
        let dist = RequestSizeDistribution {
            small: 500,
            medium: 300,
            large: 150,
            extra_large: 50,
        };

        let total = (dist.small + dist.medium + dist.large + dist.extra_large) as f64;
        let small_pct = dist.small as f64 / total;
        let extra_large_pct = dist.extra_large as f64 / total;

        assert!((small_pct - 0.5).abs() < f64::EPSILON);
        assert!((extra_large_pct - 0.05).abs() < f64::EPSILON);
    }

    // ==================== SeasonalTrend Tests ====================

    #[test]
    fn test_seasonal_trend_creation() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap();

        let trend = SeasonalTrend {
            period: "Q1 2024".to_string(),
            start_date: start,
            end_date: end,
            usage: 100000,
            growth_rate: 15.5,
        };

        assert_eq!(trend.period, "Q1 2024");
        assert_eq!(trend.usage, 100000);
        assert!((trend.growth_rate - 15.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_seasonal_trend_negative_growth() {
        let trend = SeasonalTrend {
            period: "Month".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            usage: 8000,
            growth_rate: -10.0,
        };

        assert!(trend.growth_rate < 0.0);
    }

    #[test]
    fn test_seasonal_trend_serialization() {
        let trend = SeasonalTrend {
            period: "Week".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            usage: 5000,
            growth_rate: 5.0,
        };

        let json = serde_json::to_string(&trend).unwrap();
        assert!(json.contains("Week"));

        let parsed: SeasonalTrend = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.usage, 5000);
    }

    // ==================== CostMetrics Tests ====================

    #[test]
    fn test_cost_metrics_creation() {
        let mut cost_by_period = HashMap::new();
        cost_by_period.insert("2024-01".to_string(), 100.0);
        cost_by_period.insert("2024-02".to_string(), 120.0);

        let metrics = CostMetrics {
            total_cost: 220.0,
            cost_by_period,
            cost_trends: vec![],
            budget_utilization: HashMap::new(),
        };

        assert_eq!(metrics.total_cost, 220.0);
        assert_eq!(metrics.cost_by_period.len(), 2);
    }

    #[test]
    fn test_cost_metrics_with_trends() {
        let now = Utc::now();
        let trends = vec![
            CostTrend {
                period: now,
                cost: 100.0,
                change_percentage: 0.0,
                projected_cost: 100.0,
            },
            CostTrend {
                period: now + chrono::Duration::days(30),
                cost: 110.0,
                change_percentage: 10.0,
                projected_cost: 120.0,
            },
        ];

        let metrics = CostMetrics {
            total_cost: 210.0,
            cost_by_period: HashMap::new(),
            cost_trends: trends,
            budget_utilization: HashMap::new(),
        };

        assert_eq!(metrics.cost_trends.len(), 2);
        assert!((metrics.cost_trends[1].change_percentage - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_metrics_serialization() {
        let metrics = CostMetrics {
            total_cost: 500.0,
            cost_by_period: HashMap::new(),
            cost_trends: vec![],
            budget_utilization: HashMap::new(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: CostMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_cost, 500.0);
    }

    // ==================== CostTrend Tests ====================

    #[test]
    fn test_cost_trend_creation() {
        let trend = CostTrend {
            period: Utc::now(),
            cost: 150.0,
            change_percentage: 5.5,
            projected_cost: 160.0,
        };

        assert_eq!(trend.cost, 150.0);
        assert!((trend.change_percentage - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_trend_decrease() {
        let trend = CostTrend {
            period: Utc::now(),
            cost: 90.0,
            change_percentage: -10.0,
            projected_cost: 85.0,
        };

        assert!(trend.change_percentage < 0.0);
        assert!(trend.projected_cost < trend.cost);
    }

    #[test]
    fn test_cost_trend_serialization() {
        let trend = CostTrend {
            period: Utc::now(),
            cost: 200.0,
            change_percentage: 15.0,
            projected_cost: 230.0,
        };

        let json = serde_json::to_string(&trend).unwrap();
        let parsed: CostTrend = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.cost, 200.0);
    }

    // ==================== BudgetUtilization Tests ====================

    #[test]
    fn test_budget_utilization_creation() {
        let util = BudgetUtilization {
            budget_limit: 1000.0,
            current_usage: 750.0,
            utilization_percentage: 75.0,
            projected_usage: 900.0,
            days_remaining: 10,
        };

        assert_eq!(util.budget_limit, 1000.0);
        assert_eq!(util.current_usage, 750.0);
        assert!((util.utilization_percentage - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_utilization_under_budget() {
        let util = BudgetUtilization {
            budget_limit: 500.0,
            current_usage: 200.0,
            utilization_percentage: 40.0,
            projected_usage: 350.0,
            days_remaining: 15,
        };

        assert!(util.current_usage < util.budget_limit);
        assert!(util.projected_usage < util.budget_limit);
        assert!(util.utilization_percentage < 100.0);
    }

    #[test]
    fn test_budget_utilization_over_budget_projected() {
        let util = BudgetUtilization {
            budget_limit: 500.0,
            current_usage: 450.0,
            utilization_percentage: 90.0,
            projected_usage: 600.0,
            days_remaining: 5,
        };

        assert!(util.projected_usage > util.budget_limit);
    }

    #[test]
    fn test_budget_utilization_serialization() {
        let util = BudgetUtilization {
            budget_limit: 1000.0,
            current_usage: 500.0,
            utilization_percentage: 50.0,
            projected_usage: 750.0,
            days_remaining: 15,
        };

        let json = serde_json::to_string(&util).unwrap();
        let parsed: BudgetUtilization = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.budget_limit, 1000.0);
        assert_eq!(parsed.days_remaining, 15);
    }

    // ==================== UserMetrics Tests ====================

    #[test]
    fn test_user_metrics_creation() {
        let metrics = UserMetrics {
            user_id: "user_123".to_string(),
            request_count: 1000,
            token_usage: TokenUsage {
                input_tokens: 50000,
                output_tokens: 25000,
                total_tokens: 75000,
                avg_tokens_per_request: 75.0,
            },
            cost_breakdown: CostBreakdown {
                total_cost: 50.0,
                by_provider: HashMap::new(),
                by_model: HashMap::new(),
                by_operation: HashMap::new(),
                daily_costs: vec![],
            },
            top_models: vec![],
            usage_patterns: UsagePatterns {
                peak_hours: vec![10, 11],
                usage_by_weekday: HashMap::new(),
                request_size_distribution: RequestSizeDistribution {
                    small: 100,
                    medium: 50,
                    large: 20,
                    extra_large: 5,
                },
                seasonal_trends: vec![],
            },
        };

        assert_eq!(metrics.user_id, "user_123");
        assert_eq!(metrics.request_count, 1000);
    }

    #[test]
    fn test_user_metrics_with_top_models() {
        let top_models = vec![
            ModelUsage {
                model: "gpt-4".to_string(),
                requests: 500,
                tokens: 200000,
                cost: 30.0,
                success_rate: 0.99,
            },
            ModelUsage {
                model: "gpt-3.5-turbo".to_string(),
                requests: 500,
                tokens: 100000,
                cost: 5.0,
                success_rate: 0.995,
            },
        ];

        let metrics = UserMetrics {
            user_id: "power_user".to_string(),
            request_count: 1000,
            token_usage: TokenUsage {
                input_tokens: 200000,
                output_tokens: 100000,
                total_tokens: 300000,
                avg_tokens_per_request: 300.0,
            },
            cost_breakdown: CostBreakdown {
                total_cost: 35.0,
                by_provider: HashMap::new(),
                by_model: HashMap::new(),
                by_operation: HashMap::new(),
                daily_costs: vec![],
            },
            top_models,
            usage_patterns: UsagePatterns {
                peak_hours: vec![],
                usage_by_weekday: HashMap::new(),
                request_size_distribution: RequestSizeDistribution {
                    small: 0,
                    medium: 0,
                    large: 0,
                    extra_large: 0,
                },
                seasonal_trends: vec![],
            },
        };

        assert_eq!(metrics.top_models.len(), 2);
        assert_eq!(metrics.top_models[0].model, "gpt-4");
    }

    #[test]
    fn test_user_metrics_serialization() {
        let metrics = UserMetrics {
            user_id: "test_user".to_string(),
            request_count: 100,
            token_usage: TokenUsage {
                input_tokens: 1000,
                output_tokens: 500,
                total_tokens: 1500,
                avg_tokens_per_request: 15.0,
            },
            cost_breakdown: CostBreakdown {
                total_cost: 1.0,
                by_provider: HashMap::new(),
                by_model: HashMap::new(),
                by_operation: HashMap::new(),
                daily_costs: vec![],
            },
            top_models: vec![],
            usage_patterns: UsagePatterns {
                peak_hours: vec![],
                usage_by_weekday: HashMap::new(),
                request_size_distribution: RequestSizeDistribution {
                    small: 0,
                    medium: 0,
                    large: 0,
                    extra_large: 0,
                },
                seasonal_trends: vec![],
            },
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("test_user"));

        let parsed: UserMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.user_id, "test_user");
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_analytics_workflow() {
        // Create a complete analytics snapshot
        let now = Utc::now();

        let request_metrics = AnalyticsRequestMetrics {
            total_requests: 10000,
            successful_requests: 9900,
            failed_requests: 100,
            avg_response_time_ms: 150.0,
            p95_response_time_ms: 300.0,
            p99_response_time_ms: 500.0,
            total_tokens: 5000000,
            total_cost: 250.0,
            period_start: now - chrono::Duration::days(7),
            period_end: now,
        };

        let provider_metrics = [
            ProviderMetrics {
                provider_name: "openai".to_string(),
                request_count: 6000,
                success_rate: 0.995,
                avg_latency_ms: 140.0,
                error_rate: 0.005,
                cost_efficiency: 900.0,
                uptime_percentage: 99.9,
                rate_limit_hits: 5,
            },
            ProviderMetrics {
                provider_name: "anthropic".to_string(),
                request_count: 4000,
                success_rate: 0.99,
                avg_latency_ms: 165.0,
                error_rate: 0.01,
                cost_efficiency: 850.0,
                uptime_percentage: 99.8,
                rate_limit_hits: 2,
            },
        ];

        // Verify aggregations
        let total_provider_requests: u64 = provider_metrics.iter().map(|p| p.request_count).sum();
        assert_eq!(total_provider_requests, request_metrics.total_requests);

        // Calculate weighted average latency
        let weighted_latency: f64 = provider_metrics
            .iter()
            .map(|p| p.avg_latency_ms * p.request_count as f64)
            .sum::<f64>()
            / total_provider_requests as f64;
        assert!(weighted_latency > 140.0 && weighted_latency < 165.0);
    }

    #[test]
    fn test_cost_analysis_workflow() {
        let mut by_provider = HashMap::new();
        by_provider.insert("openai".to_string(), 150.0);
        by_provider.insert("anthropic".to_string(), 100.0);

        let mut by_model = HashMap::new();
        by_model.insert("gpt-4".to_string(), 100.0);
        by_model.insert("gpt-3.5-turbo".to_string(), 50.0);
        by_model.insert("claude-3-opus".to_string(), 75.0);
        by_model.insert("claude-3-sonnet".to_string(), 25.0);

        let cost_breakdown = CostBreakdown {
            total_cost: 250.0,
            by_provider: by_provider.clone(),
            by_model: by_model.clone(),
            by_operation: HashMap::new(),
            daily_costs: vec![],
        };

        // Verify provider totals match
        let provider_sum: f64 = by_provider.values().sum();
        assert!((provider_sum - cost_breakdown.total_cost).abs() < f64::EPSILON);

        // Verify model totals match
        let model_sum: f64 = by_model.values().sum();
        assert!((model_sum - cost_breakdown.total_cost).abs() < f64::EPSILON);
    }
}
