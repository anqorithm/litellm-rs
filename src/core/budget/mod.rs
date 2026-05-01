//! Budget Management System
//!
//! This module provides comprehensive budget management for the LiteLLM-RS gateway,
//! including budget tracking, alerting, and middleware for request interception.
//!
//! ## Features
//!
//! - **Budget Types**: Flexible budget scopes (user, team, API key, provider, model, global)
//! - **Tracking**: Lock-free concurrent budget tracking using DashMap
//! - **Management**: CRUD operations for budget configuration
//! - **Middleware**: Actix-web middleware for request budget checking
//! - **Alerts**: Webhook-based alerting for budget thresholds
//!
//! ## Usage
//!
//! ```rust,ignore
//! use litellm_rs::core::budget::{BudgetManager, BudgetScope, BudgetConfig};
//!
//! // Create a budget manager
//! let manager = BudgetManager::new();
//!
//! // Create a budget for a user
//! let config = BudgetConfig::new("User Budget", 100.0);
//! let budget = manager.create_budget(
//!     BudgetScope::User("user-123".to_string()),
//!     config
//! ).await?;
//!
//! // Record spend
//! manager.record_spend(&BudgetScope::User("user-123".to_string()), 5.50).await;
//!
//! // Check remaining budget
//! let remaining = manager.get_remaining(&BudgetScope::User("user-123".to_string()));
//! ```
//!
//! ## Middleware Integration
//!
//! ```rust,ignore
//! use litellm_rs::core::budget::{BudgetCheckMiddleware, BudgetManager};
//! use actix_web::{App, web};
//! use std::sync::Arc;
//!
//! let manager = Arc::new(BudgetManager::new());
//! let middleware = BudgetCheckMiddleware::new(Arc::clone(&manager));
//!
//! App::new()
//!     .wrap(middleware)
//!     .app_data(web::Data::new(manager))
//! ```
//!
//! ## Alert Configuration
//!
//! ```rust,ignore
//! use litellm_rs::core::budget::{BudgetAlertManager, WebhookConfig, AlertSeverity};
//!
//! let alert_manager = BudgetAlertManager::new();
//!
//! // Add a webhook for critical alerts
//! alert_manager.add_webhook(WebhookConfig {
//!     url: "https://example.com/webhook".to_string(),
//!     severities: vec![AlertSeverity::Critical],
//!     ..Default::default()
//! }).await;
//! ```

mod alerts;
mod manager;
#[cfg(feature = "gateway")]
mod middleware;
mod provider_limits;
mod tracker;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use alerts::{AlertConfig, AlertStats, BudgetAlertManager, WebhookConfig};
pub use manager::{BudgetManager, BudgetManagerConfig, BudgetSummary};
#[cfg(feature = "gateway")]
pub use middleware::{
    BudgetCheckMiddleware, BudgetCheckMiddlewareService, BudgetMiddleware, BudgetMiddlewareService,
    BudgetRecorder, BudgetRecorderExt,
};
pub use provider_limits::{
    BudgetLimitKind, BudgetLimitSnapshot, BudgetPersistenceEvent, BudgetPersistenceSender,
    ModelBudgetManager, ModelLimitConfig, ProviderBudgetManager, ProviderLimitConfig,
    UnifiedBudgetLimits,
};
pub use tracker::{BudgetTracker, SpendResult};
pub use types::{
    AlertSeverity, Budget, BudgetAlert, BudgetAlertType, BudgetCheckResult, BudgetConfig,
    BudgetScope, BudgetStatus, Currency, ModelBudget, ModelUsageStats, ProviderBudget,
    ProviderUsageStats, ResetPeriod,
};

use std::sync::Arc;

/// Initialize a complete budget system with default configuration
///
/// Returns a tuple of (BudgetManager, BudgetAlertManager) that can be used
/// together for complete budget management with alerts.
pub fn init_budget_system() -> (Arc<BudgetManager>, Arc<BudgetAlertManager>) {
    let manager = Arc::new(BudgetManager::new());
    let alert_manager = Arc::new(BudgetAlertManager::new());

    (manager, alert_manager)
}

/// Initialize a budget system with custom configuration
pub fn init_budget_system_with_config(
    manager_config: BudgetManagerConfig,
    alert_config: AlertConfig,
) -> (Arc<BudgetManager>, Arc<BudgetAlertManager>) {
    let manager = Arc::new(BudgetManager::with_config(manager_config));
    let alert_manager = Arc::new(BudgetAlertManager::with_config(alert_config));

    (manager, alert_manager)
}

/// Global budget manager singleton (optional usage pattern)
static GLOBAL_BUDGET_MANAGER: std::sync::OnceLock<Arc<BudgetManager>> = std::sync::OnceLock::new();

/// Initialize the global budget manager
pub fn init_global_budget_manager(config: BudgetManagerConfig) {
    let manager = Arc::new(BudgetManager::with_config(config));
    let _ = GLOBAL_BUDGET_MANAGER.set(manager);
}

/// Get the global budget manager
pub fn get_global_budget_manager() -> Option<Arc<BudgetManager>> {
    GLOBAL_BUDGET_MANAGER.get().cloned()
}
