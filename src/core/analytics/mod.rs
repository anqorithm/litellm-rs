//! Advanced analytics and reporting system
//!
//! This module provides comprehensive analytics, cost optimization suggestions,
//! and detailed reporting capabilities.

mod collector;
mod engine;
mod optimizer;
mod reports;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use collector::MetricsCollector;
pub use engine::AnalyticsEngine;
pub use optimizer::{
    CostOptimizer, OptimizationDifficulty, OptimizationRule, OptimizationSuggestion,
    OptimizationType,
};
pub use reports::{
    ChartData, DataPoint, GeneratedReport, ReportFormat, ReportGenerator, ReportSection,
    ReportSectionData, ReportSectionType, ReportSummary, ReportTemplate,
};
pub use types::{
    AnalyticsRequestMetrics, BudgetUtilization, CostBreakdown, CostMetrics, CostTrend, DailyCost,
    ModelUsage, ProviderMetrics, RequestSizeDistribution, SeasonalTrend, TokenUsage, UsagePatterns,
    UserMetrics,
};
