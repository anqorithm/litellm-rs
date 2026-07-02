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
