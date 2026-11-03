use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use clack_extensions::params::ParamInfoFlags;

use crate::Scale;
use crate::atomic_param::{Atomic, AtomicParam, ParamKind};

/// A fieldless enum usable as an [`EnumParam`] value.
pub trait EnumValues: fmt::Display + Default + Sized + Copy + Send + Sync + 'static {
    fn to_index(self) -> u32;
    fn from_index(index: u32) -> Option<Self>;
    fn count() -> u32;
}

/// Marker for [`EnumParam`], generic over the enum `E`.
pub struct _Enum<E: EnumValues>(PhantomData<fn() -> E>);

/// Atomic cell holding an enum's index.
pub struct EnumStorage<E: EnumValues>(AtomicU32, PhantomData<fn() -> E>);

impl<E: EnumValues> Atomic<E> for EnumStorage<E> {
    fn new(v: E) -> Self {
        Self(AtomicU32::new(v.to_index()), PhantomData)
    }

    fn get(&self) -> E {
        E::from_index(self.0.load(Ordering::Relaxed)).expect("Bound were not respected")
    }

    fn set(&self, v: E) {
        self.0.store(v.to_index(), Ordering::Relaxed);
    }
}

impl<E: EnumValues> ParamKind for _Enum<E> {
    // The value *is* the enum: that keeps the default a real `E` (so `#[default]`
    // is honored) and lets `TypedParam` hand back `E` directly.
    type Value = E;
    type Storage = EnumStorage<E>;
    type Range = ();

    fn to_f32(v: E) -> f32 {
        v.to_index() as f32
    }

    fn from_f32(v: f32) -> E {
        // `as u32` saturates negative values and NaN to 0, then we clamp the high
        // end to the last variant so an out-of-range plain value still lands on a
        // real variant (rather than the enum's default).
        let index = (v.round() as u32).min(E::count().saturating_sub(1));
        E::from_index(index).expect("Bound were not respected")
    }

    fn min(_: &()) -> f32 {
        0.0
    }

    fn max(_: &()) -> f32 {
        E::count().saturating_sub(1) as f32
    }

    fn normalize(_: &(), value: f32) -> f32 {
        Scale::Linear.normalize(value, 0.0, Self::max(&()))
    }

    fn denormalize(_: &(), normalized: f32) -> f32 {
        Scale::Linear.denormalize(normalized, 0.0, Self::max(&()))
    }

    fn write_text(_: &(), _unit: &str, value: f32, writer: &mut dyn fmt::Write) -> fmt::Result {
        // Reuse the clamped conversion so the printed variant always matches
        // what `set_plain`/`plain` would round-trip.
        write!(writer, "{}", Self::from_f32(value))
    }

    fn default_flags() -> ParamInfoFlags {
        ParamInfoFlags::IS_AUTOMATABLE | ParamInfoFlags::IS_STEPPED | ParamInfoFlags::IS_ENUM
    }
}

pub type EnumParam<E> = AtomicParam<_Enum<E>>;

impl<E: EnumValues> EnumParam<E> {
    pub fn new(default: E) -> Self {
        Self::builder().default(default).build()
    }

    pub fn with_flags(mut self, flags: ParamInfoFlags) -> Self {
        self.flags = flags;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{Param, TypedParam};

    use super::*;

    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    enum TestEnum {
        #[default]
        A,
        B,
        C,
    }

    impl fmt::Display for TestEnum {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::A => write!(f, "A"),
                Self::B => write!(f, "B"),
                Self::C => write!(f, "C"),
            }
        }
    }

    impl EnumValues for TestEnum {
        fn to_index(self) -> u32 {
            self as u32
        }

        fn from_index(index: u32) -> Option<Self> {
            match index {
                0 => Some(Self::A),
                1 => Some(Self::B),
                2 => Some(Self::C),
                _ => None,
            }
        }

        fn count() -> u32 {
            3
        }
    }

    #[test]
    fn default_value() {
        let param = EnumParam::new(TestEnum::B);
        assert_eq!(param.value(), TestEnum::B);
        assert_eq!(param.default_plain(), 1.0);
    }

    #[test]
    fn builder_default_honors_enum_default() {
        // `#[default]` on the enum, not index 0, is what an absent `default = ...`
        // has to resolve to.
        let param = EnumParam::<TestEnum>::builder().build();
        assert_eq!(param.value(), TestEnum::A);
    }

    #[test]
    fn set_value_typed() {
        let param = EnumParam::new(TestEnum::A);
        param.set_value(TestEnum::C);
        assert_eq!(param.value(), TestEnum::C);
    }

    #[test]
    fn set_plain_rounds_to_variant() {
        let param = EnumParam::new(TestEnum::A);
        param.set_plain(1.6);
        assert_eq!(param.value(), TestEnum::C);
    }

    #[test]
    fn set_plain_clamps_out_of_range_to_a_variant() {
        let param = EnumParam::new(TestEnum::A);

        // Out of range must land on the last (or first) variant, never the enum
        // default.
        param.set_plain(100.0);
        assert_eq!(param.value(), TestEnum::C);

        param.set_plain(-100.0);
        assert_eq!(param.value(), TestEnum::A);
    }

    #[test]
    fn min_max_values() {
        let param = EnumParam::new(TestEnum::A);
        assert_eq!(param.min_value(), 0.0);
        assert_eq!(param.max_value(), 2.0);
    }

    #[test]
    fn with_flags() {
        let param = EnumParam::new(TestEnum::A).with_flags(ParamInfoFlags::IS_AUTOMATABLE);

        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn value_to_text() {
        let param = EnumParam::new(TestEnum::A);
        let mut buf = String::new();
        param.value_to_text(2.0, &mut buf).unwrap();
        assert_eq!(buf, "C");
    }

    #[test]
    fn ptr_change_param() {
        let param = EnumParam::new(TestEnum::A);
        let ptr = param.as_ptr();
        ptr.set_normalized(1.0);
        assert_eq!(param.value(), TestEnum::C);
    }
}
