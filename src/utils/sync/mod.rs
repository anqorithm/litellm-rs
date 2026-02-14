//! Concurrent-safe container module for the LiteLLM Gateway
//!
//! This module provides thread-safe data structures for concurrent access,
//! inspired by the Crush csync package design. All containers are designed
//! to be used independently and follow Rust best practices.
//!
//! ## Available Containers
//!
//! - [`ConcurrentMap`] - A concurrent-safe HashMap based on DashMap
//! - [`ConcurrentVec`] - A concurrent-safe Vec with fine-grained locking
//! - [`AtomicValue`] - A concurrent-safe single value container using arc-swap
//! - [`VersionedMap`] - A Map with version tracking for optimistic locking
//!
//! ## Example Usage
//!
//! ```rust
//! use litellm_rs::utils::sync::{ConcurrentMap, ConcurrentVec, AtomicValue, VersionedMap};
//!
//! // ConcurrentMap
//! let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
//! map.insert("key".to_string(), 42);
//! assert_eq!(map.get(&"key".to_string()), Some(42));
//!
//! // ConcurrentVec
//! let vec: ConcurrentVec<i32> = ConcurrentVec::new();
//! vec.push(1);
//! vec.push(2);
//! assert_eq!(vec.len(), 2);
//!
//! // AtomicValue
//! let value: AtomicValue<String> = AtomicValue::new("initial".to_string());
//! value.store("updated".to_string());
//! assert_eq!(value.load().as_ref(), "updated");
//!
//! // VersionedMap with optimistic locking
//! let vmap: VersionedMap<String, i32> = VersionedMap::new();
//! vmap.insert("key".to_string(), 100);
//! let (val, version) = vmap.get_versioned(&"key".to_string()).unwrap();
//! assert!(vmap.compare_and_swap(&"key".to_string(), 200, version).is_ok());
//! ```

mod atomic_value;
mod concurrent_map;
mod concurrent_vec;
mod versioned_map;

pub use atomic_value::AtomicValue;
pub use concurrent_map::ConcurrentMap;
pub use concurrent_vec::ConcurrentVec;
pub use versioned_map::{VersionError, VersionedEntry, VersionedMap};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// Test that all containers can be used together in a concurrent scenario
    #[test]
    fn test_all_containers_concurrent() {
        let map: Arc<ConcurrentMap<String, i32>> = Arc::new(ConcurrentMap::new());
        let vec: Arc<ConcurrentVec<i32>> = Arc::new(ConcurrentVec::new());
        let value: Arc<AtomicValue<i32>> = Arc::new(AtomicValue::new(0));
        let vmap: Arc<VersionedMap<String, i32>> = Arc::new(VersionedMap::new());

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let map = Arc::clone(&map);
                let vec = Arc::clone(&vec);
                let value = Arc::clone(&value);
                let vmap = Arc::clone(&vmap);

                thread::spawn(move || {
                    // Use ConcurrentMap
                    map.insert(format!("key_{}", i), i);

                    // Use ConcurrentVec
                    vec.push(i);

                    // Use AtomicValue (store is atomic, update is not)
                    value.store(i);

                    // Use VersionedMap
                    vmap.insert(format!("vkey_{}", i), i * 10);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 4);
        assert_eq!(vec.len(), 4);
        // AtomicValue.store is atomic, final value should be one of 0, 1, 2, 3
        let final_value = *value.load();
        assert!((0..4).contains(&final_value));
        assert_eq!(vmap.len(), 4);
    }
}
