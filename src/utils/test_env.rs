//! Test-only helpers for environment variable mutation.
//!
//! Rust now marks env var mutation APIs as `unsafe` due to process-global
//! mutation concerns. Centralize those calls here and serialize access.

use std::cell::Cell;
use std::sync::{LazyLock, Mutex, MutexGuard};

static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
thread_local! {
    static ENV_LOCK_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Reentrant lock guard for process-global environment mutation in tests.
pub struct EnvLockGuard {
    _guard: Option<MutexGuard<'static, ()>>,
}

impl Drop for EnvLockGuard {
    fn drop(&mut self) {
        ENV_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

/// Acquire a global lock for environment-variable mutation in tests.
///
/// This lock is reentrant within the same thread so callers can compose helpers
/// (e.g. hold a lock and still call `set_var`/`remove_var` safely).
pub fn lock() -> EnvLockGuard {
    let should_lock_mutex = ENV_LOCK_DEPTH.with(|depth| {
        let current = depth.get();
        depth.set(current + 1);
        current == 0
    });

    let guard = if should_lock_mutex {
        Some(
            ENV_MUTEX
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    } else {
        None
    };

    EnvLockGuard { _guard: guard }
}

/// Set an environment variable for tests.
pub fn set_var(key: &str, value: &str) {
    let _guard = lock();
    unsafe { std::env::set_var(key, value) };
}

/// Remove an environment variable for tests.
pub fn remove_var(key: &str) {
    let _guard = lock();
    unsafe { std::env::remove_var(key) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_key() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("LITELLM_TEST_ENV_{}", nanos)
    }

    #[test]
    fn set_and_remove_var_are_serialized() {
        let key = unique_key();

        set_var(&key, "value");
        assert_eq!(std::env::var(&key).ok().as_deref(), Some("value"));

        remove_var(&key);
        assert!(std::env::var(&key).is_err());
    }

    #[test]
    fn lock_is_reentrant_in_same_thread() {
        let key = unique_key();

        let _guard = lock();
        set_var(&key, "nested");
        assert_eq!(std::env::var(&key).ok().as_deref(), Some("nested"));
        remove_var(&key);
    }
}
