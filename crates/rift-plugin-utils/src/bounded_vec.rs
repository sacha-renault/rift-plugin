use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A `Vec`-like container with a fixed capacity that never reallocates.
///
/// The capacity is encoded in the type as the const parameter `N`, so it is
/// always defined by the plugin source, never by the saved state. Serializing a
/// `BoundedVec` therefore only stores its elements; deserializing rebuilds it
/// with the capacity `N` taken from the target type.
///
/// Unlike `Vec`, pushing beyond the capacity panics instead of triggering a
/// reallocation. Useful when allocation must be explicit and bounded - e.g.
/// real-time, arena-style, or performance-sensitive code.
#[derive(Debug)]
pub struct BoundedVec<T, const N: usize> {
    inner: Vec<T>,
}

impl<T: Serialize, const N: usize> Serialize for BoundedVec<T, N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Only the elements are saved: the capacity lives in the type, not in the state.
        self.inner.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for BoundedVec<T, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = Vec::<T>::deserialize(deserializer)?;
        if data.len() > N {
            return Err(serde::de::Error::custom(format!(
                "BoundedVec data length ({}) exceeds its capacity ({N})",
                data.len()
            )));
        }

        let mut inner = Vec::with_capacity(N);
        inner.extend(data);
        Ok(Self { inner })
    }
}

impl<T, const N: usize> BoundedVec<T, N> {
    /// Returns `true` if there is room for `new_items_size` more elements.
    fn has_enough_capacity(&self, new_items_size: usize) -> bool {
        self.len() + new_items_size <= self.capacity()
    }

    /// Panics if there is not enough capacity for `new_items_size` more elements.
    fn assert_capacity(&self, new_items_size: usize) {
        if !self.has_enough_capacity(new_items_size) {
            panic!("No reallocation on BoundedVec")
        }
    }
}

impl<T, const N: usize> BoundedVec<T, N> {
    /// Creates an empty `BoundedVec`. The capacity is `N`, taken from the type;
    /// no further allocation will ever occur.
    pub fn new() -> Self {
        Self {
            inner: Vec::with_capacity(N),
        }
    }

    /// Returns the fixed capacity `N`.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Returns the number of unused slots before the capacity is reached.
    pub fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Returns the number of elements currently stored.
    /// [`Vec::len`]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the vector contains no elements.
    /// [`Vec::is_empty`]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if `len == capacity`. no more elements can be pushed.
    pub fn is_full(&self) -> bool {
        self.capacity() == self.len()
    }

    /// Removes all elements without affecting capacity.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Pushes `value`, returning `Err(value)` if the vec is full.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.has_enough_capacity(1) {
            self.inner.push(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Pushes `value`. Panics if the vec is full.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        if self.try_push(value).is_err() {
            panic!("No reallocation on BoundedVec")
        }
    }

    /// Clears the vec, then fills it with `size` elements produced by `f`.
    /// Panics if `size` exceeds capacity.
    pub fn resize_with<F>(&mut self, size: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.clear();
        self.assert_capacity(size);
        self.inner.resize_with(size, f);
    }

    /// Inserts `element` at `index`, shifting elements to the right.
    /// Panics if the vec is full. [`Vec::insert`]
    pub fn insert(&mut self, index: usize, element: T) {
        self.assert_capacity(1);
        self.inner.insert(index, element);
    }

    /// Removes and returns the last element, or `None` if empty.
    /// [`Vec::pop`]
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Removes and returns the element at `index`.
    /// [`Vec::remove`]
    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }

    /// Swaps elements at indices `a` and `b`.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.inner.swap(a, b);
    }

    /// Retain (or not) element in inner
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<T, const N: usize> Default for BoundedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> BoundedVec<T, N>
where
    T: Clone,
{
    /// Appends all elements from `slice`. Panics if not enough remaining capacity.
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.assert_capacity(slice.len());
        self.inner.extend_from_slice(slice);
    }

    /// Clears the vec, then resizes it to `size` elements cloned from `value`.
    /// Panics if `size` exceeds capacity.
    pub fn resize(&mut self, size: usize, value: T) {
        self.clear();
        self.assert_capacity(size);
        self.inner.resize(size, value);
    }
}

impl<T, const N: usize> Deref for BoundedVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, const N: usize> DerefMut for BoundedVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T, const N: usize> Clone for BoundedVec<T, N>
where
    T: Clone,
{
    /// Clones the data; the capacity `N` is part of the type and therefore
    /// preserved across clones by construction.
    fn clone(&self) -> Self {
        let mut clone = Vec::with_capacity(N);
        clone.extend_from_slice(&self.inner);
        Self { inner: clone }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_correct_capacity() {
        let v: BoundedVec<i32, 10> = BoundedVec::new();
        assert_eq!(v.capacity(), 10);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn new_zero_capacity() {
        let v: BoundedVec<i32, 0> = BoundedVec::new();
        assert_eq!(v.capacity(), 0);
        assert!(v.is_full());
    }

    #[test]
    fn push_within_capacity() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert!(v.is_full());
    }

    #[test]
    #[should_panic]
    fn push_beyond_capacity_panics() {
        let mut v: BoundedVec<i32, 1> = BoundedVec::new();
        v.push(1);
        v.push(2); // panics
    }

    #[test]
    fn try_push_returns_ok_when_space() {
        let mut v: BoundedVec<i32, 1> = BoundedVec::new();
        assert!(v.try_push(42).is_ok());
    }

    #[test]
    fn try_push_returns_err_when_full() {
        let mut v: BoundedVec<i32, 1> = BoundedVec::new();
        v.push(1);
        assert_eq!(v.try_push(2), Err(2));
    }

    #[test]
    fn remaining_capacity() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        assert_eq!(v.remaining_capacity(), 5);
        v.push(1);
        assert_eq!(v.remaining_capacity(), 4);
    }

    #[test]
    fn is_full() {
        let mut v: BoundedVec<i32, 2> = BoundedVec::new();
        assert!(!v.is_full());
        v.push(1);
        assert!(!v.is_full());
        v.push(2);
        assert!(v.is_full());
    }

    #[test]
    fn clear_resets_len_not_capacity() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        v.push(1);
        v.push(2);
        v.clear();
        assert_eq!(v.len(), 0);
        assert_eq!(v.capacity(), 5); // capacity preserved
    }

    #[test]
    fn pop_returns_last_element() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn remove_correct_element() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        assert_eq!(v.remove(1), 20);
        assert_eq!(&*v, &[10, 30]);
    }

    #[test]
    fn insert_within_capacity() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v.push(3);
        v.insert(1, 2);
        assert_eq!(&*v, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn insert_beyond_capacity_panics() {
        let mut v: BoundedVec<i32, 2> = BoundedVec::new();
        v.push(1);
        v.push(2);
        v.insert(0, 0); // panics
    }

    #[test]
    fn swap_elements() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.swap(0, 2);
        assert_eq!(&*v, &[3, 2, 1]);
    }

    #[test]
    fn deref_to_slice() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn deref_mut_allows_mutation() {
        let mut v: BoundedVec<i32, 3> = BoundedVec::new();
        v.push(1);
        v[0] = 99;
        assert_eq!(v[0], 99);
    }

    #[test]
    fn extend_from_slice_within_capacity() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        v.extend_from_slice(&[1, 2, 3]);
        assert_eq!(&*v, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn extend_from_slice_beyond_capacity_panics() {
        let mut v: BoundedVec<i32, 2> = BoundedVec::new();
        v.extend_from_slice(&[1, 2, 3]); // panics
    }

    #[test]
    fn resize_fills_and_clears() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        v.push(99);
        v.resize(3, 0);
        assert_eq!(&*v, &[0, 0, 0]);
    }

    #[test]
    fn resize_with_fills_and_clears() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        v.push(99);
        let mut n = 0;
        v.resize_with(3, || {
            n += 1;
            n
        });
        assert_eq!(&*v, &[1, 2, 3]);
    }

    #[test]
    fn clone_preserves_data_and_capacity() {
        let mut v: BoundedVec<i32, 5> = BoundedVec::new();
        v.push(1);
        v.push(2);
        let c = v.clone();
        assert_eq!(&*c, &[1, 2]);
        assert_eq!(c.capacity(), 5); // capacity preserved by the type
    }

    #[test]
    fn serde_does_not_store_capacity() {
        let mut v: BoundedVec<i32, 10> = BoundedVec::new();
        v.push(1);
        v.push(2);
        v.push(3);

        let json = serde_json::to_string(&v).unwrap();
        // Capacity must come from the type, not the saved state.
        assert!(!json.contains("capacity"));

        let deserialized: BoundedVec<i32, 10> = serde_json::from_str(&json).unwrap();
        assert_eq!(&*deserialized, &[1, 2, 3]);
        assert_eq!(deserialized.capacity(), 10);
    }

    #[test]
    fn serde_rejects_data_exceeding_capacity() {
        // Serialize with a larger capacity, then load into a smaller one.
        let mut v: BoundedVec<i32, 10> = BoundedVec::new();
        v.extend_from_slice(&[1, 2, 3]);
        let json = serde_json::to_string(&v).unwrap();

        let result: Result<BoundedVec<i32, 2>, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}
