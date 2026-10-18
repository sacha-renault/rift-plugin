use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering},
};

use bitflags::Flags;

/// Primitive bit type usable as the backing storage of [`AtomicFlags`].
pub trait AtomicBits: Copy {
    type Storage: Default;
    type Bits;

    fn new(bits: Self::Bits) -> Self::Storage;
    fn store(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering);
    fn load(storage: &Self::Storage, ordering: Ordering) -> Self::Bits;
    fn fetch_and(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn fetch_or(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
}

macro_rules! impl_atomic_bits {
    ($bits:ty, $atomic:ty) => {
        impl AtomicBits for $bits {
            type Storage = $atomic;
            type Bits = $bits;

            fn new(bits: Self::Bits) -> Self::Storage {
                <$atomic>::new(bits)
            }

            fn store(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) {
                <$atomic>::store(storage, bits, ordering);
            }

            fn load(storage: &Self::Storage, ordering: Ordering) -> Self::Bits {
                <$atomic>::load(storage, ordering)
            }

            fn fetch_and(
                storage: &Self::Storage,
                bits: Self::Bits,
                ordering: Ordering,
            ) -> Self::Bits {
                <$atomic>::fetch_and(storage, bits, ordering)
            }

            fn fetch_or(
                storage: &Self::Storage,
                bits: Self::Bits,
                ordering: Ordering,
            ) -> Self::Bits {
                <$atomic>::fetch_or(storage, bits, ordering)
            }
        }
    };
}

impl_atomic_bits!(u8, AtomicU8);
impl_atomic_bits!(u16, AtomicU16);
impl_atomic_bits!(u32, AtomicU32);
impl_atomic_bits!(u64, AtomicU64);

pub struct AtomicFlags<T, F>
where
    T: AtomicBits,
    F: Flags<Bits = T::Bits>,
    F: Copy,
{
    inner: T::Storage,
    _p: PhantomData<fn() -> F>,
}

impl<F> AtomicFlags<F::Bits, F>
where
    F: Flags,
    F::Bits: AtomicBits<Bits = F::Bits>,
    F: Copy,
{
    pub fn new(flags: F) -> Self {
        Self {
            inner: F::Bits::new(flags.bits()),
            _p: PhantomData,
        }
    }

    pub fn store(&self, flags: F, ordering: Ordering) {
        F::Bits::store(&self.inner, flags.bits(), ordering);
    }

    pub fn load(&self, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::load(&self.inner, ordering))
    }

    pub fn fetch_and(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_and(&self.inner, flags.bits(), ordering))
    }

    pub fn fetch_or(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_or(&self.inner, flags.bits(), ordering))
    }

    pub fn fetch_clear(&self, flags: F, ordering: Ordering) -> F {
        let previous = self.fetch_and(flags.complement(), ordering);
        previous.intersection(flags)
    }

    /// Returns which of the requested flags are currently set, without clearing them.
    pub fn peek(&self, flags: F, ordering: Ordering) -> F {
        self.load(ordering).intersection(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitflags::bitflags;

    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct TestFlags: u8 {
            const A = 0b0001;
            const B = 0b0010;
            const C = 0b0100;
        }
    }

    #[test]
    fn store_and_load() {
        let flags = AtomicFlags::<u8, TestFlags>::new(TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A);

        flags.store(TestFlags::B, Ordering::SeqCst);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
    }

    #[test]
    fn fetch_clear_clears_requested_flags_and_returns_cleared() {
        let flags = AtomicFlags::<u8, TestFlags>::new(TestFlags::A | TestFlags::B);

        let cleared = flags.fetch_clear(TestFlags::A, Ordering::SeqCst);
        assert_eq!(cleared, TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

        // Taking a flag that isn't set is a no-op.
        let cleared = flags.fetch_clear(TestFlags::A, Ordering::SeqCst);
        assert_eq!(cleared, TestFlags::empty());
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

        let cleared = flags.fetch_clear(TestFlags::A | TestFlags::B, Ordering::SeqCst);
        assert_eq!(cleared, TestFlags::B);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());
    }

    #[test]
    fn peek_returns_requested_flags_without_clearing() {
        let flags = AtomicFlags::<u8, TestFlags>::new(TestFlags::A | TestFlags::B);

        assert_eq!(flags.peek(TestFlags::A, Ordering::SeqCst), TestFlags::A);
        assert_eq!(
            flags.peek(TestFlags::A | TestFlags::B | TestFlags::C, Ordering::SeqCst),
            TestFlags::A | TestFlags::B
        );
        assert_eq!(
            flags.peek(TestFlags::C, Ordering::SeqCst),
            TestFlags::empty()
        );

        // Peeking doesn't clear anything.
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);
    }

    #[test]
    fn fetch_operations() {
        let flags = AtomicFlags::<u8, TestFlags>::new(TestFlags::A);

        let previous = flags.fetch_or(TestFlags::B, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);

        let previous = flags.fetch_and(!TestFlags::A, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A | TestFlags::B);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
    }
}
