use std::sync::atomic::{AtomicU32, Ordering};

/// A small helper to atomically load and store an `f32` value.
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    /// Creates a new atomic `f32`.
    #[inline]
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(f32_to_u32_bytes(value)))
    }

    /// Stores the given `value` using the given `order`ing.
    #[inline]
    pub fn store(&self, value: f32, order: Ordering) {
        self.0.store(f32_to_u32_bytes(value), order)
    }

    /// Loads the contained `value` using the given `order`ing.
    #[inline]
    pub fn load(&self, order: Ordering) -> f32 {
        f32_from_u32_bytes(self.0.load(order))
    }
}

/// Packs a `f32` into the bytes of an `u32`.
///
/// The resulting value is meaningless and should not be used directly,
/// except for unpacking with [`f32_from_u32_bytes`].
///
/// This is an internal helper used by [`Atomicf32`].
#[inline]
fn f32_to_u32_bytes(value: f32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

/// The counterpart to [`f32_to_u32_bytes`].
#[inline]
fn f32_from_u32_bytes(bytes: u32) -> f32 {
    f32::from_ne_bytes(bytes.to_ne_bytes())
}
