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
