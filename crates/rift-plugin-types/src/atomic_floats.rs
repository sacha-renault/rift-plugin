use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// A small helper to atomically load and store an `f32` value.
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    /// Creates a new atomic `f32`.
    #[inline]
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    /// Stores the given `value` using the given `order`ing.
    #[inline]
    pub fn store(&self, value: f32, order: Ordering) {
        self.0.store(value.to_bits(), order)
    }

    /// Loads the contained `value` using the given `order`ing.
    #[inline]
    pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.0.load(order))
    }
}

#[cfg(test)]
mod tests_f32 {
    use super::*;

    #[test]
    fn test_atomic_f32() {
        let value = AtomicF32::new(10.);
        assert_eq!(value.load(Ordering::Relaxed), 10.);

        value.store(15., Ordering::Relaxed);
        assert_eq!(value.load(Ordering::Relaxed), 15.);
    }
}

/// A small helper to atomically load and store an `f64` value.
pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    /// Creates a new atomic `f64`.
    #[inline]
    pub fn new(value: f64) -> Self {
        Self(AtomicU64::new(value.to_bits()))
    }

    /// Stores the given `value` using the given `order`ing.
    #[inline]
    pub fn store(&self, value: f64, order: Ordering) {
        self.0.store(value.to_bits(), order)
    }

    /// Loads the contained `value` using the given `order`ing.
    #[inline]
    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.0.load(order))
    }
}

#[cfg(test)]
mod tests_f64 {
    use super::*;

    #[test]
    fn test_atomic_f64() {
        let value = AtomicF32::new(10.);
        assert_eq!(value.load(Ordering::Relaxed), 10.);

        value.store(15., Ordering::Relaxed);
        assert_eq!(value.load(Ordering::Relaxed), 15.);
    }
}
