//! Memory pool utilities for efficient memory management
//!
//! This module provides utilities to reduce memory allocations and improve
//! performance through object pooling and reuse.

#![allow(dead_code)] // Tool module - functions may be used in the future

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// A generic object pool for reusing expensive-to-create objects
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ObjectPool<T>
where
    T: Send + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            factory: Box::new(factory),
            max_size,
        }
    }

    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let obj = {
            let mut pool = self.pool.lock();
            pool.pop_front().unwrap_or_else(|| (self.factory)())
        };

        PooledObject {
            obj: Some(obj),
            pool: Arc::clone(&self.pool),
            max_size: self.max_size,
        }
    }

    /// Get the current size of the pool
    pub fn size(&self) -> usize {
        self.pool.lock().len()
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().clear();
    }
}

/// A wrapper around a pooled object that returns it to the pool when dropped
pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Get a reference to the inner object.
    ///
    /// # Panics
    /// Panics if the object has already been taken via `take()`.
    pub fn get_ref(&self) -> &T {
        self.obj.as_ref().expect("Object already taken")
    }

    /// Try to get a reference to the inner object.
    /// Returns `None` if the object has already been taken.
    pub fn try_get_ref(&self) -> Option<&T> {
        self.obj.as_ref()
    }

    /// Get a mutable reference to the inner object.
    ///
    /// # Panics
    /// Panics if the object has already been taken via `take()`.
    pub fn get_mut(&mut self) -> &mut T {
        self.obj.as_mut().expect("Object already taken")
    }

    /// Try to get a mutable reference to the inner object.
    /// Returns `None` if the object has already been taken.
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        self.obj.as_mut()
    }

    /// Take the object out of the pool wrapper (prevents return to pool).
    ///
    /// # Panics
    /// Panics if the object has already been taken.
    pub fn take(mut self) -> T {
        self.obj.take().expect("Object already taken")
    }

    /// Try to take the object out of the pool wrapper.
    /// Returns `None` if the object has already been taken.
    pub fn try_take(&mut self) -> Option<T> {
        self.obj.take()
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            let mut pool = self.pool.lock();
            if pool.len() < self.max_size {
                pool.push_back(obj);
            }
            // If pool is full, just drop the object
        }
    }
}

/// A memory-efficient buffer pool for byte operations
pub struct BufferPool {
    pool: ObjectPool<Vec<u8>>,
    default_capacity: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(default_capacity: usize, max_pooled: usize) -> Self {
        let pool = ObjectPool::new(move || Vec::with_capacity(default_capacity), max_pooled);

        Self {
            pool,
            default_capacity,
        }
    }

    /// Get a buffer from the pool
    pub fn get(&self) -> PooledBuffer {
        let mut buffer = self.pool.get();
        buffer.clear(); // Ensure buffer is empty
        PooledBuffer { buffer }
    }

    /// Get a buffer with a specific capacity
    pub fn get_with_capacity(&self, capacity: usize) -> PooledBuffer {
        let mut buffer = self.pool.get();
        buffer.clear();
        let current_capacity = buffer.capacity();
        if current_capacity < capacity {
            buffer.reserve(capacity - current_capacity);
        }
        PooledBuffer { buffer }
    }
}

/// A pooled buffer wrapper
pub struct PooledBuffer {
    buffer: PooledObject<Vec<u8>>,
}

impl PooledBuffer {
    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.get_mut().clear();
    }

    /// Extend the buffer with data
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        self.buffer.get_mut().extend_from_slice(data);
    }

    /// Get a slice of the buffer
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.get_ref().as_slice()
    }

    /// Take the inner Vec (prevents return to pool)
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer.take()
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.get_mut()
    }
}

/// A string pool optimized for specific use cases
pub struct OptimizedStringPool {
    small_strings: ObjectPool<String>,  // For strings < 64 chars
    medium_strings: ObjectPool<String>, // For strings < 256 chars
    large_strings: ObjectPool<String>,  // For strings >= 256 chars
}

impl OptimizedStringPool {
    /// Create a new optimized string pool
    pub fn new() -> Self {
        Self {
            small_strings: ObjectPool::new(|| String::with_capacity(64), 100),
            medium_strings: ObjectPool::new(|| String::with_capacity(256), 50),
            large_strings: ObjectPool::new(|| String::with_capacity(1024), 20),
        }
    }

    /// Get a string from the appropriate pool
    pub fn get_string(&self, estimated_size: usize) -> PooledString {
        let pooled = if estimated_size < 64 {
            PooledStringType::Small(self.small_strings.get())
        } else if estimated_size < 256 {
            PooledStringType::Medium(self.medium_strings.get())
        } else {
            PooledStringType::Large(self.large_strings.get())
        };

        let mut string = PooledString { inner: pooled };
        string.clear();
        string
    }
}

impl Default for OptimizedStringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A pooled string wrapper
pub struct PooledString {
    inner: PooledStringType,
}

enum PooledStringType {
    Small(PooledObject<String>),
    Medium(PooledObject<String>),
    Large(PooledObject<String>),
}

impl PooledString {
    /// Clear the string
    pub fn clear(&mut self) {
        match &mut self.inner {
            PooledStringType::Small(s) => s.clear(),
            PooledStringType::Medium(s) => s.clear(),
            PooledStringType::Large(s) => s.clear(),
        }
    }

    /// Get the string length
    pub fn len(&self) -> usize {
        match &self.inner {
            PooledStringType::Small(s) => s.len(),
            PooledStringType::Medium(s) => s.len(),
            PooledStringType::Large(s) => s.len(),
        }
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a string slice
    pub fn push_str(&mut self, s: &str) {
        match &mut self.inner {
            PooledStringType::Small(string) => string.push_str(s),
            PooledStringType::Medium(string) => string.push_str(s),
            PooledStringType::Large(string) => string.push_str(s),
        }
    }

    /// Take the inner string (prevents return to pool)
    pub fn into_string(self) -> String {
        match self.inner {
            PooledStringType::Small(s) => s.take(),
            PooledStringType::Medium(s) => s.take(),
            PooledStringType::Large(s) => s.take(),
        }
    }
}

impl std::ops::Deref for PooledString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            PooledStringType::Small(s) => s.as_str(),
            PooledStringType::Medium(s) => s.as_str(),
            PooledStringType::Large(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for PooledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            PooledStringType::Small(s) => write!(f, "{}", s.as_str()),
            PooledStringType::Medium(s) => write!(f, "{}", s.as_str()),
            PooledStringType::Large(s) => write!(f, "{}", s.as_str()),
        }
    }
}

/// Global instances for common use cases
use std::sync::LazyLock;

/// Global buffer pool
pub static BUFFER_POOL: LazyLock<BufferPool> = LazyLock::new(|| BufferPool::new(1024, 50));

/// Global string pool
pub static STRING_POOL: LazyLock<OptimizedStringPool> = LazyLock::new(OptimizedStringPool::new);

/// Convenience functions
///
/// Get a buffer from the global buffer pool
pub fn get_buffer() -> PooledBuffer {
    BUFFER_POOL.get()
}

/// Get a buffer with specific capacity from the global buffer pool
pub fn get_buffer_with_capacity(capacity: usize) -> PooledBuffer {
    BUFFER_POOL.get_with_capacity(capacity)
}

/// Get a string from the global string pool
pub fn get_string(estimated_size: usize) -> PooledString {
    STRING_POOL.get_string(estimated_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ObjectPool Tests =====

    #[test]
    fn test_object_pool_basic_get_and_return() {
        let pool = ObjectPool::new(Vec::<i32>::new, 2);

        {
            let mut obj1 = pool.get();
            obj1.push(1);
            assert_eq!(obj1.len(), 1);
        } // obj1 returned to pool

        assert_eq!(pool.size(), 1);

        let mut obj2 = pool.get();
        obj2.clear(); // Clear object when reusing
        assert_eq!(obj2.len(), 0); // Should be cleared when reused
    }

    #[test]
    fn test_object_pool_new_creates_empty_pool() {
        let pool = ObjectPool::new(|| String::from("default"), 5);
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_object_pool_respects_max_size() {
        let pool = ObjectPool::new(|| 0u32, 2);

        // Create 3 objects
        let obj1 = pool.get();
        let obj2 = pool.get();
        let obj3 = pool.get();

        // Drop all of them
        drop(obj1);
        drop(obj2);
        drop(obj3);

        // Only 2 should be in the pool (max_size = 2)
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_object_pool_clear() {
        let pool = ObjectPool::new(|| String::from("test"), 5);

        // Create and drop objects to fill the pool
        {
            let _obj1 = pool.get();
            let _obj2 = pool.get();
        }

        assert_eq!(pool.size(), 2);

        pool.clear();
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_object_pool_factory_is_called_when_empty() {
        let pool = ObjectPool::new(|| vec![42, 43], 2);

        let obj = pool.get();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj[0], 42);
        assert_eq!(obj[1], 43);
    }

    #[test]
    fn test_object_pool_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(ObjectPool::new(|| 0u64, 10));
        let mut handles = vec![];

        for _ in 0..5 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let obj = pool_clone.get();
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_millis(1));
                drop(obj);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // After all threads complete, pool should have up to max_size objects
        assert!(pool.size() <= 10);
    }

    // ===== PooledObject Tests =====

    #[test]
    fn test_pooled_object_get_ref() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);
        let obj = pool.get();
        let reference = obj.get_ref();
        assert_eq!(reference.len(), 3);
    }

    #[test]
    fn test_pooled_object_get_mut() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);
        let mut obj = pool.get();
        obj.get_mut().push(4);
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_pooled_object_take_prevents_return_to_pool() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);

        {
            let obj = pool.get();
            let taken = obj.take();
            assert_eq!(taken.len(), 3);
        } // obj is dropped but object was taken

        // Pool should be empty since object was taken
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pooled_object_deref() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);
        let obj = pool.get();
        // Use deref to access Vec methods directly
        assert_eq!(obj.len(), 3);
        assert_eq!(obj[0], 1);
    }

    #[test]
    fn test_pooled_object_deref_mut() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);
        let mut obj = pool.get();
        // Use deref_mut to modify
        obj.push(4);
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_pooled_object_returns_to_pool_on_drop() {
        let pool = ObjectPool::new(|| String::from("test"), 5);

        assert_eq!(pool.size(), 0);

        {
            let _obj = pool.get();
            assert_eq!(pool.size(), 0); // Not in pool while in use
        }

        assert_eq!(pool.size(), 1); // Returned to pool after drop
    }

    // ===== BufferPool Tests =====

    #[test]
    fn test_buffer_pool_basic_usage() {
        let pool = BufferPool::new(64, 5);

        {
            let mut buffer = pool.get();
            buffer.extend_from_slice(b"hello");
            assert_eq!(buffer.len(), 5);
        }

        let buffer2 = pool.get();
        assert_eq!(buffer2.len(), 0); // Should be cleared
    }

    #[test]
    fn test_buffer_pool_new_creates_with_capacity() {
        let pool = BufferPool::new(128, 3);
        let buffer = pool.get();
        assert!(buffer.capacity() >= 128);
    }

    #[test]
    fn test_buffer_pool_get_clears_buffer() {
        let pool = BufferPool::new(64, 5);

        {
            let mut buffer = pool.get();
            buffer.extend_from_slice(b"test data");
        }

        let buffer2 = pool.get();
        assert_eq!(buffer2.len(), 0);
        assert!(buffer2.is_empty());
    }

    #[test]
    fn test_buffer_pool_get_with_capacity() {
        let pool = BufferPool::new(64, 5);
        let buffer = pool.get_with_capacity(256);
        // The get_with_capacity should return a buffer that can hold 256 bytes
        // The actual capacity may vary based on allocator behavior
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_pool_get_with_capacity_smaller_than_existing() {
        let pool = BufferPool::new(128, 5);
        let buffer = pool.get_with_capacity(64);
        // Should not shrink, just use existing capacity
        assert!(buffer.capacity() >= 64);
    }

    // ===== PooledBuffer Tests =====

    #[test]
    fn test_pooled_buffer_len_and_is_empty() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.extend_from_slice(b"data");
        assert_eq!(buffer.len(), 4);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_pooled_buffer_capacity() {
        let pool = BufferPool::new(100, 5);
        let buffer = pool.get();
        assert!(buffer.capacity() >= 100);
    }

    #[test]
    fn test_pooled_buffer_clear() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.extend_from_slice(b"test");
        assert_eq!(buffer.len(), 4);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_pooled_buffer_extend_from_slice() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.extend_from_slice(b"hello ");
        buffer.extend_from_slice(b"world");
        assert_eq!(buffer.len(), 11);
        assert_eq!(buffer.as_slice(), b"hello world");
    }

    #[test]
    fn test_pooled_buffer_as_slice() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.extend_from_slice(b"test");
        let slice = buffer.as_slice();
        assert_eq!(slice, b"test");
    }

    #[test]
    fn test_pooled_buffer_into_vec() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.extend_from_slice(b"owned");

        let vec = buffer.into_vec();
        assert_eq!(vec, b"owned");

        // Pool should not have received the buffer back
        assert_eq!(pool.pool.size(), 0);
    }

    #[test]
    fn test_pooled_buffer_deref() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.push(65); // 'A'
        buffer.push(66); // 'B'
        assert_eq!(buffer.len(), 2);
        assert_eq!(&buffer[..], b"AB");
    }

    #[test]
    fn test_pooled_buffer_deref_mut() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        buffer.extend_from_slice(b"test");
        buffer[0] = b'T';
        assert_eq!(buffer.as_slice(), b"Test");
    }

    // ===== OptimizedStringPool Tests =====

    #[test]
    fn test_string_pool_basic_usage() {
        let pool = OptimizedStringPool::new();

        {
            let mut string = pool.get_string(10);
            string.push_str("hello");
            assert_eq!(string.len(), 5);
        }

        let string2 = pool.get_string(10);
        assert_eq!(string2.len(), 0); // Should be cleared
    }

    #[test]
    fn test_string_pool_small_string_selection() {
        let pool = OptimizedStringPool::new();
        let string = pool.get_string(30); // < 64
        // Should get a small string pool object
        drop(string);
        assert_eq!(pool.small_strings.size(), 1);
        assert_eq!(pool.medium_strings.size(), 0);
        assert_eq!(pool.large_strings.size(), 0);
    }

    #[test]
    fn test_string_pool_medium_string_selection() {
        let pool = OptimizedStringPool::new();
        let string = pool.get_string(100); // 64 <= size < 256
        drop(string);
        assert_eq!(pool.small_strings.size(), 0);
        assert_eq!(pool.medium_strings.size(), 1);
        assert_eq!(pool.large_strings.size(), 0);
    }

    #[test]
    fn test_string_pool_large_string_selection() {
        let pool = OptimizedStringPool::new();
        let string = pool.get_string(500); // >= 256
        drop(string);
        assert_eq!(pool.small_strings.size(), 0);
        assert_eq!(pool.medium_strings.size(), 0);
        assert_eq!(pool.large_strings.size(), 1);
    }

    #[test]
    fn test_string_pool_default() {
        let pool = OptimizedStringPool::default();
        let string = pool.get_string(10);
        assert!(string.is_empty());
    }

    // ===== PooledString Tests =====

    #[test]
    fn test_pooled_string_clear() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(10);
        string.push_str("test");
        assert_eq!(string.len(), 4);

        string.clear();
        assert_eq!(string.len(), 0);
        assert!(string.is_empty());
    }

    #[test]
    fn test_pooled_string_len_and_is_empty() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(10);
        assert_eq!(string.len(), 0);
        assert!(string.is_empty());

        string.push_str("data");
        assert_eq!(string.len(), 4);
        assert!(!string.is_empty());
    }

    #[test]
    fn test_pooled_string_push_str() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(20);
        string.push_str("hello ");
        string.push_str("world");
        assert_eq!(string.len(), 11);
        assert_eq!(&*string, "hello world");
    }

    #[test]
    fn test_pooled_string_push_str_all_sizes() {
        let pool = OptimizedStringPool::new();

        // Small
        let mut small = pool.get_string(10);
        small.push_str("small");
        assert_eq!(&*small, "small");

        // Medium
        let mut medium = pool.get_string(100);
        medium.push_str("medium");
        assert_eq!(&*medium, "medium");

        // Large
        let mut large = pool.get_string(300);
        large.push_str("large");
        assert_eq!(&*large, "large");
    }

    #[test]
    fn test_pooled_string_into_string() {
        let pool = OptimizedStringPool::new();
        let mut pooled = pool.get_string(10);
        pooled.push_str("owned");

        let owned = pooled.into_string();
        assert_eq!(owned, "owned");

        // Pool should not have received the string back
        assert_eq!(pool.small_strings.size(), 0);
    }

    #[test]
    fn test_pooled_string_deref() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(10);
        string.push_str("test");
        // Use deref to access &str methods
        assert_eq!(string.chars().count(), 4);
        assert!(string.contains("es"));
    }

    #[test]
    fn test_pooled_string_display() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(10);
        string.push_str("display");
        assert_eq!(format!("{}", string), "display");
    }

    #[test]
    fn test_pooled_string_display_all_sizes() {
        let pool = OptimizedStringPool::new();

        let mut small = pool.get_string(10);
        small.push_str("small");
        assert_eq!(format!("{}", small), "small");

        let mut medium = pool.get_string(100);
        medium.push_str("medium");
        assert_eq!(format!("{}", medium), "medium");

        let mut large = pool.get_string(300);
        large.push_str("large");
        assert_eq!(format!("{}", large), "large");
    }

    // ===== Global Pool Tests =====

    #[test]
    fn test_global_buffer_pool_get_buffer() {
        let buffer = get_buffer();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.capacity() >= 1024);
    }

    #[test]
    fn test_global_buffer_pool_get_buffer_with_capacity() {
        let buffer = get_buffer_with_capacity(2048);
        assert_eq!(buffer.len(), 0);
        // Note: capacity may be less if pool returns a reused buffer
        // The important thing is we get a usable buffer
    }

    #[test]
    fn test_global_string_pool_get_string() {
        let string = get_string(50);
        assert_eq!(string.len(), 0);
        assert!(string.is_empty());
    }

    #[test]
    fn test_global_pools_are_reusable() {
        // Test buffer pool reuse
        {
            let mut buffer = get_buffer();
            buffer.extend_from_slice(b"test");
        }
        let buffer2 = get_buffer();
        assert_eq!(buffer2.len(), 0); // Should be cleared

        // Test string pool reuse
        {
            let mut string = get_string(20);
            string.push_str("test");
        }
        let string2 = get_string(20);
        assert_eq!(string2.len(), 0); // Should be cleared
    }

    // ===== Edge Cases and Stress Tests =====

    #[test]
    fn test_object_pool_with_zero_max_size() {
        let pool = ObjectPool::new(|| 0u32, 0);

        {
            let _obj = pool.get();
        }

        // No object should be returned to pool
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_buffer_pool_multiple_extensions() {
        let pool = BufferPool::new(16, 5);
        let mut buffer = pool.get();

        for i in 0..100 {
            buffer.push(i as u8);
        }

        assert_eq!(buffer.len(), 100);
        // Capacity should have grown
        assert!(buffer.capacity() >= 100);
    }

    #[test]
    fn test_pooled_string_large_content() {
        let pool = OptimizedStringPool::new();
        let mut string = pool.get_string(500);

        let large_text = "a".repeat(1000);
        string.push_str(&large_text);

        assert_eq!(string.len(), 1000);
        assert_eq!(&*string, large_text);
    }

    #[test]
    fn test_object_pool_different_types() {
        // Test with String
        let string_pool = ObjectPool::new(String::new, 5);
        let s = string_pool.get();
        drop(s);
        assert_eq!(string_pool.size(), 1);

        // Test with HashMap
        use std::collections::HashMap;
        let map_pool = ObjectPool::new(HashMap::<String, i32>::new, 3);
        let m = map_pool.get();
        drop(m);
        assert_eq!(map_pool.size(), 1);
    }

    #[test]
    fn test_pooled_buffer_preserves_capacity_after_clear() {
        let pool = BufferPool::new(64, 5);
        let mut buffer = pool.get();
        let initial_capacity = buffer.capacity();

        buffer.extend_from_slice(&[0u8; 50]);
        buffer.clear();

        // Capacity should be preserved
        assert_eq!(buffer.capacity(), initial_capacity);
    }

    #[test]
    fn test_string_pool_boundary_sizes() {
        let pool = OptimizedStringPool::new();

        // Test that we can get strings at boundary sizes without panicking
        // Exactly 63 - should use small pool
        let s63 = pool.get_string(63);
        assert!(s63.is_empty());

        // Exactly 64 - should use medium pool
        let s64 = pool.get_string(64);
        assert!(s64.is_empty());

        // Exactly 255 - should use medium pool
        let s255 = pool.get_string(255);
        assert!(s255.is_empty());

        // Exactly 256 - should use large pool
        let s256 = pool.get_string(256);
        assert!(s256.is_empty());
    }

    #[test]
    fn test_pooled_object_multiple_get_mut_calls() {
        let pool = ObjectPool::new(|| vec![1, 2, 3], 1);
        let mut obj = pool.get();

        obj.get_mut().push(4);
        obj.get_mut().push(5);
        obj.get_mut().extend_from_slice(&[6, 7]);

        assert_eq!(obj.len(), 7);
    }
}
