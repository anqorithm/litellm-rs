//! Budget types and data structures
//!
//! This module defines the core types for budget management including
//! Budget, BudgetScope, BudgetAlert, and BudgetStatus.

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use super::config::BudgetConfig;
use super::{add_budget_spend, budget_can_spend};

/// Reset period for budgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResetPeriod {
    /// Reset daily at midnight UTC
    Daily,
    /// Reset weekly on Sunday at midnight UTC
    Weekly,
    /// Reset monthly on the 1st at midnight UTC
    #[default]
    Monthly,
    /// Never reset (lifetime budget)
    Never,
}

impl fmt::Display for ResetPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Never => write!(f, "never"),
        }
    }
}

/// Currency for budget amounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    #[default]
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::USD => write!(f, "USD"),
            Self::EUR => write!(f, "EUR"),
            Self::GBP => write!(f, "GBP"),
            Self::JPY => write!(f, "JPY"),
            Self::CNY => write!(f, "CNY"),
        }
    }
}

/// Budget scope - defines what entity the budget applies to
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum BudgetScope {
    /// Budget for a specific user
    User(String),
    /// Budget for a team
    Team(String),
    /// Budget for an API key
    ApiKey(String),
    /// Budget for a specific provider (e.g., "openai", "anthropic")
    Provider(String),
    /// Budget for a specific model (e.g., "gpt-4", "claude-3-opus")
    Model(String),
    /// Global budget for the entire gateway
    Global,
}

impl fmt::Display for BudgetScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User(id) => write!(f, "user:{}", id),
            Self::Team(id) => write!(f, "team:{}", id),
            Self::ApiKey(id) => write!(f, "api_key:{}", id),
            Self::Provider(id) => write!(f, "provider:{}", id),
            Self::Model(id) => write!(f, "model:{}", id),
            Self::Global => write!(f, "global"),
        }
    }
}

impl BudgetScope {
    /// Create a scope key for storage
    pub fn to_key(&self) -> String {
        self.to_string()
    }

    /// Parse a scope from a key string
    pub fn from_key(key: &str) -> Option<Self> {
        if key == "global" {
            return Some(Self::Global);
        }

        let parts: Vec<&str> = key.splitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }

        match parts[0] {
            "user" => Some(Self::User(parts[1].to_string())),
            "team" => Some(Self::Team(parts[1].to_string())),
            "api_key" => Some(Self::ApiKey(parts[1].to_string())),
            "provider" => Some(Self::Provider(parts[1].to_string())),
            "model" => Some(Self::Model(parts[1].to_string())),
            _ => None,
        }
    }
}

/// Budget status indicating the current state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BudgetStatus {
    /// Budget is within normal limits
    #[default]
    Ok,
    /// Budget has reached soft limit (warning threshold)
    Warning,
    /// Budget has been exceeded
    Exceeded,
}

impl fmt::Display for BudgetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Warning => write!(f, "warning"),
            Self::Exceeded => write!(f, "exceeded"),
        }
    }
}

/// Budget configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Unique budget identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Budget scope (user, team, api_key, provider, model, or global)
    pub scope: BudgetScope,
    /// Maximum budget amount
    pub max_budget: f64,
    /// Soft limit (warning threshold), typically 80% of max_budget
    pub soft_limit: f64,
    /// Current spend amount
    pub current_spend: f64,
    /// Reset period for the budget
    pub reset_period: ResetPeriod,
    /// Currency for budget amounts
    pub currency: Currency,
    /// Whether the budget is enabled
    pub enabled: bool,
    /// When the budget was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last reset
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Budget {
    /// Create a new budget with the given configuration
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        scope: BudgetScope,
        max_budget: f64,
    ) -> Self {
        let now = chrono::Utc::now();
        let soft_limit = max_budget * 0.8; // Default soft limit at 80%

        Self {
            id: id.into(),
            name: name.into(),
            scope,
            max_budget,
            soft_limit,
            current_spend: 0.0,
            reset_period: ResetPeriod::default(),
            currency: Currency::default(),
            enabled: true,
            created_at: now,
            updated_at: now,
            last_reset_at: Some(now),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get the current budget status
    pub fn status(&self) -> BudgetStatus {
        if self.current_spend >= self.max_budget {
            BudgetStatus::Exceeded
        } else if self.current_spend >= self.soft_limit {
            BudgetStatus::Warning
        } else {
            BudgetStatus::Ok
        }
    }

    /// Get remaining budget
    pub fn remaining(&self) -> f64 {
        (self.max_budget - self.current_spend).max(0.0)
    }

    /// Get usage percentage (0.0 to 100.0+)
    pub fn usage_percentage(&self) -> f64 {
        if self.max_budget <= 0.0 {
            return 0.0;
        }
        (self.current_spend / self.max_budget) * 100.0
    }

    /// Check if the budget allows a spend of the given amount
    pub fn can_spend(&self, amount: f64) -> bool {
        budget_can_spend(self.current_spend, self.max_budget, self.enabled, amount).unwrap_or(false)
    }

    /// Record a spend amount
    pub fn record_spend(&mut self, amount: f64) {
        if let Ok(current_spend) = add_budget_spend(self.current_spend, amount) {
            self.current_spend = current_spend.as_f64();
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Reset the budget
    pub fn reset(&mut self) {
        self.current_spend = 0.0;
        let now = chrono::Utc::now();
        self.last_reset_at = Some(now);
        self.updated_at = now;
    }

    /// Check if the budget should be reset based on the reset period
    pub fn should_reset(&self) -> bool {
        let now = chrono::Utc::now();

        match self.reset_period {
            ResetPeriod::Never => false,
            ResetPeriod::Daily => {
                if let Some(last_reset) = self.last_reset_at {
                    now.date_naive() > last_reset.date_naive()
                } else {
                    true
                }
            }
            ResetPeriod::Weekly => {
                if let Some(last_reset) = self.last_reset_at {
                    let last_week = last_reset.iso_week();
                    let current_week = now.iso_week();
                    current_week.year() > last_week.year()
                        || (current_week.year() == last_week.year()
                            && current_week.week() > last_week.week())
                } else {
                    true
                }
            }
            ResetPeriod::Monthly => {
                if let Some(last_reset) = self.last_reset_at {
                    now.year() > last_reset.year()
                        || (now.year() == last_reset.year() && now.month() > last_reset.month())
                } else {
                    true
                }
            }
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Informational alert
    #[default]
    Info,
    /// Warning alert (soft limit reached)
    Warning,
    /// Critical alert (budget exceeded)
    Critical,
}

/// Budget alert for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    /// Unique alert identifier
    pub id: String,
    /// Budget ID this alert is for
    pub budget_id: String,
    /// Budget scope
    pub scope: BudgetScope,
    /// Alert type
    pub alert_type: BudgetAlertType,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Current spend at time of alert
    pub current_spend: f64,
    /// Threshold that was crossed
    pub threshold: f64,
    /// Maximum budget
    pub max_budget: f64,
    /// When the alert was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    /// When the alert was acknowledged
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Types of budget alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetAlertType {
    /// Soft limit warning (typically 80%)
    SoftLimitReached,
    /// Budget exceeded
    BudgetExceeded,
    /// Budget reset notification
    BudgetReset,
    /// Approaching limit (90%, 95%, etc.)
    ApproachingLimit,
}

impl fmt::Display for BudgetAlertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SoftLimitReached => write!(f, "soft_limit_reached"),
            Self::BudgetExceeded => write!(f, "budget_exceeded"),
            Self::BudgetReset => write!(f, "budget_reset"),
            Self::ApproachingLimit => write!(f, "approaching_limit"),
        }
    }
}

impl BudgetAlert {
    /// Create a new budget alert
    pub fn new(budget: &Budget, alert_type: BudgetAlertType, threshold: f64) -> Self {
        let severity = match alert_type {
            BudgetAlertType::SoftLimitReached | BudgetAlertType::ApproachingLimit => {
                AlertSeverity::Warning
            }
            BudgetAlertType::BudgetExceeded => AlertSeverity::Critical,
            BudgetAlertType::BudgetReset => AlertSeverity::Info,
        };

        let message = match alert_type {
            BudgetAlertType::SoftLimitReached => {
                format!(
                    "Budget '{}' has reached soft limit: ${:.2} of ${:.2} ({:.1}%)",
                    budget.name,
                    budget.current_spend,
                    budget.max_budget,
                    budget.usage_percentage()
                )
            }
            BudgetAlertType::BudgetExceeded => {
                format!(
                    "Budget '{}' has been exceeded: ${:.2} of ${:.2} ({:.1}%)",
                    budget.name,
                    budget.current_spend,
                    budget.max_budget,
                    budget.usage_percentage()
                )
            }
            BudgetAlertType::BudgetReset => {
                format!("Budget '{}' has been reset", budget.name)
            }
            BudgetAlertType::ApproachingLimit => {
                format!(
                    "Budget '{}' is approaching limit: ${:.2} of ${:.2} ({:.1}%)",
                    budget.name,
                    budget.current_spend,
                    budget.max_budget,
                    budget.usage_percentage()
                )
            }
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            budget_id: budget.id.clone(),
            scope: budget.scope.clone(),
            alert_type,
            severity,
            message,
            current_spend: budget.current_spend,
            threshold,
            max_budget: budget.max_budget,
            created_at: chrono::Utc::now(),
            acknowledged: false,
            acknowledged_at: None,
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
        self.acknowledged_at = Some(chrono::Utc::now());
    }
}

/// Budget check result returned when checking budget availability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheckResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Current budget status
    pub status: BudgetStatus,
    /// Current spend
    pub current_spend: f64,
    /// Maximum budget
    pub max_budget: f64,
    /// Remaining budget
    pub remaining: f64,
    /// Usage percentage
    pub usage_percentage: f64,
    /// Budget ID
    pub budget_id: String,
    /// Budget scope
    pub scope: BudgetScope,
}

impl BudgetCheckResult {
    /// Create a check result from a budget
    pub fn from_budget(budget: &Budget, request_amount: f64) -> Self {
        Self {
            allowed: budget.can_spend(request_amount),
            status: budget.status(),
            current_spend: budget.current_spend,
            max_budget: budget.max_budget,
            remaining: budget.remaining(),
            usage_percentage: budget.usage_percentage(),
            budget_id: budget.id.clone(),
            scope: budget.scope.clone(),
        }
    }

    /// Create an allowed result when no budget is configured
    pub fn no_budget() -> Self {
        Self {
            allowed: true,
            status: BudgetStatus::Ok,
            current_spend: 0.0,
            max_budget: f64::INFINITY,
            remaining: f64::INFINITY,
            usage_percentage: 0.0,
            budget_id: String::new(),
            scope: BudgetScope::Global,
        }
    }
}

/// Provider-specific budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderBudget {
    /// Provider name (e.g., "openai", "anthropic")
    pub provider_name: String,
    /// Maximum budget for this provider
    pub max_budget: f64,
    /// Current spend amount
    pub current_spend: f64,
    /// Soft limit (warning threshold)
    pub soft_limit: f64,
    /// Reset period for the budget
    pub reset_period: ResetPeriod,
    /// Currency for budget amounts
    pub currency: Currency,
    /// Whether the budget is enabled
    pub enabled: bool,
    /// When the budget was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last reset
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProviderBudget {
    /// Create a new provider budget
    pub fn new(provider_name: impl Into<String>, max_budget: f64) -> Self {
        let now = chrono::Utc::now();
        let soft_limit = max_budget * 0.8;

        Self {
            provider_name: provider_name.into(),
            max_budget,
            current_spend: 0.0,
            soft_limit,
            reset_period: ResetPeriod::default(),
            currency: Currency::default(),
            enabled: true,
            created_at: now,
            updated_at: now,
            last_reset_at: Some(now),
        }
    }

    /// Get the current budget status
    pub fn status(&self) -> BudgetStatus {
        if self.current_spend >= self.max_budget {
            BudgetStatus::Exceeded
        } else if self.current_spend >= self.soft_limit {
            BudgetStatus::Warning
        } else {
            BudgetStatus::Ok
        }
    }

    /// Get remaining budget
    pub fn remaining(&self) -> f64 {
        (self.max_budget - self.current_spend).max(0.0)
    }

    /// Get usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.max_budget <= 0.0 {
            return 0.0;
        }
        (self.current_spend / self.max_budget) * 100.0
    }

    /// Check if the budget allows spending
    pub fn can_spend(&self, amount: f64) -> bool {
        budget_can_spend(self.current_spend, self.max_budget, self.enabled, amount).unwrap_or(false)
    }

    /// Record a spend amount
    pub fn record_spend(&mut self, amount: f64) {
        if let Ok(current_spend) = add_budget_spend(self.current_spend, amount) {
            self.current_spend = current_spend.as_f64();
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Reset the budget
    pub fn reset(&mut self) {
        self.current_spend = 0.0;
        let now = chrono::Utc::now();
        self.last_reset_at = Some(now);
        self.updated_at = now;
    }

    /// Check if the budget should be reset
    pub fn should_reset(&self) -> bool {
        let now = chrono::Utc::now();

        match self.reset_period {
            ResetPeriod::Never => false,
            ResetPeriod::Daily => {
                if let Some(last_reset) = self.last_reset_at {
                    now.date_naive() > last_reset.date_naive()
                } else {
                    true
                }
            }
            ResetPeriod::Weekly => {
                if let Some(last_reset) = self.last_reset_at {
                    let last_week = last_reset.iso_week();
                    let current_week = now.iso_week();
                    current_week.year() > last_week.year()
                        || (current_week.year() == last_week.year()
                            && current_week.week() > last_week.week())
                } else {
                    true
                }
            }
            ResetPeriod::Monthly => {
                if let Some(last_reset) = self.last_reset_at {
                    now.year() > last_reset.year()
                        || (now.year() == last_reset.year() && now.month() > last_reset.month())
                } else {
                    true
                }
            }
        }
    }
}

/// Model-specific budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBudget {
    /// Model name (e.g., "gpt-4", "claude-3-opus")
    pub model_name: String,
    /// Maximum budget for this model
    pub max_budget: f64,
    /// Current spend amount
    pub current_spend: f64,
    /// Soft limit (warning threshold)
    pub soft_limit: f64,
    /// Reset period for the budget
    pub reset_period: ResetPeriod,
    /// Currency for budget amounts
    pub currency: Currency,
    /// Whether the budget is enabled
    pub enabled: bool,
    /// When the budget was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// When the budget was last reset
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ModelBudget {
    /// Create a new model budget
    pub fn new(model_name: impl Into<String>, max_budget: f64) -> Self {
        let now = chrono::Utc::now();
        let soft_limit = max_budget * 0.8;

        Self {
            model_name: model_name.into(),
            max_budget,
            current_spend: 0.0,
            soft_limit,
            reset_period: ResetPeriod::default(),
            currency: Currency::default(),
            enabled: true,
            created_at: now,
            updated_at: now,
            last_reset_at: Some(now),
        }
    }

    /// Get the current budget status
    pub fn status(&self) -> BudgetStatus {
        if self.current_spend >= self.max_budget {
            BudgetStatus::Exceeded
        } else if self.current_spend >= self.soft_limit {
            BudgetStatus::Warning
        } else {
            BudgetStatus::Ok
        }
    }

    /// Get remaining budget
    pub fn remaining(&self) -> f64 {
        (self.max_budget - self.current_spend).max(0.0)
    }

    /// Get usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.max_budget <= 0.0 {
            return 0.0;
        }
        (self.current_spend / self.max_budget) * 100.0
    }

    /// Check if the budget allows spending
    pub fn can_spend(&self, amount: f64) -> bool {
        budget_can_spend(self.current_spend, self.max_budget, self.enabled, amount).unwrap_or(false)
    }

    /// Record a spend amount
    pub fn record_spend(&mut self, amount: f64) {
        if let Ok(current_spend) = add_budget_spend(self.current_spend, amount) {
            self.current_spend = current_spend.as_f64();
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Reset the budget
    pub fn reset(&mut self) {
        self.current_spend = 0.0;
        let now = chrono::Utc::now();
        self.last_reset_at = Some(now);
        self.updated_at = now;
    }

    /// Check if the budget should be reset
    pub fn should_reset(&self) -> bool {
        let now = chrono::Utc::now();

        match self.reset_period {
            ResetPeriod::Never => false,
            ResetPeriod::Daily => {
                if let Some(last_reset) = self.last_reset_at {
                    now.date_naive() > last_reset.date_naive()
                } else {
                    true
                }
            }
            ResetPeriod::Weekly => {
                if let Some(last_reset) = self.last_reset_at {
                    let last_week = last_reset.iso_week();
                    let current_week = now.iso_week();
                    current_week.year() > last_week.year()
                        || (current_week.year() == last_week.year()
                            && current_week.week() > last_week.week())
                } else {
                    true
                }
            }
            ResetPeriod::Monthly => {
                if let Some(last_reset) = self.last_reset_at {
                    now.year() > last_reset.year()
                        || (now.year() == last_reset.year() && now.month() > last_reset.month())
                } else {
                    true
                }
            }
        }
    }
}

/// Provider usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsageStats {
    /// Provider name
    pub provider_name: String,
    /// Current spend
    pub current_spend: f64,
    /// Maximum budget
    pub max_budget: f64,
    /// Remaining budget
    pub remaining: f64,
    /// Usage percentage
    pub usage_percentage: f64,
    /// Budget status
    pub status: BudgetStatus,
    /// Reset period
    pub reset_period: ResetPeriod,
    /// Last reset time
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of requests made
    pub request_count: u64,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    /// Model name
    pub model_name: String,
    /// Current spend
    pub current_spend: f64,
    /// Maximum budget
    pub max_budget: f64,
    /// Remaining budget
    pub remaining: f64,
    /// Usage percentage
    pub usage_percentage: f64,
    /// Budget status
    pub status: BudgetStatus,
    /// Reset period
    pub reset_period: ResetPeriod,
    /// Last reset time
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of requests made
    pub request_count: u64,
}
