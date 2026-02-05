//! Versioned Map with optimistic locking support
//!
//! Provides a concurrent-safe Map that tracks versions for each entry,
//! enabling optimistic locking patterns for concurrent updates.

use dashmap::DashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Error type for versioned map operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionError {
    /// The key was not found in the map.
    KeyNotFound,
    /// The version did not match (concurrent modification detected).
    VersionMismatch {
        /// The expected version.
        expected: u64,
        /// The actual current version.
        actual: u64,
    },
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::KeyNotFound => write!(f, "Key not found"),
            VersionError::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "Version mismatch: expected {}, actual {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for VersionError {}

/// An entry in the versioned map containing the value and its version.
#[derive(Debug, Clone)]
pub struct VersionedEntry<V> {
    /// The stored value.
    pub value: V,
    /// The version number of this entry.
    pub version: u64,
}

impl<V> VersionedEntry<V> {
    /// Creates a new versioned entry.
    pub fn new(value: V, version: u64) -> Self {
        Self { value, version }
    }
}

/// A concurrent-safe Map with version tracking for optimistic locking.
///
/// Each entry in the map has an associated version number that is incremented
/// on every update. This enables optimistic locking patterns where you can
/// read a value with its version, perform some computation, and then update
/// only if the version hasn't changed.
///
/// # Type Parameters
///
/// * `K` - The key type, must implement `Eq + Hash`
/// * `V` - The value type, must implement `Clone`
///
/// # Example
///
/// ```rust
/// use litellm_rs::utils::sync::VersionedMap;
///
/// let map: VersionedMap<String, i32> = VersionedMap::new();
/// map.insert("key".to_string(), 100);
///
/// // Get value with version
/// let (value, version) = map.get_versioned(&"key".to_string()).unwrap();
/// assert_eq!(value, 100);
///
/// // Update only if version matches (optimistic locking)
/// assert!(map.compare_and_swap(&"key".to_string(), 200, version).is_ok());
///
/// // Subsequent update with old version fails
/// assert!(map.compare_and_swap(&"key".to_string(), 300, version).is_err());
/// ```
#[derive(Debug)]
pub struct VersionedMap<K, V>
where
    K: Eq + Hash,
{
    inner: Arc<DashMap<K, VersionedEntry<V>>>,
    global_version: AtomicU64,
}

impl<K, V> Default for VersionedMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for VersionedMap<K, V>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            global_version: AtomicU64::new(self.global_version.load(Ordering::SeqCst)),
        }
    }
}

impl<K, V> VersionedMap<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new empty `VersionedMap`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            global_version: AtomicU64::new(0),
        }
    }

    /// Creates a new `VersionedMap` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the map
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(DashMap::with_capacity(capacity)),
            global_version: AtomicU64::new(0),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("a".to_string(), 1);
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if the map contains the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("key".to_string(), 1);
    /// assert!(map.contains_key(&"key".to_string()));
    /// ```
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Removes a key from the map.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Returns
    ///
    /// The versioned entry if the key was present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("key".to_string(), 42);
    /// let entry = map.remove(&"key".to_string());
    /// assert!(entry.is_some());
    /// assert_eq!(entry.unwrap().value, 42);
    /// ```
    pub fn remove(&self, key: &K) -> Option<VersionedEntry<V>> {
        self.inner.remove(key).map(|(_, v)| v)
    }

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Returns the current global version counter.
    ///
    /// This is incremented on every insert or update operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// let v1 = map.global_version();
    /// map.insert("key".to_string(), 1);
    /// let v2 = map.global_version();
    /// assert!(v2 > v1);
    /// ```
    pub fn global_version(&self) -> u64 {
        self.global_version.load(Ordering::SeqCst)
    }

    /// Returns all keys in the map.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("a".to_string(), 1);
    /// let keys = map.keys();
    /// assert_eq!(keys.len(), 1);
    /// ```
    pub fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.inner.iter().map(|r| r.key().clone()).collect()
    }

    fn next_version(&self) -> u64 {
        self.global_version.fetch_add(1, Ordering::SeqCst) + 1
    }
}

impl<K, V> VersionedMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Inserts a key-value pair into the map.
    ///
    /// The entry is assigned a new version number.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// The version number assigned to this entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// let version = map.insert("key".to_string(), 42);
    /// assert!(version > 0);
    /// ```
    pub fn insert(&self, key: K, value: V) -> u64 {
        let version = self.next_version();
        self.inner.insert(key, VersionedEntry::new(value, version));
        version
    }

    /// Gets a clone of the value for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A clone of the value if present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("key".to_string(), 42);
    /// assert_eq!(map.get(&"key".to_string()), Some(42));
    /// ```
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|r| r.value.clone())
    }

    /// Gets a clone of the value along with its version.
    ///
    /// This is the primary method for optimistic locking - read the value
    /// and version, then use `compare_and_swap` to update.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A tuple of (value, version) if present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("key".to_string(), 42);
    /// let (value, version) = map.get_versioned(&"key".to_string()).unwrap();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get_versioned(&self, key: &K) -> Option<(V, u64)> {
        self.inner.get(key).map(|r| (r.value.clone(), r.version))
    }

    /// Gets the version of an entry without cloning the value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// The version if the key is present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// let v1 = map.insert("key".to_string(), 42);
    /// assert_eq!(map.get_version(&"key".to_string()), Some(v1));
    /// ```
    pub fn get_version(&self, key: &K) -> Option<u64> {
        self.inner.get(key).map(|r| r.version)
    }

    /// Atomically updates a value only if the version matches.
    ///
    /// This is the core optimistic locking operation. It will only succeed
    /// if the current version matches the expected version, preventing
    /// lost updates in concurrent scenarios.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to update
    /// * `new_value` - The new value to set
    /// * `expected_version` - The expected current version
    ///
    /// # Returns
    ///
    /// `Ok(new_version)` if the update succeeded, `Err(VersionError)` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("key".to_string(), 100);
    ///
    /// let (_, version) = map.get_versioned(&"key".to_string()).unwrap();
    ///
    /// // This succeeds
    /// assert!(map.compare_and_swap(&"key".to_string(), 200, version).is_ok());
    ///
    /// // This fails because version changed
    /// assert!(map.compare_and_swap(&"key".to_string(), 300, version).is_err());
    /// ```
    pub fn compare_and_swap(
        &self,
        key: &K,
        new_value: V,
        expected_version: u64,
    ) -> Result<u64, VersionError> {
        let mut entry = self.inner.get_mut(key).ok_or(VersionError::KeyNotFound)?;

        if entry.version != expected_version {
            return Err(VersionError::VersionMismatch {
                expected: expected_version,
                actual: entry.version,
            });
        }

        let new_version = self.next_version();
        entry.value = new_value;
        entry.version = new_version;
        Ok(new_version)
    }

    /// Updates a value using a closure, with automatic retry on version conflict.
    ///
    /// This method will retry the update if a concurrent modification is detected,
    /// up to the specified maximum number of retries. If it still fails due
    /// to contention, it falls back to a locked update to guarantee progress.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to update
    /// * `f` - A closure that takes the current value and returns the new value
    /// * `max_retries` - Maximum number of retry attempts
    ///
    /// # Returns
    ///
    /// `Ok((new_value, new_version))` if the update succeeded, `Err(VersionError)` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("counter".to_string(), 0);
    ///
    /// let result = map.update_with_retry(&"counter".to_string(), |v| v + 1, 3);
    /// assert!(result.is_ok());
    /// assert_eq!(map.get(&"counter".to_string()), Some(1));
    /// ```
    pub fn update_with_retry<F>(
        &self,
        key: &K,
        f: F,
        max_retries: usize,
    ) -> Result<(V, u64), VersionError>
    where
        F: Fn(V) -> V,
    {
        for _ in 0..=max_retries {
            let (current_value, current_version) =
                self.get_versioned(key).ok_or(VersionError::KeyNotFound)?;

            let new_value = f(current_value);

            match self.compare_and_swap(key, new_value.clone(), current_version) {
                Ok(new_version) => return Ok((new_value, new_version)),
                Err(VersionError::VersionMismatch { .. }) => continue,
                Err(e) => return Err(e),
            }
        }

        // Final attempt failed due to contention; fall back to a locked update.
        let mut entry = self.inner.get_mut(key).ok_or(VersionError::KeyNotFound)?;
        let new_value = f(entry.value.clone());
        let new_version = self.next_version();
        entry.value = new_value.clone();
        entry.version = new_version;
        Ok((new_value, new_version))
    }

    /// Gets or inserts a value, returning the value and its version.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up or insert
    /// * `default` - The default value to insert if key is not present
    ///
    /// # Returns
    ///
    /// A tuple of (value, version).
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// let (value, _) = map.get_or_insert("key".to_string(), 42);
    /// assert_eq!(value, 42);
    /// ```
    pub fn get_or_insert(&self, key: K, default: V) -> (V, u64) {
        let version = self.next_version();
        let entry = self
            .inner
            .entry(key)
            .or_insert_with(|| VersionedEntry::new(default, version));
        (entry.value.clone(), entry.version)
    }

    /// Returns all entries as a vector of (key, value, version) tuples.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::VersionedMap;
    ///
    /// let map: VersionedMap<String, i32> = VersionedMap::new();
    /// map.insert("a".to_string(), 1);
    /// let entries = map.entries();
    /// assert_eq!(entries.len(), 1);
    /// ```
    pub fn entries(&self) -> Vec<(K, V, u64)>
    where
        K: Clone,
    {
        self.inner
            .iter()
            .map(|r| (r.key().clone(), r.value.clone(), r.version))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_and_default() {
        let map1: VersionedMap<String, i32> = VersionedMap::new();
        let map2: VersionedMap<String, i32> = VersionedMap::default();
        assert!(map1.is_empty());
        assert!(map2.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let version = map.insert("key".to_string(), 42);
        assert!(version > 0);
        assert_eq!(map.get(&"key".to_string()), Some(42));
    }

    #[test]
    fn test_get_versioned() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let v1 = map.insert("key".to_string(), 42);
        let (value, version) = map.get_versioned(&"key".to_string()).unwrap();
        assert_eq!(value, 42);
        assert_eq!(version, v1);
    }

    #[test]
    fn test_get_version() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let v1 = map.insert("key".to_string(), 42);
        assert_eq!(map.get_version(&"key".to_string()), Some(v1));
        assert_eq!(map.get_version(&"nonexistent".to_string()), None);
    }

    #[test]
    fn test_compare_and_swap_success() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("key".to_string(), 100);

        let (_, version) = map.get_versioned(&"key".to_string()).unwrap();
        let result = map.compare_and_swap(&"key".to_string(), 200, version);
        assert!(result.is_ok());
        assert_eq!(map.get(&"key".to_string()), Some(200));
    }

    #[test]
    fn test_compare_and_swap_version_mismatch() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("key".to_string(), 100);

        let (_, version) = map.get_versioned(&"key".to_string()).unwrap();

        // Update the value, changing the version
        map.insert("key".to_string(), 150);

        // Now CAS should fail
        let result = map.compare_and_swap(&"key".to_string(), 200, version);
        assert!(matches!(result, Err(VersionError::VersionMismatch { .. })));
    }

    #[test]
    fn test_compare_and_swap_key_not_found() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let result = map.compare_and_swap(&"nonexistent".to_string(), 100, 1);
        assert!(matches!(result, Err(VersionError::KeyNotFound)));
    }

    #[test]
    fn test_update_with_retry() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("counter".to_string(), 0);

        let result = map.update_with_retry(&"counter".to_string(), |v| v + 1, 3);
        assert!(result.is_ok());
        assert_eq!(map.get(&"counter".to_string()), Some(1));
    }

    #[test]
    fn test_remove() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("key".to_string(), 42);
        let entry = map.remove(&"key".to_string());
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().value, 42);
        assert!(map.is_empty());
    }

    #[test]
    fn test_clear() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_len_and_is_empty() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.insert("a".to_string(), 1);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_contains_key() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("key".to_string(), 42);
        assert!(map.contains_key(&"key".to_string()));
        assert!(!map.contains_key(&"other".to_string()));
    }

    #[test]
    fn test_global_version() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let v1 = map.global_version();
        map.insert("a".to_string(), 1);
        let v2 = map.global_version();
        map.insert("b".to_string(), 2);
        let v3 = map.global_version();

        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn test_get_or_insert() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        let (value1, _) = map.get_or_insert("key".to_string(), 42);
        assert_eq!(value1, 42);

        let (value2, _) = map.get_or_insert("key".to_string(), 100);
        assert_eq!(value2, 42); // Original value preserved
    }

    #[test]
    fn test_keys_and_entries() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        let keys = map.keys();
        assert_eq!(keys.len(), 2);

        let entries = map.entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_version_error_display() {
        let err1 = VersionError::KeyNotFound;
        assert_eq!(format!("{}", err1), "Key not found");

        let err2 = VersionError::VersionMismatch {
            expected: 1,
            actual: 2,
        };
        assert_eq!(
            format!("{}", err2),
            "Version mismatch: expected 1, actual 2"
        );
    }

    #[test]
    fn test_clone() {
        let map1: VersionedMap<String, i32> = VersionedMap::new();
        map1.insert("key".to_string(), 42);
        let map2 = map1.clone();

        // Both maps share the same underlying data
        assert_eq!(map2.get(&"key".to_string()), Some(42));
        map2.insert("new".to_string(), 100);
        assert_eq!(map1.get(&"new".to_string()), Some(100));
    }

    #[test]
    fn test_concurrent_inserts() {
        let map: Arc<VersionedMap<i32, i32>> = Arc::new(VersionedMap::new());
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let map = Arc::clone(&map);
                thread::spawn(move || {
                    for j in 0..100 {
                        map.insert(i * 100 + j, j);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 1000);
    }

    #[test]
    fn test_concurrent_compare_and_swap() {
        let map: Arc<VersionedMap<String, i32>> = Arc::new(VersionedMap::new());
        map.insert("counter".to_string(), 0);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let map = Arc::clone(&map);
                thread::spawn(move || {
                    for _ in 0..100 {
                        // Use update_with_retry for safe concurrent updates
                        let _ = map.update_with_retry(&"counter".to_string(), |v| v + 1, 10);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // With retry, all updates should succeed
        assert_eq!(map.get(&"counter".to_string()), Some(1000));
    }

    #[test]
    fn test_optimistic_locking_pattern() {
        let map: VersionedMap<String, i32> = VersionedMap::new();
        map.insert("balance".to_string(), 1000);

        // Simulate a transaction: read, compute, write
        let (balance, version) = map.get_versioned(&"balance".to_string()).unwrap();
        let new_balance = balance - 100; // Withdraw 100

        // Commit the transaction
        let result = map.compare_and_swap(&"balance".to_string(), new_balance, version);
        assert!(result.is_ok());
        assert_eq!(map.get(&"balance".to_string()), Some(900));
    }
}
