use std::ops::{Deref, DerefMut};

/// A `Vec`-like container with a fixed capacity that never reallocates.
///
/// Unlike `Vec`, pushing beyond the allocated capacity panics instead of
/// triggering a reallocation. Useful when allocation must be explicit and
/// bounded — e.g. real-time, arena-style, or performance-sensitive code.
///
/// Capacity is set once at construction and preserved through cloning.
#[derive(Debug)]
pub struct BoundedVec<T> {
    inner: Vec<T>,
}

impl<T> BoundedVec<T> {
    /// Returns `true` if there is room for `new_items_size` more elements.
    fn has_enough_capacity(&self, new_items_size: usize) -> bool {
        self.len() + new_items_size <= self.capacity()
    }

    /// Panics if there is not enough capacity for `new_items_size` more elements.
    fn assert_capacity(&self, new_items_size: usize) {
        if !self.has_enough_capacity(new_items_size) {
            panic!("No reallocation on OnceVec")
        }
    }
}

impl<T> BoundedVec<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_full(&self) -> bool {
        self.capacity() == self.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.has_enough_capacity(1) {
            self.inner.push(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    #[inline(always)]
    pub fn push(&mut self, value: T) {
        if self.try_push(value).is_err() {
            panic!("No reallocation on OnceVec")
        }
    }

    pub fn resize_with<F>(&mut self, size: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.clear();
        self.assert_capacity(size);
        self.inner.resize_with(size, f);
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.assert_capacity(1);
        self.inner.insert(index, element);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.inner.swap(a, b);
    }
}

impl<T> BoundedVec<T>
where
    T: Clone,
{
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.assert_capacity(slice.len());
        self.inner.extend_from_slice(slice);
    }

    pub fn resize(&mut self, size: usize, value: T) {
        self.clear();
        self.assert_capacity(size);
        self.inner.resize(size, value);
    }
}

impl<T> Deref for BoundedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for BoundedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Clone for BoundedVec<T>
where
    T: Clone,
{
    /// Cloning once Vec allocate same capacity for the clone
    /// as capacity is as important as the data OnceVec contain
    fn clone(&self) -> Self {
        let mut clone = Vec::with_capacity(self.inner.capacity());
        clone.extend_from_slice(&self.inner);
        Self { inner: clone }
    }
}
