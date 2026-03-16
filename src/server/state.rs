//! Application state shared across HTTP handlers
//!
//! This module provides the AppState struct and its implementations.

use crate::config::Config;
use crate::core::budget::UnifiedBudgetLimits;
use crate::services::pricing::PricingService;
use std::sync::Arc;

/// HTTP server state shared across handlers
///
/// This struct contains shared resources that need to be accessed across
/// multiple request handlers. All fields are wrapped in Arc for efficient
/// sharing across threads.
#[derive(Clone)]
pub struct AppState {
    /// Gateway configuration (shared read-only)
    pub config: Arc<Config>,
    /// Authentication system
    pub auth: Arc<crate::auth::AuthSystem>,
    /// Unified router (new UnifiedRouter implementation)
    pub unified_router: Arc<crate::core::router::UnifiedRouter>,
    /// Storage layer
    pub storage: Arc<crate::storage::StorageLayer>,
    /// Unified pricing service
    pub pricing: Arc<PricingService>,
    /// Budget limits for provider and model cost tracking
    pub budget_limits: Arc<UnifiedBudgetLimits>,
}

impl AppState {
    /// Create a new AppState with unified router
    pub fn new_with_unified_router(
        config: Config,
        auth: crate::auth::AuthSystem,
        unified_router: crate::core::router::UnifiedRouter,
        storage: crate::storage::StorageLayer,
        pricing: Arc<PricingService>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            unified_router: Arc::new(unified_router),
            storage: Arc::new(storage),
            pricing,
            budget_limits: Arc::new(UnifiedBudgetLimits::new()),
        }
    }

    /// Get gateway configuration
    #[allow(dead_code)] // May be used by handlers
    pub fn config(&self) -> &Config {
        &self.config
    }
}
