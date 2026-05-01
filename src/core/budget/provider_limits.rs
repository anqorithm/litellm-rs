//! Provider and Model budget limits management
//!
//! This module provides specialized budget management for per-provider
//! and per-model budget limits with lock-free concurrent access.

use super::types::{
    BudgetStatus, Currency, ModelBudget, ModelUsageStats, ProviderBudget, ProviderUsageStats,
    ResetPeriod,
};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info, warn};

/// Configuration for setting a provider budget limit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Create a new provider limit configuration
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

/// Configuration for setting a model budget limit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Create a new model limit configuration
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Internal state for tracking request counts
struct RequestCounter {
    count: AtomicU64,
}

impl Default for RequestCounter {
    fn default() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }
}

/// Manager for per-provider budget limits
///
/// Uses DashMap for lock-free concurrent access, allowing multiple threads
/// to check and update provider budgets simultaneously.
#[derive(Clone)]
pub struct ProviderBudgetManager {
    /// Provider budgets keyed by provider name
    budgets: Arc<DashMap<String, ProviderBudget>>,
    /// Request counters per provider
    request_counts: Arc<DashMap<String, RequestCounter>>,
    /// Whether the manager is enabled
    enabled: Arc<std::sync::atomic::AtomicBool>,
    /// Optional async persistence sink
    persistence_tx: Option<BudgetPersistenceSender>,
}

impl Default for ProviderBudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderBudgetManager {
    /// Create a new provider budget manager
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(DashMap::new()),
            request_counts: Arc::new(DashMap::new()),
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            persistence_tx: None,
        }
    }

    /// Create a provider budget manager with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            budgets: Arc::new(DashMap::with_capacity(capacity)),
            request_counts: Arc::new(DashMap::with_capacity(capacity)),
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            persistence_tx: None,
        }
    }

    /// Attach an async persistence sink.
    pub fn with_persistence(mut self, persistence_tx: BudgetPersistenceSender) -> Self {
        self.persistence_tx = Some(persistence_tx);
        self
    }

    fn send_persistence_event(&self, event: BudgetPersistenceEvent) {
        if let Some(tx) = &self.persistence_tx
            && tx.send(event).is_err()
        {
            warn!("Provider budget persistence worker is unavailable");
        }
    }

    fn snapshot_for(&self, budget: &ProviderBudget) -> BudgetLimitSnapshot {
        let request_count = self
            .request_counts
            .get(&budget.provider_name)
            .map(|c| c.count.load(Ordering::Relaxed))
            .unwrap_or(0);

        BudgetLimitSnapshot {
            kind: BudgetLimitKind::Provider,
            name: budget.provider_name.clone(),
            max_budget: budget.max_budget,
            current_spend: budget.current_spend,
            soft_limit: budget.soft_limit,
            reset_period: budget.reset_period,
            currency: budget.currency,
            enabled: budget.enabled,
            last_reset_at: budget.last_reset_at,
            request_count,
        }
    }

    /// Restore a provider budget from durable state without re-emitting persistence events.
    pub fn restore_snapshot(&self, snapshot: &BudgetLimitSnapshot) {
        if snapshot.kind != BudgetLimitKind::Provider {
            return;
        }

        let mut budget = ProviderBudget::new(&snapshot.name, snapshot.max_budget);
        budget.current_spend = snapshot.current_spend;
        budget.soft_limit = snapshot.soft_limit;
        budget.reset_period = snapshot.reset_period;
        budget.currency = snapshot.currency;
        budget.enabled = snapshot.enabled;
        budget.last_reset_at = snapshot.last_reset_at;

        self.budgets.insert(snapshot.name.clone(), budget);
        self.request_counts.insert(
            snapshot.name.clone(),
            RequestCounter {
                count: AtomicU64::new(snapshot.request_count),
            },
        );
    }

    /// Check if the manager is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable the manager
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Set a provider budget limit
    pub fn set_provider_limit(&self, provider: &str, config: ProviderLimitConfig) {
        let mut budget = ProviderBudget::new(provider, config.max_budget);
        budget.soft_limit = config.max_budget * config.soft_limit_percentage;
        budget.reset_period = config.reset_period;
        budget.currency = config.currency;
        budget.enabled = config.enabled;

        info!(
            "Setting provider budget limit for '{}': ${:.2} ({})",
            provider, config.max_budget, config.reset_period
        );

        let snapshot = self.snapshot_for(&budget);
        self.budgets.insert(provider.to_string(), budget);
        self.request_counts.entry(provider.to_string()).or_default();
        self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
    }

    /// Remove a provider budget limit
    pub fn remove_provider_limit(&self, provider: &str) -> bool {
        let removed = self.budgets.remove(provider).is_some();
        self.request_counts.remove(provider);

        if removed {
            info!("Removed provider budget limit for '{}'", provider);
            self.send_persistence_event(BudgetPersistenceEvent::Delete {
                kind: BudgetLimitKind::Provider,
                name: provider.to_string(),
            });
        }

        removed
    }

    /// Check if a provider has a budget limit
    pub fn has_provider_limit(&self, provider: &str) -> bool {
        self.budgets.contains_key(provider)
    }

    /// Check provider budget status
    ///
    /// Returns the current budget status for the provider.
    /// Returns `BudgetStatus::Ok` if no budget is configured.
    pub fn check_provider_budget(&self, provider: &str) -> BudgetStatus {
        if !self.is_enabled() {
            return BudgetStatus::Ok;
        }

        match self.budgets.get(provider) {
            Some(budget) => {
                if !budget.enabled {
                    return BudgetStatus::Ok;
                }
                budget.status()
            }
            None => BudgetStatus::Ok,
        }
    }

    /// Check if a provider can spend the given amount
    ///
    /// Returns true if the spend is allowed, false otherwise.
    pub fn can_provider_spend(&self, provider: &str, amount: f64) -> bool {
        if !self.is_enabled() {
            return true;
        }

        match self.budgets.get(provider) {
            Some(budget) => budget.can_spend(amount),
            None => true, // No budget configured = unlimited
        }
    }

    /// Record spending for a provider
    ///
    /// Returns the new budget status after recording the spend.
    pub fn record_provider_spend(&self, provider: &str, amount: f64) -> Option<BudgetStatus> {
        if amount <= 0.0 {
            return None;
        }

        // Increment request count
        if let Some(counter) = self.request_counts.get(provider) {
            counter.count.fetch_add(1, Ordering::Relaxed);
        }

        // Record spend
        self.budgets.get_mut(provider).map(|mut budget| {
            let previous_status = budget.status();
            budget.record_spend(amount);
            let new_status = budget.status();

            debug!(
                "Recorded spend ${:.4} for provider '{}': ${:.2} / ${:.2} ({})",
                amount, provider, budget.current_spend, budget.max_budget, new_status
            );

            // Log status transitions
            if new_status != previous_status {
                match new_status {
                    BudgetStatus::Warning => {
                        warn!(
                            "Provider '{}' approaching budget limit: ${:.2} / ${:.2} ({:.1}%)",
                            provider,
                            budget.current_spend,
                            budget.max_budget,
                            budget.usage_percentage()
                        );
                    }
                    BudgetStatus::Exceeded => {
                        warn!(
                            "Provider '{}' exceeded budget limit: ${:.2} / ${:.2}",
                            provider, budget.current_spend, budget.max_budget
                        );
                    }
                    BudgetStatus::Ok => {}
                }
            }

            let snapshot = self.snapshot_for(&budget);
            drop(budget);
            self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));

            new_status
        })
    }

    /// Get provider usage statistics
    pub fn get_provider_usage(&self, provider: &str) -> Option<ProviderUsageStats> {
        let budget = self.budgets.get(provider)?;
        let request_count = self
            .request_counts
            .get(provider)
            .map(|c| c.count.load(Ordering::Relaxed))
            .unwrap_or(0);

        Some(ProviderUsageStats {
            provider_name: provider.to_string(),
            current_spend: budget.current_spend,
            max_budget: budget.max_budget,
            remaining: budget.remaining(),
            usage_percentage: budget.usage_percentage(),
            status: budget.status(),
            reset_period: budget.reset_period,
            last_reset_at: budget.last_reset_at,
            request_count,
        })
    }

    /// Get all provider budgets
    pub fn list_provider_budgets(&self) -> Vec<ProviderBudget> {
        self.budgets.iter().map(|r| r.value().clone()).collect()
    }

    /// Get all provider usage statistics
    pub fn list_provider_usage(&self) -> Vec<ProviderUsageStats> {
        self.budgets
            .iter()
            .map(|r| {
                let provider = r.key();
                let budget = r.value();
                let request_count = self
                    .request_counts
                    .get(provider)
                    .map(|c| c.count.load(Ordering::Relaxed))
                    .unwrap_or(0);

                ProviderUsageStats {
                    provider_name: provider.clone(),
                    current_spend: budget.current_spend,
                    max_budget: budget.max_budget,
                    remaining: budget.remaining(),
                    usage_percentage: budget.usage_percentage(),
                    status: budget.status(),
                    reset_period: budget.reset_period,
                    last_reset_at: budget.last_reset_at,
                    request_count,
                }
            })
            .collect()
    }

    /// Get providers that are within budget
    pub fn get_available_providers(&self) -> Vec<String> {
        if !self.is_enabled() {
            return self.budgets.iter().map(|r| r.key().clone()).collect();
        }

        self.budgets
            .iter()
            .filter(|r| {
                let budget = r.value();
                !budget.enabled || budget.status() != BudgetStatus::Exceeded
            })
            .map(|r| r.key().clone())
            .collect()
    }

    /// Get providers that have exceeded their budget
    pub fn get_exceeded_providers(&self) -> Vec<String> {
        self.budgets
            .iter()
            .filter(|r| {
                let budget = r.value();
                budget.enabled && budget.status() == BudgetStatus::Exceeded
            })
            .map(|r| r.key().clone())
            .collect()
    }

    /// Reset a specific provider's budget
    pub fn reset_provider_budget(&self, provider: &str) -> bool {
        if let Some(mut budget) = self.budgets.get_mut(provider) {
            info!(
                "Resetting provider '{}' budget (was ${:.2})",
                provider, budget.current_spend
            );
            budget.reset();

            // Reset request count
            if let Some(counter) = self.request_counts.get(provider) {
                counter.count.store(0, Ordering::Relaxed);
            }

            let snapshot = self.snapshot_for(&budget);
            drop(budget);
            self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
            true
        } else {
            false
        }
    }

    /// Reset all provider budgets that are due based on their reset period
    pub fn reset_due_budgets(&self) -> Vec<String> {
        let mut reset_providers = Vec::new();

        for mut entry in self.budgets.iter_mut() {
            if entry.value().should_reset() {
                let provider = entry.key().clone();
                info!(
                    "Auto-resetting provider '{}' budget (was ${:.2})",
                    provider,
                    entry.value().current_spend
                );
                entry.value_mut().reset();

                // Reset request count
                if let Some(counter) = self.request_counts.get(&provider) {
                    counter.count.store(0, Ordering::Relaxed);
                }

                let snapshot = self.snapshot_for(entry.value());
                self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
                reset_providers.push(provider);
            }
        }

        reset_providers
    }

    /// Get provider count
    pub fn provider_count(&self) -> usize {
        self.budgets.len()
    }
}

/// Manager for per-model budget limits
#[derive(Clone)]
pub struct ModelBudgetManager {
    /// Model budgets keyed by model name
    budgets: Arc<DashMap<String, ModelBudget>>,
    /// Request counters per model
    request_counts: Arc<DashMap<String, RequestCounter>>,
    /// Whether the manager is enabled
    enabled: Arc<std::sync::atomic::AtomicBool>,
    /// Optional async persistence sink
    persistence_tx: Option<BudgetPersistenceSender>,
}

impl Default for ModelBudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelBudgetManager {
    /// Create a new model budget manager
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(DashMap::new()),
            request_counts: Arc::new(DashMap::new()),
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            persistence_tx: None,
        }
    }

    /// Attach an async persistence sink.
    pub fn with_persistence(mut self, persistence_tx: BudgetPersistenceSender) -> Self {
        self.persistence_tx = Some(persistence_tx);
        self
    }

    fn send_persistence_event(&self, event: BudgetPersistenceEvent) {
        if let Some(tx) = &self.persistence_tx
            && tx.send(event).is_err()
        {
            warn!("Model budget persistence worker is unavailable");
        }
    }

    fn snapshot_for(&self, budget: &ModelBudget) -> BudgetLimitSnapshot {
        let request_count = self
            .request_counts
            .get(&budget.model_name)
            .map(|c| c.count.load(Ordering::Relaxed))
            .unwrap_or(0);

        BudgetLimitSnapshot {
            kind: BudgetLimitKind::Model,
            name: budget.model_name.clone(),
            max_budget: budget.max_budget,
            current_spend: budget.current_spend,
            soft_limit: budget.soft_limit,
            reset_period: budget.reset_period,
            currency: budget.currency,
            enabled: budget.enabled,
            last_reset_at: budget.last_reset_at,
            request_count,
        }
    }

    /// Restore a model budget from durable state without re-emitting persistence events.
    pub fn restore_snapshot(&self, snapshot: &BudgetLimitSnapshot) {
        if snapshot.kind != BudgetLimitKind::Model {
            return;
        }

        let mut budget = ModelBudget::new(&snapshot.name, snapshot.max_budget);
        budget.current_spend = snapshot.current_spend;
        budget.soft_limit = snapshot.soft_limit;
        budget.reset_period = snapshot.reset_period;
        budget.currency = snapshot.currency;
        budget.enabled = snapshot.enabled;
        budget.last_reset_at = snapshot.last_reset_at;

        self.budgets.insert(snapshot.name.clone(), budget);
        self.request_counts.insert(
            snapshot.name.clone(),
            RequestCounter {
                count: AtomicU64::new(snapshot.request_count),
            },
        );
    }

    /// Check if the manager is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable the manager
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Set a model budget limit
    pub fn set_model_limit(&self, model: &str, config: ModelLimitConfig) {
        let mut budget = ModelBudget::new(model, config.max_budget);
        budget.soft_limit = config.max_budget * config.soft_limit_percentage;
        budget.reset_period = config.reset_period;
        budget.currency = config.currency;
        budget.enabled = config.enabled;

        info!(
            "Setting model budget limit for '{}': ${:.2} ({})",
            model, config.max_budget, config.reset_period
        );

        let snapshot = self.snapshot_for(&budget);
        self.budgets.insert(model.to_string(), budget);
        self.request_counts.entry(model.to_string()).or_default();
        self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
    }

    /// Remove a model budget limit
    pub fn remove_model_limit(&self, model: &str) -> bool {
        let removed = self.budgets.remove(model).is_some();
        self.request_counts.remove(model);

        if removed {
            info!("Removed model budget limit for '{}'", model);
            self.send_persistence_event(BudgetPersistenceEvent::Delete {
                kind: BudgetLimitKind::Model,
                name: model.to_string(),
            });
        }

        removed
    }

    /// Check if a model has a budget limit
    pub fn has_model_limit(&self, model: &str) -> bool {
        self.budgets.contains_key(model)
    }

    /// Check model budget status
    pub fn check_model_budget(&self, model: &str) -> BudgetStatus {
        if !self.is_enabled() {
            return BudgetStatus::Ok;
        }

        match self.budgets.get(model) {
            Some(budget) => {
                if !budget.enabled {
                    return BudgetStatus::Ok;
                }
                budget.status()
            }
            None => BudgetStatus::Ok,
        }
    }

    /// Check if a model can spend the given amount
    pub fn can_model_spend(&self, model: &str, amount: f64) -> bool {
        if !self.is_enabled() {
            return true;
        }

        match self.budgets.get(model) {
            Some(budget) => budget.can_spend(amount),
            None => true,
        }
    }

    /// Record spending for a model
    pub fn record_model_spend(&self, model: &str, amount: f64) -> Option<BudgetStatus> {
        if amount <= 0.0 {
            return None;
        }

        // Increment request count
        if let Some(counter) = self.request_counts.get(model) {
            counter.count.fetch_add(1, Ordering::Relaxed);
        }

        // Record spend
        self.budgets.get_mut(model).map(|mut budget| {
            let previous_status = budget.status();
            budget.record_spend(amount);
            let new_status = budget.status();

            debug!(
                "Recorded spend ${:.4} for model '{}': ${:.2} / ${:.2} ({})",
                amount, model, budget.current_spend, budget.max_budget, new_status
            );

            if new_status != previous_status && new_status == BudgetStatus::Exceeded {
                warn!(
                    "Model '{}' exceeded budget limit: ${:.2} / ${:.2}",
                    model, budget.current_spend, budget.max_budget
                );
            }

            let snapshot = self.snapshot_for(&budget);
            drop(budget);
            self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));

            new_status
        })
    }

    /// Get model usage statistics
    pub fn get_model_usage(&self, model: &str) -> Option<ModelUsageStats> {
        let budget = self.budgets.get(model)?;
        let request_count = self
            .request_counts
            .get(model)
            .map(|c| c.count.load(Ordering::Relaxed))
            .unwrap_or(0);

        Some(ModelUsageStats {
            model_name: model.to_string(),
            current_spend: budget.current_spend,
            max_budget: budget.max_budget,
            remaining: budget.remaining(),
            usage_percentage: budget.usage_percentage(),
            status: budget.status(),
            reset_period: budget.reset_period,
            last_reset_at: budget.last_reset_at,
            request_count,
        })
    }

    /// Get all model budgets
    pub fn list_model_budgets(&self) -> Vec<ModelBudget> {
        self.budgets.iter().map(|r| r.value().clone()).collect()
    }

    /// Get all model usage statistics
    pub fn list_model_usage(&self) -> Vec<ModelUsageStats> {
        self.budgets
            .iter()
            .map(|r| {
                let model = r.key();
                let budget = r.value();
                let request_count = self
                    .request_counts
                    .get(model)
                    .map(|c| c.count.load(Ordering::Relaxed))
                    .unwrap_or(0);

                ModelUsageStats {
                    model_name: model.clone(),
                    current_spend: budget.current_spend,
                    max_budget: budget.max_budget,
                    remaining: budget.remaining(),
                    usage_percentage: budget.usage_percentage(),
                    status: budget.status(),
                    reset_period: budget.reset_period,
                    last_reset_at: budget.last_reset_at,
                    request_count,
                }
            })
            .collect()
    }

    /// Reset a specific model's budget
    pub fn reset_model_budget(&self, model: &str) -> bool {
        if let Some(mut budget) = self.budgets.get_mut(model) {
            info!(
                "Resetting model '{}' budget (was ${:.2})",
                model, budget.current_spend
            );
            budget.reset();

            if let Some(counter) = self.request_counts.get(model) {
                counter.count.store(0, Ordering::Relaxed);
            }

            let snapshot = self.snapshot_for(&budget);
            drop(budget);
            self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
            true
        } else {
            false
        }
    }

    /// Reset all model budgets that are due
    pub fn reset_due_budgets(&self) -> Vec<String> {
        let mut reset_models = Vec::new();

        for mut entry in self.budgets.iter_mut() {
            if entry.value().should_reset() {
                let model = entry.key().clone();
                info!(
                    "Auto-resetting model '{}' budget (was ${:.2})",
                    model,
                    entry.value().current_spend
                );
                entry.value_mut().reset();

                if let Some(counter) = self.request_counts.get(&model) {
                    counter.count.store(0, Ordering::Relaxed);
                }

                let snapshot = self.snapshot_for(entry.value());
                self.send_persistence_event(BudgetPersistenceEvent::Upsert(snapshot));
                reset_models.push(model);
            }
        }

        reset_models
    }

    /// Get model count
    pub fn model_count(&self) -> usize {
        self.budgets.len()
    }
}

/// Combined manager for both provider and model budgets
#[derive(Clone)]
pub struct UnifiedBudgetLimits {
    /// Provider budget manager
    pub providers: ProviderBudgetManager,
    /// Model budget manager
    pub models: ModelBudgetManager,
}

impl Default for UnifiedBudgetLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedBudgetLimits {
    /// Create a new unified budget limits manager
    pub fn new() -> Self {
        Self {
            providers: ProviderBudgetManager::new(),
            models: ModelBudgetManager::new(),
        }
    }

    /// Create a unified budget limits manager with persistence enabled.
    pub fn with_persistence(persistence_tx: BudgetPersistenceSender) -> Self {
        Self {
            providers: ProviderBudgetManager::new().with_persistence(persistence_tx.clone()),
            models: ModelBudgetManager::new().with_persistence(persistence_tx),
        }
    }

    /// Restore persisted provider/model snapshots and attach a persistence sink.
    pub fn from_snapshots_with_persistence(
        snapshots: impl IntoIterator<Item = BudgetLimitSnapshot>,
        persistence_tx: BudgetPersistenceSender,
    ) -> Self {
        let limits = Self::with_persistence(persistence_tx);
        for snapshot in snapshots {
            match snapshot.kind {
                BudgetLimitKind::Provider => limits.providers.restore_snapshot(&snapshot),
                BudgetLimitKind::Model => limits.models.restore_snapshot(&snapshot),
            }
        }
        limits
    }

    /// Check if both provider and model can spend
    pub fn can_spend(&self, provider: &str, model: &str, amount: f64) -> bool {
        self.providers.can_provider_spend(provider, amount)
            && self.models.can_model_spend(model, amount)
    }

    /// Record spend for both provider and model
    pub fn record_spend(&self, provider: &str, model: &str, amount: f64) {
        self.providers.record_provider_spend(provider, amount);
        self.models.record_model_spend(model, amount);
    }

    /// Filter out providers that have exceeded their budget
    pub fn filter_available_providers(&self, providers: Vec<String>) -> Vec<String> {
        let exceeded = self.providers.get_exceeded_providers();
        providers
            .into_iter()
            .filter(|p| !exceeded.contains(p))
            .collect()
    }

    /// Check if a provider is available (not exceeded budget)
    pub fn is_provider_available(&self, provider: &str) -> bool {
        self.providers.check_provider_budget(provider) != BudgetStatus::Exceeded
    }

    /// Check if a model is available (not exceeded budget)
    pub fn is_model_available(&self, model: &str) -> bool {
        self.models.check_model_budget(model) != BudgetStatus::Exceeded
    }

    /// Reset all due budgets
    pub fn reset_due_budgets(&self) -> (Vec<String>, Vec<String>) {
        let providers = self.providers.reset_due_budgets();
        let models = self.models.reset_due_budgets();
        (providers, models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provider Budget Manager Tests
    #[test]
    fn test_provider_budget_manager_creation() {
        let manager = ProviderBudgetManager::new();
        assert_eq!(manager.provider_count(), 0);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_set_provider_limit() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);

        assert!(manager.has_provider_limit("openai"));
        assert_eq!(manager.provider_count(), 1);
    }

    #[test]
    fn test_remove_provider_limit() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);
        assert!(manager.has_provider_limit("openai"));

        assert!(manager.remove_provider_limit("openai"));
        assert!(!manager.has_provider_limit("openai"));
        assert_eq!(manager.provider_count(), 0);
    }

    #[test]
    fn test_check_provider_budget() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);

        assert_eq!(manager.check_provider_budget("openai"), BudgetStatus::Ok);
        assert_eq!(manager.check_provider_budget("unknown"), BudgetStatus::Ok);
    }

    #[test]
    fn test_can_provider_spend() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);

        assert!(manager.can_provider_spend("openai", 50.0));
        assert!(manager.can_provider_spend("openai", 100.0));
        assert!(!manager.can_provider_spend("openai", 101.0));

        // Unknown provider has no limit
        assert!(manager.can_provider_spend("unknown", 10000.0));
    }

    #[test]
    fn test_record_provider_spend() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);

        let status = manager.record_provider_spend("openai", 50.0);
        assert_eq!(status, Some(BudgetStatus::Ok));

        let status = manager.record_provider_spend("openai", 30.0);
        assert_eq!(status, Some(BudgetStatus::Warning));

        let status = manager.record_provider_spend("openai", 25.0);
        assert_eq!(status, Some(BudgetStatus::Exceeded));
    }

    #[test]
    fn test_get_provider_usage() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);
        manager.record_provider_spend("openai", 30.0);

        let usage = manager.get_provider_usage("openai").unwrap();

        assert_eq!(usage.provider_name, "openai");
        assert_eq!(usage.current_spend, 30.0);
        assert_eq!(usage.max_budget, 100.0);
        assert_eq!(usage.remaining, 70.0);
        assert_eq!(usage.request_count, 1);
    }

    #[test]
    fn test_get_available_providers() {
        let manager = ProviderBudgetManager::new();

        manager.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );
        manager.set_provider_limit(
            "anthropic",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );

        // Exceed openai budget
        manager.record_provider_spend("openai", 150.0);

        let available = manager.get_available_providers();
        assert!(available.contains(&"anthropic".to_string()));
        assert!(!available.contains(&"openai".to_string()));
    }

    #[test]
    fn test_get_exceeded_providers() {
        let manager = ProviderBudgetManager::new();

        manager.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );
        manager.record_provider_spend("openai", 150.0);

        let exceeded = manager.get_exceeded_providers();
        assert_eq!(exceeded.len(), 1);
        assert_eq!(exceeded[0], "openai");
    }

    #[test]
    fn test_reset_provider_budget() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);
        manager.record_provider_spend("openai", 75.0);

        assert!(manager.reset_provider_budget("openai"));

        let usage = manager.get_provider_usage("openai").unwrap();
        assert_eq!(usage.current_spend, 0.0);
        assert_eq!(usage.request_count, 0);
    }

    #[test]
    fn test_disabled_manager_allows_all() {
        let manager = ProviderBudgetManager::new();
        let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_provider_limit("openai", config);
        manager.record_provider_spend("openai", 150.0);

        // Normally would be exceeded
        assert_eq!(
            manager.check_provider_budget("openai"),
            BudgetStatus::Exceeded
        );

        // Disable manager
        manager.set_enabled(false);

        // Now returns Ok and allows spending
        assert_eq!(manager.check_provider_budget("openai"), BudgetStatus::Ok);
        assert!(manager.can_provider_spend("openai", 1000.0));
    }

    // Model Budget Manager Tests
    #[test]
    fn test_model_budget_manager_creation() {
        let manager = ModelBudgetManager::new();
        assert_eq!(manager.model_count(), 0);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_set_model_limit() {
        let manager = ModelBudgetManager::new();
        let config = ModelLimitConfig::new(500.0, ResetPeriod::Monthly);

        manager.set_model_limit("gpt-4", config);

        assert!(manager.has_model_limit("gpt-4"));
        assert_eq!(manager.model_count(), 1);
    }

    #[test]
    fn test_check_model_budget() {
        let manager = ModelBudgetManager::new();
        let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_model_limit("gpt-4", config);

        assert_eq!(manager.check_model_budget("gpt-4"), BudgetStatus::Ok);
    }

    #[test]
    fn test_record_model_spend() {
        let manager = ModelBudgetManager::new();
        let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_model_limit("gpt-4", config);

        let status = manager.record_model_spend("gpt-4", 50.0);
        assert_eq!(status, Some(BudgetStatus::Ok));

        let status = manager.record_model_spend("gpt-4", 55.0);
        assert_eq!(status, Some(BudgetStatus::Exceeded));
    }

    #[test]
    fn test_get_model_usage() {
        let manager = ModelBudgetManager::new();
        let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

        manager.set_model_limit("gpt-4", config);
        manager.record_model_spend("gpt-4", 25.0);

        let usage = manager.get_model_usage("gpt-4").unwrap();

        assert_eq!(usage.model_name, "gpt-4");
        assert_eq!(usage.current_spend, 25.0);
        assert_eq!(usage.request_count, 1);
    }

    // Unified Budget Limits Tests
    #[test]
    fn test_unified_budget_limits() {
        let limits = UnifiedBudgetLimits::new();

        limits.providers.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
        );
        limits
            .models
            .set_model_limit("gpt-4", ModelLimitConfig::new(500.0, ResetPeriod::Monthly));

        assert!(limits.can_spend("openai", "gpt-4", 100.0));

        limits.record_spend("openai", "gpt-4", 100.0);

        let provider_usage = limits.providers.get_provider_usage("openai").unwrap();
        let model_usage = limits.models.get_model_usage("gpt-4").unwrap();

        assert_eq!(provider_usage.current_spend, 100.0);
        assert_eq!(model_usage.current_spend, 100.0);
    }

    #[test]
    fn test_unified_budget_limits_restore_snapshots() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let limits = UnifiedBudgetLimits::from_snapshots_with_persistence(
            vec![
                BudgetLimitSnapshot {
                    kind: BudgetLimitKind::Provider,
                    name: "openai".to_string(),
                    max_budget: 100.0,
                    current_spend: 25.0,
                    soft_limit: 80.0,
                    reset_period: ResetPeriod::Monthly,
                    currency: Currency::USD,
                    enabled: true,
                    last_reset_at: None,
                    request_count: 3,
                },
                BudgetLimitSnapshot {
                    kind: BudgetLimitKind::Model,
                    name: "gpt-4".to_string(),
                    max_budget: 50.0,
                    current_spend: 10.0,
                    soft_limit: 40.0,
                    reset_period: ResetPeriod::Daily,
                    currency: Currency::USD,
                    enabled: true,
                    last_reset_at: None,
                    request_count: 2,
                },
            ],
            tx,
        );

        let provider_usage = limits.providers.get_provider_usage("openai").unwrap();
        assert_eq!(provider_usage.current_spend, 25.0);
        assert_eq!(provider_usage.request_count, 3);

        let model_usage = limits.models.get_model_usage("gpt-4").unwrap();
        assert_eq!(model_usage.current_spend, 10.0);
        assert_eq!(model_usage.request_count, 2);
    }

    #[test]
    fn test_provider_budget_manager_emits_persistence_events() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let manager = ProviderBudgetManager::new().with_persistence(tx);

        manager.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );
        manager.record_provider_spend("openai", 12.5);

        let first = rx.try_recv().expect("set should emit an upsert");
        assert!(matches!(
            first,
            BudgetPersistenceEvent::Upsert(BudgetLimitSnapshot {
                kind: BudgetLimitKind::Provider,
                ..
            })
        ));

        let second = rx.try_recv().expect("spend should emit an upsert");
        match second {
            BudgetPersistenceEvent::Upsert(snapshot) => {
                assert_eq!(snapshot.name, "openai");
                assert_eq!(snapshot.current_spend, 12.5);
                assert_eq!(snapshot.request_count, 1);
            }
            BudgetPersistenceEvent::Delete { .. } => panic!("expected upsert"),
        }
    }

    #[test]
    fn test_filter_available_providers() {
        let limits = UnifiedBudgetLimits::new();

        limits.providers.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );
        limits.providers.set_provider_limit(
            "anthropic",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );
        limits.providers.set_provider_limit(
            "google",
            ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
        );

        // Exceed openai budget
        limits.providers.record_provider_spend("openai", 150.0);

        let providers = vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "google".to_string(),
        ];
        let available = limits.filter_available_providers(providers);

        assert_eq!(available.len(), 2);
        assert!(!available.contains(&"openai".to_string()));
        assert!(available.contains(&"anthropic".to_string()));
        assert!(available.contains(&"google".to_string()));
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let manager = Arc::new(ProviderBudgetManager::new());
        manager.set_provider_limit(
            "openai",
            ProviderLimitConfig::new(10000.0, ResetPeriod::Monthly),
        );

        let mut handles = vec![];

        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    manager_clone.record_provider_spend("openai", 1.0);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let usage = manager.get_provider_usage("openai").unwrap();
        assert_eq!(usage.current_spend, 1000.0);
        assert_eq!(usage.request_count, 1000);
    }
}
