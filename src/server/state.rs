//! Application state shared across HTTP handlers
//!
//! This module provides the AppState struct and its implementations.

use crate::config::Config;
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
    /// Request router (legacy ProviderRegistry)
    pub router: Arc<crate::core::providers::ProviderRegistry>,
    /// Unified router (new UnifiedRouter implementation)
    pub unified_router: Option<Arc<crate::core::router::UnifiedRouter>>,
    /// Storage layer
    pub storage: Arc<crate::storage::StorageLayer>,
    /// Unified pricing service
    pub pricing: Arc<PricingService>,
}

impl AppState {
    /// Create a new AppState with shared resources
    pub fn new(
        config: Config,
        auth: crate::auth::AuthSystem,
        router: crate::core::providers::ProviderRegistry,
        storage: crate::storage::StorageLayer,
        pricing: Arc<PricingService>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            router: Arc::new(router),
            unified_router: None,
            storage: Arc::new(storage),
            pricing,
        }
    }

    /// Create a new AppState with unified router
    pub fn new_with_unified_router(
        config: Config,
        auth: crate::auth::AuthSystem,
        router: crate::core::providers::ProviderRegistry,
        unified_router: crate::core::router::UnifiedRouter,
        storage: crate::storage::StorageLayer,
        pricing: Arc<PricingService>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            router: Arc::new(router),
            unified_router: Some(Arc::new(unified_router)),
            storage: Arc::new(storage),
            pricing,
        }
    }
}
