//! Shared state management utilities
//!
//! This module provides utilities for efficient shared state management,
//! reducing the need for excessive Arc wrapping while maintaining thread safety.

#![allow(dead_code)] // Tool module - functions may be used in the future

use std::sync::OnceLock;
use parking_lot::RwLock;
use std::sync::Arc;

/// A trait for types that can be shared efficiently across threads
pub trait SharedResource: Send + Sync + 'static {}

/// Automatic implementation for types that meet the requirements
impl<T> SharedResource for T where T: Send + Sync + 'static {}

/// A wrapper for shared resources that provides efficient access patterns
#[derive(Debug)]
pub struct Shared<T> {
    inner: Arc<T>,
}

impl<T> Shared<T> {
    /// Create a new shared resource
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Get an Arc clone for cases where ownership is needed
    pub fn arc(&self) -> Arc<T> {
        Arc::clone(&self.inner)
    }

    /// Get the strong reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> std::ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A shared mutable resource with read-write lock
#[derive(Debug)]
pub struct SharedMut<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> SharedMut<T> {
    /// Create a new shared mutable resource
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Get a read lock
    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.inner.read()
    }

    /// Get a write lock
    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.inner.write()
    }

    /// Try to get a read lock without blocking
    pub fn try_read(&self) -> Option<parking_lot::RwLockReadGuard<'_, T>> {
        self.inner.try_read()
    }

    /// Try to get a write lock without blocking
    pub fn try_write(&self) -> Option<parking_lot::RwLockWriteGuard<'_, T>> {
        self.inner.try_write()
    }

    /// Get an Arc clone for cases where ownership is needed
    pub fn arc(&self) -> Arc<RwLock<T>> {
        Arc::clone(&self.inner)
    }
}

impl<T> Clone for SharedMut<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// A global shared resource that can be initialized once
pub struct GlobalShared<T> {
    cell: OnceLock<Shared<T>>,
}

impl<T> GlobalShared<T> {
    /// Create a new global shared resource
    pub const fn new() -> Self {
        Self {
            cell: OnceLock::new(),
        }
    }

    /// Initialize the global resource (can only be called once)
    pub fn init(&self, value: T) -> Result<(), T> {
        self.cell.set(Shared::new(value)).map_err(|shared| {
            // Extract the value from the Shared wrapper
            match Arc::try_unwrap(shared.inner) {
                Ok(value) => value,
                Err(_) => panic!("Failed to extract value from shared resource"),
            }
        })
    }

    /// Get the global resource (panics if not initialized)
    pub fn get(&self) -> &Shared<T> {
        self.cell.get().expect("Global resource not initialized")
    }

    /// Try to get the global resource (returns None if not initialized)
    pub fn try_get(&self) -> Option<&Shared<T>> {
        self.cell.get()
    }
}

impl<T> Default for GlobalShared<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to create a global shared resource
#[macro_export]
macro_rules! global_shared {
    ($name:ident: $type:ty) => {
        static $name: $crate::utils::shared_state::GlobalShared<$type> =
            $crate::utils::shared_state::GlobalShared::new();
    };
}

/// A builder pattern for creating shared resources with dependencies
pub struct SharedBuilder<T> {
    value: Option<T>,
}

impl<T> SharedBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self { value: None }
    }

    /// Set the value
    pub fn with_value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    /// Build the shared resource
    pub fn build(self) -> Option<Shared<T>> {
        self.value.map(Shared::new)
    }

    /// Build the shared resource or panic
    pub fn build_or_panic(self, msg: &str) -> Shared<T> {
        self.build().expect(msg)
    }
}

impl<T> Default for SharedBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with shared resources
pub mod utils {
    use super::*;

    /// Create a shared resource from a value
    pub fn share<T>(value: T) -> Shared<T> {
        Shared::new(value)
    }

    /// Create a shared mutable resource from a value
    pub fn share_mut<T>(value: T) -> SharedMut<T> {
        SharedMut::new(value)
    }

    /// Convert an Arc to a Shared (zero-cost if T is already Arc-wrapped)
    pub fn from_arc<T>(arc: Arc<T>) -> Shared<T> {
        Shared { inner: arc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_resource() {
        let shared = Shared::new(42);
        assert_eq!(*shared.get(), 42);
        assert_eq!(*shared, 42);

        let cloned = shared.clone();
        assert_eq!(*cloned.get(), 42);
        assert_eq!(shared.strong_count(), 2);
    }

    #[test]
    fn test_shared_mut_resource() {
        let shared = SharedMut::new(42);

        {
            let read_guard = shared.read();
            assert_eq!(*read_guard, 42);
        }

        {
            let mut write_guard = shared.write();
            *write_guard = 100;
        }

        {
            let read_guard = shared.read();
            assert_eq!(*read_guard, 100);
        }
    }

    #[test]
    fn test_global_shared() {
        let global: GlobalShared<i32> = GlobalShared::new();

        assert!(global.try_get().is_none());

        global.init(42).unwrap();
        assert_eq!(**global.get(), 42);

        // Second init should fail
        assert!(global.init(100).is_err());
    }

    #[test]
    fn test_shared_builder() {
        let shared = SharedBuilder::new().with_value(42).build().unwrap();

        assert_eq!(*shared.get(), 42);
    }
}
