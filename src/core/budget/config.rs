use super::{Currency, ResetPeriod};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

/// Configuration for creating or updating a budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Human-readable name
    pub name: String,
    /// Maximum budget amount
    pub max_budget: f64,
    /// Soft limit (optional, defaults to 80% of max_budget)
    pub soft_limit: Option<f64>,
    /// Reset period
    pub reset_period: Option<ResetPeriod>,
    /// Currency
    pub currency: Option<Currency>,
    /// Whether the budget is enabled
    pub enabled: Option<bool>,
    /// Optional metadata
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl BudgetConfig {
    /// Create a new budget configuration.
    pub fn new(name: impl Into<String>, max_budget: f64) -> Self {
        Self {
            name: name.into(),
            max_budget,
            soft_limit: None,
            reset_period: None,
            currency: None,
            enabled: None,
            metadata: None,
        }
    }

    /// Set the soft limit.
    pub fn with_soft_limit(mut self, soft_limit: f64) -> Self {
        self.soft_limit = Some(soft_limit);
        self
    }

    /// Set the reset period.
    pub fn with_reset_period(mut self, period: ResetPeriod) -> Self {
        self.reset_period = Some(period);
        self
    }

    /// Set the currency.
    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = Some(currency);
        self
    }
}

/// Configuration for setting a provider budget limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLimitConfig {
    /// Maximum budget for this provider
    pub max_budget: f64,
    /// Reset period for the budget
    pub reset_period: ResetPeriod,
    /// Soft limit percentage (0.0 to 1.0)
    #[serde(default = "default_soft_limit_percentage")]
    pub soft_limit_percentage: f64,
    /// Currency
    #[serde(default)]
    pub currency: Currency,
    /// Whether the limit is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_soft_limit_percentage() -> f64 {
    0.8
}

fn default_enabled() -> bool {
    true
}

impl ProviderLimitConfig {
    /// Create a new provider limit configuration.
    pub fn new(max_budget: f64, reset_period: ResetPeriod) -> Self {
        Self {
            max_budget,
            reset_period,
            soft_limit_percentage: 0.8,
            currency: Currency::default(),
            enabled: true,
        }
    }
}

/// Configuration for setting a model budget limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimitConfig {
    /// Maximum budget for this model
    pub max_budget: f64,
    /// Reset period for the budget
    pub reset_period: ResetPeriod,
    /// Soft limit percentage (0.0 to 1.0)
    #[serde(default = "default_soft_limit_percentage")]
    pub soft_limit_percentage: f64,
    /// Currency
    #[serde(default)]
    pub currency: Currency,
    /// Whether the limit is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl ModelLimitConfig {
    /// Create a new model limit configuration.
    pub fn new(max_budget: f64, reset_period: ResetPeriod) -> Self {
        Self {
            max_budget,
            reset_period,
            soft_limit_percentage: 0.8,
            currency: Currency::default(),
            enabled: true,
        }
    }
}

/// Persisted budget scope kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BudgetLimitKind {
    /// Provider budget snapshot.
    Provider,
    /// Model budget snapshot.
    Model,
}

impl BudgetLimitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Model => "model",
        }
    }
}

/// Durable snapshot of a provider/model budget limit and current usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimitSnapshot {
    pub kind: BudgetLimitKind,
    pub name: String,
    pub max_budget: f64,
    pub current_spend: f64,
    pub soft_limit: f64,
    pub reset_period: ResetPeriod,
    pub currency: Currency,
    pub enabled: bool,
    pub last_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    pub request_count: u64,
}

impl BudgetLimitSnapshot {
    pub fn scope_key(&self) -> String {
        format!("{}:{}", self.kind.as_str(), self.name)
    }
}

/// Persistence events emitted after in-memory budget mutations.
#[derive(Debug, Clone)]
pub enum BudgetPersistenceEvent {
    Upsert(BudgetLimitSnapshot),
    Delete { kind: BudgetLimitKind, name: String },
}

pub type BudgetPersistenceSender = UnboundedSender<BudgetPersistenceEvent>;
