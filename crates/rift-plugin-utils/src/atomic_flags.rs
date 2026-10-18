use std::{
    marker::PhantomData,
    sync::atomic::{
        AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16, AtomicU32,
        AtomicU64, AtomicUsize, Ordering,
    },
};

use bitflags::Flags;

/// Primitive bit type usable as the backing storage of [`AtomicFlags`].
pub trait AtomicBits: Copy {
    type Storage;
    type Bits;

    fn new(bits: Self::Bits) -> Self::Storage;
    fn store(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering);
    fn load(storage: &Self::Storage, ordering: Ordering) -> Self::Bits;
    fn fetch_and(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn fetch_or(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn fetch_xor(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn fetch_nand(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn swap(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits;
    fn into_inner(storage: Self::Storage) -> Self::Bits;
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

            fn fetch_xor(
                storage: &Self::Storage,
                bits: Self::Bits,
                ordering: Ordering,
            ) -> Self::Bits {
                <$atomic>::fetch_xor(storage, bits, ordering)
            }

            fn fetch_nand(
                storage: &Self::Storage,
                bits: Self::Bits,
                ordering: Ordering,
            ) -> Self::Bits {
                <$atomic>::fetch_nand(storage, bits, ordering)
            }

            fn swap(storage: &Self::Storage, bits: Self::Bits, ordering: Ordering) -> Self::Bits {
                <$atomic>::swap(storage, bits, ordering)
            }

            fn into_inner(storage: Self::Storage) -> Self::Bits {
                storage.into_inner()
            }
        }
    };
}

// Bitflag crate implement Flag for almost all integer
// types:
//     u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
// So this implement AtomicBits for every type possible. There isn't
// 128 bit atomic implementation in rust.
impl_atomic_bits!(u8, AtomicU8);
impl_atomic_bits!(u16, AtomicU16);
impl_atomic_bits!(u32, AtomicU32);
impl_atomic_bits!(u64, AtomicU64);
impl_atomic_bits!(usize, AtomicUsize);
impl_atomic_bits!(i8, AtomicI8);
impl_atomic_bits!(i16, AtomicI16);
impl_atomic_bits!(i32, AtomicI32);
impl_atomic_bits!(i64, AtomicI64);
impl_atomic_bits!(isize, AtomicIsize);

pub struct AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits<Bits = F::Bits>,
    F: Copy,
{
    storage: <F::Bits as AtomicBits>::Storage,
    _p: PhantomData<fn() -> F>,
}

impl<F> AtomicFlags<F>
where
    F: Flags,
    F::Bits: AtomicBits<Bits = F::Bits>,
    F: Copy,
{
    pub fn new(flags: F) -> Self {
        Self {
            storage: F::Bits::new(flags.bits()),
            _p: PhantomData,
        }
    }

    pub fn store(&self, flags: F, ordering: Ordering) {
        F::Bits::store(&self.storage, flags.bits(), ordering);
    }

    pub fn load(&self, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::load(&self.storage, ordering))
    }

    pub fn fetch_and(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_and(&self.storage, flags.bits(), ordering))
    }

    pub fn fetch_or(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_or(&self.storage, flags.bits(), ordering))
    }

    pub fn fetch_xor(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_xor(&self.storage, flags.bits(), ordering))
    }

    pub fn fetch_nand(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::fetch_nand(&self.storage, flags.bits(), ordering))
    }

    pub fn swap(&self, flags: F, ordering: Ordering) -> F {
        F::from_bits_retain(F::Bits::swap(&self.storage, flags.bits(), ordering))
    }

    /// Removes the requested flags and returns the ones that were set.
    ///
    /// Empty if none of `flags` were set.
    pub fn take(&self, flags: F, ordering: Ordering) -> F {
        let previous = self.fetch_and(flags.complement(), ordering);
        previous.intersection(flags)
    }

    /// Returns which of the requested flags are currently set, without clearing them.
    pub fn peek(&self, flags: F, ordering: Ordering) -> F {
        self.load(ordering).intersection(flags)
    }

    /// Clears all bits.
    pub fn clear(&self, ordering: Ordering) {
        self.store(F::empty(), ordering);
    }

    pub fn into_inner(self) -> F {
        F::from_bits_retain(F::Bits::into_inner(self.storage))
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
        let flags = AtomicFlags::new(TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A);

        flags.store(TestFlags::B, Ordering::SeqCst);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
    }

    #[test]
    fn take_removes_requested_flags_and_returns_taken() {
        let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

        let taken = flags.take(TestFlags::A, Ordering::SeqCst);
        assert_eq!(taken, TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

        // Taking a flag that isn't set is a no-op.
        let taken = flags.take(TestFlags::A, Ordering::SeqCst);
        assert_eq!(taken, TestFlags::empty());
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);

        let taken = flags.take(TestFlags::A | TestFlags::B, Ordering::SeqCst);
        assert_eq!(taken, TestFlags::B);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());
    }

    #[test]
    fn peek_returns_requested_flags_without_clearing() {
        let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

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
        let flags = AtomicFlags::new(TestFlags::A);

        let previous = flags.fetch_or(TestFlags::B, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::B);

        let previous = flags.fetch_and(!TestFlags::A, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A | TestFlags::B);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
    }

    #[test]
    fn fetch_xor_and_swap_return_previous_state() {
        let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

        let previous = flags.fetch_xor(TestFlags::B | TestFlags::C, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A | TestFlags::B);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::A | TestFlags::C);

        let previous = flags.swap(TestFlags::B, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A | TestFlags::C);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::B);
    }

    #[test]
    fn fetch_nand_operates_on_the_raw_word() {
        let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);

        let previous = flags.fetch_nand(TestFlags::A, Ordering::SeqCst);
        assert_eq!(previous, TestFlags::A | TestFlags::B);

        // NAND affects bits outside the mask too: !(0b011 & 0b001) = 0xFE.
        assert_eq!(flags.load(Ordering::SeqCst).bits(), !TestFlags::A.bits());
    }

    #[test]
    fn clear_and_into_inner() {
        let flags = AtomicFlags::new(TestFlags::A | TestFlags::B);
        flags.clear(Ordering::SeqCst);
        assert_eq!(flags.load(Ordering::SeqCst), TestFlags::empty());

        let flags = AtomicFlags::new(TestFlags::A | TestFlags::C);
        assert_eq!(flags.into_inner(), TestFlags::A | TestFlags::C);
    }
}
