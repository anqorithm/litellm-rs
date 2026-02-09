use super::*;
use std::thread;

#[test]
fn test_new_and_default() {
    let vec1: ConcurrentVec<i32> = ConcurrentVec::new();
    let vec2: ConcurrentVec<i32> = ConcurrentVec::default();
    assert!(vec1.is_empty());
    assert!(vec2.is_empty());
}

#[test]
fn test_with_capacity() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::with_capacity(100);
    assert!(vec.is_empty());
    assert!(vec.capacity() >= 100);
}

#[test]
fn test_push_and_pop() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
}

#[test]
fn test_get_and_set() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.push(1);
    vec.push(2);
    assert_eq!(vec.get(0), Some(1));
    assert_eq!(vec.get(1), Some(2));
    assert_eq!(vec.get(2), None);

    assert!(vec.set(0, 10));
    assert_eq!(vec.get(0), Some(10));
    assert!(!vec.set(10, 100));
}

#[test]
fn test_first_and_last() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    assert_eq!(vec.first(), None);
    assert_eq!(vec.last(), None);

    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.first(), Some(1));
    assert_eq!(vec.last(), Some(3));
}

#[test]
fn test_len_and_is_empty() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);

    vec.push(1);
    assert!(!vec.is_empty());
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_clear() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.push(1);
    vec.push(2);
    vec.clear();
    assert!(vec.is_empty());
}

#[test]
fn test_to_vec() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.to_vec(), vec![1, 2, 3]);
}

#[test]
fn test_extend() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3]);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.to_vec(), vec![1, 2, 3]);
}

#[test]
fn test_insert_and_remove() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.push(1);
    vec.push(3);
    vec.insert(1, 2);
    assert_eq!(vec.to_vec(), vec![1, 2, 3]);

    assert_eq!(vec.remove(1), Some(2));
    assert_eq!(vec.to_vec(), vec![1, 3]);
    assert_eq!(vec.remove(10), None);
}

#[test]
fn test_swap_remove() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3]);
    assert_eq!(vec.swap_remove(0), Some(1));
    assert_eq!(vec.len(), 2);
    // Order changed: [3, 2]
}

#[test]
fn test_retain() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3, 4, 5]);
    vec.retain(|x| *x % 2 == 0);
    assert_eq!(vec.to_vec(), vec![2, 4]);
}

#[test]
fn test_for_each_mut() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3]);
    vec.for_each_mut(|x| *x *= 2);
    assert_eq!(vec.to_vec(), vec![2, 4, 6]);
}

#[test]
fn test_contains_and_position() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3, 2]);
    assert!(vec.contains(&2));
    assert!(!vec.contains(&5));
    assert_eq!(vec.position(&2), Some(1));
    assert_eq!(vec.position(&5), None);
}

#[test]
fn test_truncate() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.extend(vec![1, 2, 3, 4, 5]);
    vec.truncate(3);
    assert_eq!(vec.to_vec(), vec![1, 2, 3]);
}

#[test]
fn test_reserve_and_shrink() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    vec.reserve(100);
    assert!(vec.capacity() >= 100);
    vec.push(1);
    vec.shrink_to_fit();
}

#[test]
fn test_from_vec() {
    let vec: ConcurrentVec<i32> = ConcurrentVec::from(vec![1, 2, 3]);
    assert_eq!(vec.to_vec(), vec![1, 2, 3]);
}

#[test]
fn test_clone() {
    let vec1: ConcurrentVec<i32> = ConcurrentVec::new();
    vec1.push(1);
    let vec2 = vec1.clone();
    // Both vecs share the same underlying data
    assert_eq!(vec2.get(0), Some(1));
    vec2.push(2);
    assert_eq!(vec1.len(), 2);
}

#[test]
fn test_concurrent_push() {
    let vec: Arc<ConcurrentVec<i32>> = Arc::new(ConcurrentVec::new());
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let vec = Arc::clone(&vec);
            thread::spawn(move || {
                for j in 0..100 {
                    vec.push(i * 100 + j);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(vec.len(), 1000);
}

#[test]
fn test_concurrent_read_write() {
    let vec: Arc<ConcurrentVec<i32>> = Arc::new(ConcurrentVec::new());
    vec.extend(0..100);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let vec = Arc::clone(&vec);
            thread::spawn(move || {
                for _ in 0..100 {
                    // Mix of reads and writes
                    let _ = vec.get(i % 100);
                    vec.push(i as i32);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert!(vec.len() >= 100);
}
