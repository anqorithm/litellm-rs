//! Concurrent-safe Vec implementation
//!
//! Provides a thread-safe vector with fine-grained locking using parking_lot.
//! Supports common vector operations with concurrent access safety.

use parking_lot::RwLock;
use std::sync::Arc;

/// A concurrent-safe Vec using RwLock for synchronization.
///
/// This container provides thread-safe access to a vector with read-write
/// locking. Multiple readers can access the vector simultaneously, but
/// writers have exclusive access.
///
/// # Type Parameters
///
/// * `T` - The element type, must implement `Clone`
///
/// # Example
///
/// ```rust
/// use litellm_rs::utils::sync::ConcurrentVec;
///
/// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
/// vec.push(1);
/// vec.push(2);
/// assert_eq!(vec.len(), 2);
/// assert_eq!(vec.get(0), Some(1));
/// ```
#[derive(Debug)]
pub struct ConcurrentVec<T> {
    inner: Arc<RwLock<Vec<T>>>,
}

impl<T> Default for ConcurrentVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for ConcurrentVec<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> ConcurrentVec<T> {
    /// Creates a new empty `ConcurrentVec`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// assert!(vec.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new `ConcurrentVec` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the vector
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::with_capacity(100);
    /// assert!(vec.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
        }
    }

    /// Appends an element to the back of the vector.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn push(&self, value: T) {
        self.inner.write().push(value);
    }

    /// Removes the last element from the vector and returns it.
    ///
    /// # Returns
    ///
    /// The last element if the vector is not empty, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.pop(), Some(2));
    /// assert_eq!(vec.pop(), Some(1));
    /// assert_eq!(vec.pop(), None);
    /// ```
    pub fn pop(&self) -> Option<T> {
        self.inner.write().pop()
    }

    /// Returns the number of elements in the vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// assert_eq!(vec.len(), 0);
    /// vec.push(1);
    /// assert_eq!(vec.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// assert!(vec.is_empty());
    /// vec.push(1);
    /// assert!(!vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Clears the vector, removing all elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.clear();
    /// assert!(vec.is_empty());
    /// ```
    pub fn clear(&self) {
        self.inner.write().clear();
    }

    /// Returns the capacity of the vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::with_capacity(10);
    /// assert!(vec.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.inner.read().capacity()
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional elements to reserve space for
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.reserve(100);
    /// assert!(vec.capacity() >= 100);
    /// ```
    pub fn reserve(&self, additional: usize) {
        self.inner.write().reserve(additional);
    }

    /// Shrinks the capacity of the vector as much as possible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::with_capacity(100);
    /// vec.push(1);
    /// vec.shrink_to_fit();
    /// ```
    pub fn shrink_to_fit(&self) {
        self.inner.write().shrink_to_fit();
    }

    /// Truncates the vector to the specified length.
    ///
    /// # Arguments
    ///
    /// * `len` - The new length of the vector
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.truncate(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn truncate(&self, len: usize) {
        self.inner.write().truncate(len);
    }
}

impl<T: Clone> ConcurrentVec<T> {
    /// Gets a clone of the element at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the element to get
    ///
    /// # Returns
    ///
    /// A clone of the element if the index is valid, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(42);
    /// assert_eq!(vec.get(0), Some(42));
    /// assert_eq!(vec.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<T> {
        self.inner.read().get(index).cloned()
    }

    /// Sets the element at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the element to set
    /// * `value` - The new value
    ///
    /// # Returns
    ///
    /// `true` if the index was valid and the element was set, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// assert!(vec.set(0, 42));
    /// assert_eq!(vec.get(0), Some(42));
    /// assert!(!vec.set(10, 100));
    /// ```
    pub fn set(&self, index: usize, value: T) -> bool {
        let mut guard = self.inner.write();
        if index < guard.len() {
            guard[index] = value;
            true
        } else {
            false
        }
    }

    /// Gets a clone of the first element.
    ///
    /// # Returns
    ///
    /// A clone of the first element if the vector is not empty, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// assert_eq!(vec.first(), None);
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.first(), Some(1));
    /// ```
    pub fn first(&self) -> Option<T> {
        self.inner.read().first().cloned()
    }

    /// Gets a clone of the last element.
    ///
    /// # Returns
    ///
    /// A clone of the last element if the vector is not empty, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// assert_eq!(vec.last(), None);
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.last(), Some(2));
    /// ```
    pub fn last(&self) -> Option<T> {
        self.inner.read().last().cloned()
    }

    /// Returns a snapshot of all elements as a Vec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let snapshot = vec.to_vec();
    /// assert_eq!(snapshot, vec![1, 2]);
    /// ```
    pub fn to_vec(&self) -> Vec<T> {
        self.inner.read().clone()
    }

    /// Extends the vector with elements from an iterator.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator yielding elements to append
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.extend(vec![1, 2, 3]);
    /// assert_eq!(vec.len(), 3);
    /// ```
    pub fn extend<I: IntoIterator<Item = T>>(&self, iter: I) {
        self.inner.write().extend(iter);
    }

    /// Inserts an element at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index at which to insert
    /// * `value` - The value to insert
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(3);
    /// vec.insert(1, 2);
    /// assert_eq!(vec.to_vec(), vec![1, 2, 3]);
    /// ```
    pub fn insert(&self, index: usize, value: T) {
        self.inner.write().insert(index, value);
    }

    /// Removes and returns the element at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the element to remove
    ///
    /// # Returns
    ///
    /// The removed element if the index is valid, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// assert_eq!(vec.remove(1), Some(2));
    /// assert_eq!(vec.to_vec(), vec![1, 3]);
    /// ```
    pub fn remove(&self, index: usize) -> Option<T> {
        let mut guard = self.inner.write();
        if index < guard.len() {
            Some(guard.remove(index))
        } else {
            None
        }
    }

    /// Removes and returns the element at the specified index by swapping with the last element.
    ///
    /// This is O(1) but does not preserve ordering.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the element to remove
    ///
    /// # Returns
    ///
    /// The removed element if the index is valid, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// assert_eq!(vec.swap_remove(0), Some(1));
    /// // Order is not preserved: [3, 2]
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn swap_remove(&self, index: usize) -> Option<T> {
        let mut guard = self.inner.write();
        if index < guard.len() {
            Some(guard.swap_remove(index))
        } else {
            None
        }
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
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.extend(vec![1, 2, 3, 4, 5]);
    /// vec.retain(|x| *x % 2 == 0);
    /// assert_eq!(vec.to_vec(), vec![2, 4]);
    /// ```
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.write().retain(f);
    }

    /// Applies a function to each element in place.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that transforms each element
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.extend(vec![1, 2, 3]);
    /// vec.for_each_mut(|x| *x *= 2);
    /// assert_eq!(vec.to_vec(), vec![2, 4, 6]);
    /// ```
    pub fn for_each_mut<F>(&self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let mut guard = self.inner.write();
        for item in guard.iter_mut() {
            f(item);
        }
    }

    /// Returns `true` if the vector contains the specified element.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to search for
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert!(vec.contains(&1));
    /// assert!(!vec.contains(&3));
    /// ```
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.read().contains(value)
    }

    /// Returns the index of the first element equal to the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to search for
    ///
    /// # Returns
    ///
    /// The index of the first matching element, or `None` if not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use litellm_rs::utils::sync::ConcurrentVec;
    ///
    /// let vec: ConcurrentVec<i32> = ConcurrentVec::new();
    /// vec.extend(vec![1, 2, 3, 2]);
    /// assert_eq!(vec.position(&2), Some(1));
    /// assert_eq!(vec.position(&5), None);
    /// ```
    pub fn position(&self, value: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.inner.read().iter().position(|x| x == value)
    }
}

impl<T: Clone> From<Vec<T>> for ConcurrentVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(vec)),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
