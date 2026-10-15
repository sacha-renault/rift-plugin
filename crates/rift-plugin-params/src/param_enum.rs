use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use clack_extensions::params::ParamInfoFlags;

use crate::atomic_param::{Atomic, AtomicParam, ParamKind};
use crate::scale::Linear;

use super::traits::TypedParam;

pub trait EnumValues: std::fmt::Display + Default + Sized + Copy + 'static {
    fn to_index(self) -> u32;
    fn from_index(index: u32) -> Option<Self>;
    fn count() -> u32;
}

pub struct _Enum<E: EnumValues>(PhantomData<fn() -> E>);

impl Atomic<u32> for AtomicU32 {
    fn new(v: u32) -> Self {
        Self::new(v)
    }

    fn get(&self) -> u32 {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, v: u32) {
        self.store(v, Ordering::Relaxed);
    }
}

impl<E: EnumValues> ParamKind for _Enum<E> {
    type Scale = Linear;
    type Storage = AtomicU32;
    type Value = u32;

    fn to_f32(v: Self::Value) -> f32 {
        v as f32
    }

    fn from_f32(v: f32) -> Self::Value {
        v.round() as u32
    }

    fn default_min() -> Self::Value {
        0
    }

    fn default_max() -> Self::Value {
        E::count() - 1
    }
}

impl<E: EnumValues> TypedParam for EnumParam<E> {
    type Type = E;

    fn value(&self) -> Self::Type {
        let index = self.storage.get();
        match E::from_index(index) {
            Some(v) => v,
            None => {
                debug_assert!(false, "Enum was out of bound");
                E::default()
            }
        }
    }

    fn set_value(&self, value: Self::Type) {
        self.storage.set(value.to_index());
    }
}

impl<E: EnumValues> EnumParam<E> {
    pub fn new(default: E) -> Self {
        Self::builder().default(default.to_index()).build()
    }

    pub fn with_flags(mut self, flags: ParamInfoFlags) -> Self {
        self.flags = flags;
        self
    }
}

pub type EnumParam<E> = AtomicParam<_Enum<E>>;

#[cfg(test)]
mod tests {

    use crate::Param;

    use super::*;

    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    enum TestEnum {
        #[default]
        A,
        B,
        C,
    }

    impl std::fmt::Display for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
