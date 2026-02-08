//! Test-only helpers for environment variable mutation.
//!
//! Rust now marks env var mutation APIs as `unsafe` due to process-global
//! mutation concerns. Centralize those calls here and serialize access.

use std::sync::{LazyLock, Mutex, MutexGuard};

static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Acquire a global lock for environment-variable mutation in tests.
pub fn lock() -> MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Set an environment variable for tests.
pub fn set_var(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) };
}

/// Remove an environment variable for tests.
pub fn remove_var(key: &str) {
    unsafe { std::env::remove_var(key) };
}

