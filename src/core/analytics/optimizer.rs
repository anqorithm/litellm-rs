//! Cost optimization system

use super::types::UserMetrics;
use crate::utils::error::gateway_error::Result;
use serde::{Deserialize, Serialize};

/// Cost optimization suggestions
pub struct CostOptimizer {
    /// Optimization rules
    optimization_rules: Vec<OptimizationRule>,
}

/// Optimization rule
#[derive(Debug, Clone)]
pub struct OptimizationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Potential savings
    pub potential_savings: f64,
    /// Implementation difficulty
    pub difficulty: OptimizationDifficulty,
    /// Rule type
    pub rule_type: OptimizationType,
}

/// Optimization difficulty levels
#[derive(Debug, Clone)]
pub enum OptimizationDifficulty {
    /// Easy to implement optimization
    Easy,
    /// Medium difficulty optimization
    Medium,
    /// Hard to implement optimization
    Hard,
}

/// Types of optimizations
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Switch to cheaper provider
    ProviderSwitch,
    /// Use smaller model
    ModelDowngrade,
    /// Implement caching
    Caching,
    /// Batch requests
    Batching,
    /// Optimize prompts
    PromptOptimization,
    /// Use different pricing tier
    PricingTier,
}

/// Cost optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Suggestion title
    pub title: String,
    /// Description
    pub description: String,
    /// Potential monthly savings
    pub potential_savings: f64,
    /// Implementation effort
    pub effort: String,
    /// Priority level
    pub priority: u8,
    /// Specific recommendations
    pub recommendations: Vec<String>,
}

impl Default for CostOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CostOptimizer {
    /// Create a new cost optimizer
    pub fn new() -> Self {
        Self {
            optimization_rules: Self::default_rules(),
        }
    }

    /// Analyze metrics and generate suggestions
    pub async fn analyze_and_suggest(
        &self,
        metrics: &UserMetrics,
    ) -> Result<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();

        // Analyze cost patterns and generate suggestions
        if metrics.cost_breakdown.total_cost > 100.0 {
            suggestions.push(OptimizationSuggestion {
                title: "Consider Model Optimization".to_string(),
                description:
                    "Your usage patterns suggest potential savings through model optimization"
                        .to_string(),
                potential_savings: metrics.cost_breakdown.total_cost * 0.2,
                effort: "Medium".to_string(),
                priority: 8,
                recommendations: vec![
                    "Evaluate if smaller models can meet your needs".to_string(),
                    "Implement request caching for repeated queries".to_string(),
                ],
            });
        }

        Ok(suggestions)
    }

    /// Get optimization rules
    pub fn rules(&self) -> &[OptimizationRule] {
        &self.optimization_rules
    }

    /// Default optimization rules
    fn default_rules() -> Vec<OptimizationRule> {
        vec![
            OptimizationRule {
                name: "Provider Cost Comparison".to_string(),
                description: "Compare costs across different providers".to_string(),
                potential_savings: 0.3,
                difficulty: OptimizationDifficulty::Easy,
                rule_type: OptimizationType::ProviderSwitch,
            },
            OptimizationRule {
                name: "Model Right-sizing".to_string(),
                description: "Use appropriately sized models for tasks".to_string(),
                potential_savings: 0.4,
                difficulty: OptimizationDifficulty::Medium,
                rule_type: OptimizationType::ModelDowngrade,
            },
        ]
    }
}
