use std::sync::atomic::{AtomicU64, Ordering};

/// A small helper to atomically load and store an `f64` value.
pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    /// Creates a new atomic `f64`.
    #[inline]
    pub fn new(value: f64) -> Self {
        Self(AtomicU64::new(f64_to_u64_bytes(value)))
    }

    /// Stores the given `value` using the given `order`ing.
    #[inline]
    pub fn store(&self, value: f64, order: Ordering) {
        self.0.store(f64_to_u64_bytes(value), order)
    }

    /// Loads the contained `value` using the given `order`ing.
    #[inline]
    pub fn load(&self, order: Ordering) -> f64 {
        f64_from_u64_bytes(self.0.load(order))
    }
}

/// Packs a `f64` into the bytes of an `u64`.
///
/// The resulting value is meaningless and should not be used directly,
/// except for unpacking with [`f64_from_u64_bytes`].
///
/// This is an internal helper used by [`Atomicf64`].
#[inline]
fn f64_to_u64_bytes(value: f64) -> u64 {
    u64::from_ne_bytes(value.to_ne_bytes())
}

/// The counterpart to [`f64_to_u64_bytes`].
#[inline]
fn f64_from_u64_bytes(bytes: u64) -> f64 {
    f64::from_ne_bytes(bytes.to_ne_bytes())
}
