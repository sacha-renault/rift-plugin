//! A [`Vec`] wrapper with cheap equality checks for vizia's reactive data system.
//!
//! Tracks identity and a mutation counter instead of deep-comparing elements.
//! Mutable borrows always bump the counter, even if nothing actually changes,
//! this can cause redundant re-renders but never incorrect behavior.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use vizia::prelude::Data;

static NEXT_VEC_ID: AtomicU64 = AtomicU64::new(0);

/// A [`Vec`] wrapper that supports cheap equality comparison via identity and
/// mutation tracking, designed for use as reactive model data in vizia.
#[derive(Clone)]
pub struct ComparativeVec<T: 'static> {
    inner: Vec<T>,
    change_count: u64,
    id: u64,
}

impl<T> From<Vec<T>> for ComparativeVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from_vec(value)
    }
}

impl<T> ComparativeVec<T> {
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        let id = NEXT_VEC_ID.fetch_add(1, Ordering::Relaxed);

        Self {
            inner: vec,
            change_count: 0,
            id,
        }
    }
}

impl<T: 'static> Deref for ComparativeVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: 'static> DerefMut for ComparativeVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.change_count = self.change_count.wrapping_add(1);
        &mut self.inner
    }
}

impl<T> Data for ComparativeVec<T>
where
    T: 'static + Clone,
{
    fn same(&self, other: &Self) -> bool {
        self.id == other.id && self.change_count == other.change_count
    }
}
