//! Concurrent-safe Map implementation based on DashMap
//!
//! Provides a thread-safe HashMap with fine-grained locking for high-performance
//! concurrent access. This is a thin wrapper around DashMap with additional
//! convenience methods.

use dashmap::DashMap;
use std::hash::Hash;
use std::sync::Arc;

/// A concurrent-safe HashMap based on DashMap.
///
/// This container provides thread-safe access to a key-value store with
/// fine-grained locking, allowing multiple readers and writers to access
/// different keys simultaneously.
///
/// # Type Parameters
///
/// * `K` - The key type, must implement `Eq + Hash`
/// * `V` - The value type, must implement `Clone`
///
/// # Example
///
/// ```rust
/// use litellm_rs::utils::sync::ConcurrentMap;
///
/// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
/// map.insert("key".to_string(), 42);
/// assert_eq!(map.get(&"key".to_string()), Some(42));
/// ```
#[derive(Debug)]
pub struct ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    inner: Arc<DashMap<K, V>>,
}

impl<K, V> Default for ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K, V> ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new empty `ConcurrentMap`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    /// Creates a new `ConcurrentMap` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the map
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(DashMap::with_capacity(capacity)),
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map already had this key present, the old value is returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// The old value if the key was present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// assert_eq!(map.insert("key".to_string(), 1), None);
    /// assert_eq!(map.insert("key".to_string(), 2), Some(1));
    /// ```
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Returns the number of elements in the map.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.insert("b".to_string(), 2);
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// assert!(map.is_empty());
    /// map.insert("key".to_string(), 1);
    /// assert!(!map.is_empty());
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
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("key".to_string(), 1);
    /// assert!(map.contains_key(&"key".to_string()));
    /// assert!(!map.contains_key(&"other".to_string()));
    /// ```
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Removes a key from the map, returning the value if present.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Returns
    ///
    /// The value if the key was present, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("key".to_string(), 42);
    /// assert_eq!(map.remove(&"key".to_string()), Some(42));
    /// assert_eq!(map.remove(&"key".to_string()), None);
    /// ```
    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key).map(|(_, v)| v)
    }

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.insert("b".to_string(), 2);
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Returns all keys in the map.
    ///
    /// Note: This creates a snapshot of the keys at the time of the call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.insert("b".to_string(), 2);
    /// let keys = map.keys();
    /// assert_eq!(keys.len(), 2);
    /// ```
    pub fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.inner.iter().map(|r| r.key().clone()).collect()
    }
}

impl<K, V> ConcurrentMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
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
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("key".to_string(), 42);
    /// assert_eq!(map.get(&"key".to_string()), Some(42));
    /// assert_eq!(map.get(&"other".to_string()), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|r| r.value().clone())
    }

    /// Gets a clone of the value or inserts a default value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up or insert
    /// * `default` - The default value to insert if key is not present
    ///
    /// # Returns
    ///
    /// A clone of the existing or newly inserted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// assert_eq!(map.get_or_insert("key".to_string(), 42), 42);
    /// assert_eq!(map.get_or_insert("key".to_string(), 100), 42);
    /// ```
    pub fn get_or_insert(&self, key: K, default: V) -> V {
        self.inner.entry(key).or_insert(default).value().clone()
    }

    /// Gets a clone of the value or inserts a value computed by a closure.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up or insert
    /// * `f` - A closure that computes the default value
    ///
    /// # Returns
    ///
    /// A clone of the existing or newly inserted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// let value = map.get_or_insert_with("key".to_string(), || 42);
    /// assert_eq!(value, 42);
    /// ```
    pub fn get_or_insert_with<F>(&self, key: K, f: F) -> V
    where
        F: FnOnce() -> V,
    {
        self.inner.entry(key).or_insert_with(f).value().clone()
    }

    /// Updates a value in place using a closure.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to update
    /// * `f` - A closure that takes the current value and returns the new value
    ///
    /// # Returns
    ///
    /// `true` if the key was found and updated, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("counter".to_string(), 0);
    /// map.update(&"counter".to_string(), |v| v + 1);
    /// assert_eq!(map.get(&"counter".to_string()), Some(1));
    /// ```
    pub fn update<F>(&self, key: &K, f: F) -> bool
    where
        F: FnOnce(V) -> V,
    {
        if let Some(mut entry) = self.inner.get_mut(key) {
            let new_value = f(entry.value().clone());
            *entry.value_mut() = new_value;
            true
        } else {
            false
        }
    }

    /// Returns all values in the map.
    ///
    /// Note: This creates a snapshot of the values at the time of the call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.insert("b".to_string(), 2);
    /// let values = map.values();
    /// assert_eq!(values.len(), 2);
    /// ```
    pub fn values(&self) -> Vec<V> {
        self.inner.iter().map(|r| r.value().clone()).collect()
    }

    /// Returns all key-value pairs in the map.
    ///
    /// Note: This creates a snapshot at the time of the call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// let entries = map.entries();
    /// assert_eq!(entries.len(), 1);
    /// ```
    pub fn entries(&self) -> Vec<(K, V)>
    where
        K: Clone,
    {
        self.inner
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that returns `true` for elements to keep
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentMap;
    ///
    /// let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
    /// map.insert("a".to_string(), 1);
    /// map.insert("b".to_string(), 2);
    /// map.insert("c".to_string(), 3);
    /// map.retain(|_, v| *v > 1);
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_and_default() {
        let map1: ConcurrentMap<String, i32> = ConcurrentMap::new();
        let map2: ConcurrentMap<String, i32> = ConcurrentMap::default();
        assert!(map1.is_empty());
        assert!(map2.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::with_capacity(100);
        assert!(map.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        assert_eq!(map.insert("key".to_string(), 42), None);
        assert_eq!(map.get(&"key".to_string()), Some(42));
        assert_eq!(map.insert("key".to_string(), 100), Some(42));
        assert_eq!(map.get(&"key".to_string()), Some(100));
    }

    #[test]
    fn test_remove() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("key".to_string(), 42);
        assert_eq!(map.remove(&"key".to_string()), Some(42));
        assert_eq!(map.remove(&"key".to_string()), None);
        assert!(map.is_empty());
    }

    #[test]
    fn test_contains_key() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("key".to_string(), 42);
        assert!(map.contains_key(&"key".to_string()));
        assert!(!map.contains_key(&"other".to_string()));
    }

    #[test]
    fn test_len_and_is_empty() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.insert("a".to_string(), 1);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);

        map.insert("b".to_string(), 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_clear() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_get_or_insert() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        assert_eq!(map.get_or_insert("key".to_string(), 42), 42);
        assert_eq!(map.get_or_insert("key".to_string(), 100), 42);
    }

    #[test]
    fn test_get_or_insert_with() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        let value = map.get_or_insert_with("key".to_string(), || 42);
        assert_eq!(value, 42);
        let value = map.get_or_insert_with("key".to_string(), || 100);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_update() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("counter".to_string(), 0);
        assert!(map.update(&"counter".to_string(), |v| v + 1));
        assert_eq!(map.get(&"counter".to_string()), Some(1));
        assert!(!map.update(&"nonexistent".to_string(), |v| v + 1));
    }

    #[test]
    fn test_keys_and_values() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        let keys = map.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));

        let values = map.values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&1));
        assert!(values.contains(&2));
    }

    #[test]
    fn test_entries() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("a".to_string(), 1);
        let entries = map.entries();
        assert_eq!(entries.len(), 1);
        assert!(entries.contains(&("a".to_string(), 1)));
    }

    #[test]
    fn test_retain() {
        let map: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);
        map.retain(|_, v| *v > 1);
        assert_eq!(map.len(), 2);
        assert!(!map.contains_key(&"a".to_string()));
    }

    #[test]
    fn test_clone() {
        let map1: ConcurrentMap<String, i32> = ConcurrentMap::new();
        map1.insert("key".to_string(), 42);
        let map2 = map1.clone();
        // Both maps share the same underlying data
        assert_eq!(map2.get(&"key".to_string()), Some(42));
        map2.insert("new".to_string(), 100);
        assert_eq!(map1.get(&"new".to_string()), Some(100));
    }

    #[test]
    fn test_concurrent_access() {
        let map: Arc<ConcurrentMap<i32, i32>> = Arc::new(ConcurrentMap::new());
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
    fn test_concurrent_update() {
        let map: Arc<ConcurrentMap<String, i32>> = Arc::new(ConcurrentMap::new());
        map.insert("counter".to_string(), 0);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let map = Arc::clone(&map);
                thread::spawn(move || {
                    for _ in 0..100 {
                        map.update(&"counter".to_string(), |v| v + 1);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.get(&"counter".to_string()), Some(1000));
    }
}
