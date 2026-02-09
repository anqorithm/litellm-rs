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
        }
    }

    /// Create a provider budget manager with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            budgets: Arc::new(DashMap::with_capacity(capacity)),
            request_counts: Arc::new(DashMap::with_capacity(capacity)),
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
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

        self.budgets.insert(provider.to_string(), budget);
        self.request_counts.entry(provider.to_string()).or_default();
    }

    /// Remove a provider budget limit
    pub fn remove_provider_limit(&self, provider: &str) -> bool {
        let removed = self.budgets.remove(provider).is_some();
        self.request_counts.remove(provider);

        if removed {
            info!("Removed provider budget limit for '{}'", provider);
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
        }
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

        self.budgets.insert(model.to_string(), budget);
        self.request_counts.entry(model.to_string()).or_default();
    }

    /// Remove a model budget limit
    pub fn remove_model_limit(&self, model: &str) -> bool {
        let removed = self.budgets.remove(model).is_some();
        self.request_counts.remove(model);

        if removed {
            info!("Removed model budget limit for '{}'", model);
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
#[path = "provider_limits_tests.rs"]
mod tests;
