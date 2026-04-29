//! Budget configuration models
//!
//! Configuration structures for per-provider and per-model budget limits.

use serde::{Deserialize, Serialize};

/// Budget configuration for the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfiguration {
    /// Whether budget management is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Provider-specific budget limits
    #[serde(default)]
    pub providers: Vec<ProviderBudgetConfig>,

    /// Model-specific budget limits
    #[serde(default)]
    pub models: Vec<ModelBudgetConfig>,

    /// Global budget settings
    #[serde(default)]
    pub global: GlobalBudgetSettings,
}

fn default_enabled() -> bool {
    true
}

impl Default for BudgetConfiguration {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            providers: Vec::new(),
            models: Vec::new(),
            global: GlobalBudgetSettings::default(),
        }
    }
}

/// Global budget settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBudgetSettings {
    /// Default soft limit percentage (0.0 to 1.0)
    #[serde(default = "default_soft_limit_percentage")]
    pub default_soft_limit_percentage: f64,

    /// Whether to block requests when budget is exceeded
    #[serde(default = "default_block_on_exceeded")]
    pub block_on_exceeded: bool,

    /// Whether to log warnings when approaching budget limits
    #[serde(default = "default_log_warnings")]
    pub log_warnings: bool,

    /// Auto-reset check interval in seconds
    #[serde(default = "default_reset_check_interval")]
    pub reset_check_interval_secs: u64,
}

impl Default for GlobalBudgetSettings {
    fn default() -> Self {
        Self {
            default_soft_limit_percentage: default_soft_limit_percentage(),
            block_on_exceeded: default_block_on_exceeded(),
            log_warnings: default_log_warnings(),
            reset_check_interval_secs: default_reset_check_interval(),
        }
    }
}

fn default_soft_limit_percentage() -> f64 {
    0.8
}

fn default_block_on_exceeded() -> bool {
    true
}

fn default_log_warnings() -> bool {
    true
}

fn default_reset_check_interval() -> u64 {
    60 // 1 minute
}

/// Provider budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderBudgetConfig {
    /// Provider name (e.g., "openai", "anthropic")
    pub provider: String,

    /// Maximum budget amount
    pub max_budget: f64,

    /// Reset period
    #[serde(default = "default_reset_period")]
    pub reset_period: ResetPeriodConfig,

    /// Soft limit percentage (0.0 to 1.0)
    #[serde(default = "default_soft_limit_percentage")]
    pub soft_limit_percentage: f64,

    /// Currency (default: USD)
    #[serde(default = "default_currency")]
    pub currency: String,

    /// Whether this budget is enabled
    #[serde(default = "default_budget_enabled")]
    pub enabled: bool,
}

/// Model budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBudgetConfig {
    /// Model name (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// Maximum budget amount
    pub max_budget: f64,

    /// Reset period
    #[serde(default = "default_reset_period")]
    pub reset_period: ResetPeriodConfig,

    /// Soft limit percentage (0.0 to 1.0)
    #[serde(default = "default_soft_limit_percentage")]
    pub soft_limit_percentage: f64,

    /// Currency (default: USD)
    #[serde(default = "default_currency")]
    pub currency: String,

    /// Whether this budget is enabled
    #[serde(default = "default_budget_enabled")]
    pub enabled: bool,
}

/// Reset period configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResetPeriodConfig {
    /// Never reset
    Never,
    /// Reset daily at midnight UTC
    Daily,
    /// Reset weekly on Sunday at midnight UTC
    Weekly,
    /// Reset monthly on the 1st at midnight UTC
    #[default]
    Monthly,
}

fn default_reset_period() -> ResetPeriodConfig {
    ResetPeriodConfig::Monthly
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_budget_enabled() -> bool {
    true
}

impl BudgetConfiguration {
    /// Check if budget management is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get provider budget configs
    pub fn provider_budgets(&self) -> &[ProviderBudgetConfig] {
        &self.providers
    }

    /// Get model budget configs
    pub fn model_budgets(&self) -> &[ModelBudgetConfig] {
        &self.models
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_configuration_default() {
        let config = BudgetConfiguration::default();

        assert!(config.enabled);
        assert!(config.providers.is_empty());
        assert!(config.models.is_empty());
    }

    #[test]
    fn test_budget_configuration_deserialize() {
        let yaml = r#"
enabled: true
providers:
  - provider: openai
    max_budget: 1000.0
    reset_period: monthly
    soft_limit_percentage: 0.8
    currency: USD
    enabled: true
  - provider: anthropic
    max_budget: 500.0
    reset_period: weekly
models:
  - model: gpt-4
    max_budget: 300.0
    reset_period: monthly
  - model: claude-3-opus
    max_budget: 200.0
global:
  default_soft_limit_percentage: 0.75
  block_on_exceeded: true
  log_warnings: true
  reset_check_interval_secs: 120
"#;

        let config: BudgetConfiguration = serde_yml::from_str(yaml).unwrap();

        assert!(config.enabled);
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.models.len(), 2);

        assert_eq!(config.providers[0].provider, "openai");
        assert_eq!(config.providers[0].max_budget, 1000.0);
        assert_eq!(config.providers[0].reset_period, ResetPeriodConfig::Monthly);

        assert_eq!(config.providers[1].provider, "anthropic");
        assert_eq!(config.providers[1].max_budget, 500.0);
        assert_eq!(config.providers[1].reset_period, ResetPeriodConfig::Weekly);

        assert_eq!(config.models[0].model, "gpt-4");
        assert_eq!(config.models[0].max_budget, 300.0);

        assert_eq!(config.global.default_soft_limit_percentage, 0.75);
        assert!(config.global.block_on_exceeded);
        assert_eq!(config.global.reset_check_interval_secs, 120);
    }

    #[test]
    fn test_provider_budget_config_defaults() {
        let yaml = r#"
provider: openai
max_budget: 1000.0
"#;

        let config: ProviderBudgetConfig = serde_yml::from_str(yaml).unwrap();

        assert_eq!(config.provider, "openai");
        assert_eq!(config.max_budget, 1000.0);
        assert_eq!(config.reset_period, ResetPeriodConfig::Monthly);
        assert_eq!(config.soft_limit_percentage, 0.8);
        assert_eq!(config.currency, "USD");
        assert!(config.enabled);
    }

    #[test]
    fn test_model_budget_config_defaults() {
        let yaml = r#"
model: gpt-4
max_budget: 500.0
"#;

        let config: ModelBudgetConfig = serde_yml::from_str(yaml).unwrap();

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_budget, 500.0);
        assert_eq!(config.reset_period, ResetPeriodConfig::Monthly);
        assert!(config.enabled);
    }

    #[test]
    fn test_reset_period_config() {
        assert_eq!(
            serde_yml::from_str::<ResetPeriodConfig>("daily").unwrap(),
            ResetPeriodConfig::Daily
        );
        assert_eq!(
            serde_yml::from_str::<ResetPeriodConfig>("weekly").unwrap(),
            ResetPeriodConfig::Weekly
        );
        assert_eq!(
            serde_yml::from_str::<ResetPeriodConfig>("monthly").unwrap(),
            ResetPeriodConfig::Monthly
        );
        assert_eq!(
            serde_yml::from_str::<ResetPeriodConfig>("never").unwrap(),
            ResetPeriodConfig::Never
        );
    }
}
