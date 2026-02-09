//! Dependency injection utilities for better service management
//!
//! This module provides a lightweight dependency injection system
//! that follows Rust best practices and improves testability.

use crate::utils::error::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// A trait for services that can be injected
pub trait Injectable: Send + Sync + 'static {
    /// Get the type name for debugging
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Automatic implementation for types that meet the requirements
impl<T> Injectable for T where T: Send + Sync + 'static {}

/// A service container for dependency injection
#[derive(Default)]
pub struct ServiceContainer {
    services: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    singletons: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl ServiceContainer {
    /// Create a new service container
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a service instance
    pub fn register<T>(&self, service: T) -> Result<()>
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();
        let service = Arc::new(service);

        let mut services = self.services.write();
        if services.contains_key(&type_id) {
            return Err(GatewayError::Config(format!(
                "Service {} is already registered",
                std::any::type_name::<T>()
            )));
        }

        services.insert(type_id, service);
        Ok(())
    }

    /// Register a singleton service
    pub fn register_singleton<T>(&self, service: T) -> Result<()>
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();
        let service = Arc::new(service);

        let mut singletons = self.singletons.write();
        if singletons.contains_key(&type_id) {
            return Err(GatewayError::Config(format!(
                "Singleton {} is already registered",
                std::any::type_name::<T>()
            )));
        }

        singletons.insert(type_id, service);
        Ok(())
    }

    /// Register a service factory
    pub fn register_factory<T, F>(&self, factory: F) -> Result<()>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let service = factory();
        self.register(service)
    }

    /// Get a service instance
    pub fn get<T>(&self) -> Result<Arc<T>>
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();

        // First check singletons
        {
            let singletons = self.singletons.read();
            if let Some(service) = singletons.get(&type_id) {
                return service.clone().downcast::<T>().map_err(|_| {
                    GatewayError::Internal(format!(
                        "Failed to downcast singleton service {}",
                        std::any::type_name::<T>()
                    ))
                });
            }
        }

        // Then check regular services
        {
            let services = self.services.read();
            if let Some(service) = services.get(&type_id) {
                return service.clone().downcast::<T>().map_err(|_| {
                    GatewayError::Internal(format!(
                        "Failed to downcast service {}",
                        std::any::type_name::<T>()
                    ))
                });
            }
        }

        Err(GatewayError::Config(format!(
            "Service {} is not registered",
            std::any::type_name::<T>()
        )))
    }

    /// Try to get a service instance (returns None if not found)
    pub fn try_get<T>(&self) -> Option<Arc<T>>
    where
        T: Injectable,
    {
        self.get().ok()
    }

    /// Check if a service is registered
    pub fn contains<T>(&self) -> bool
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();
        let singletons = self.singletons.read();
        let services = self.services.read();

        singletons.contains_key(&type_id) || services.contains_key(&type_id)
    }

    /// Remove a service
    pub fn remove<T>(&self) -> Result<()>
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();

        {
            let mut singletons = self.singletons.write();
            if singletons.remove(&type_id).is_some() {
                return Ok(());
            }
        }

        {
            let mut services = self.services.write();
            if services.remove(&type_id).is_some() {
                return Ok(());
            }
        }

        Err(GatewayError::Config(format!(
            "Service {} is not registered",
            std::any::type_name::<T>()
        )))
    }

    /// Clear all services
    pub fn clear(&self) {
        self.services.write().clear();
        self.singletons.write().clear();
    }

    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        let services = self.services.read();
        let singletons = self.singletons.read();
        services.len() + singletons.len()
    }
}

/// A builder for setting up service dependencies
pub struct ServiceBuilder {
    container: ServiceContainer,
}

impl ServiceBuilder {
    /// Create a new service builder
    pub fn new() -> Self {
        Self {
            container: ServiceContainer::new(),
        }
    }

    /// Add a service
    pub fn add_service<T>(self, service: T) -> Result<Self>
    where
        T: Injectable,
    {
        self.container.register(service)?;
        Ok(self)
    }

    /// Add a singleton service
    pub fn add_singleton<T>(self, service: T) -> Result<Self>
    where
        T: Injectable,
    {
        self.container.register_singleton(service)?;
        Ok(self)
    }

    /// Add a service factory
    pub fn add_factory<T, F>(self, factory: F) -> Result<Self>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.container.register_factory(factory)?;
        Ok(self)
    }

    /// Build the service container
    pub fn build(self) -> ServiceContainer {
        self.container
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A trait for types that can be configured with dependencies
pub trait WithDependencies {
    /// Configure the type with dependencies from the container
    fn with_dependencies(self, container: &ServiceContainer) -> Result<Self>
    where
        Self: Sized;
}

// Note: Macros removed for simplicity - use direct method calls instead

/// Global service container for application-wide services
use std::sync::LazyLock;

static GLOBAL_CONTAINER: LazyLock<ServiceContainer> = LazyLock::new(ServiceContainer::new);

/// Get the global service container
pub fn global_container() -> &'static ServiceContainer {
    &GLOBAL_CONTAINER
}

/// Register a service globally
pub fn register_global<T>(service: T) -> Result<()>
where
    T: Injectable,
{
    global_container().register(service)
}

/// Register a singleton service globally
pub fn register_global_singleton<T>(service: T) -> Result<()>
where
    T: Injectable,
{
    global_container().register_singleton(service)
}

/// Get a service from the global container
pub fn get_global<T>() -> Result<Arc<T>>
where
    T: Injectable,
{
    global_container().get()
}

#[cfg(test)]
#[path = "di_tests.rs"]
mod tests;
