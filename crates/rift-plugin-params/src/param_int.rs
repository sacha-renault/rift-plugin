use std::sync::atomic::{AtomicI32, Ordering};

use crate::atomic_param::{Atomic, AtomicParam, ParamKind};
use crate::scale::Linear;
use crate::{Param, TypedParam};

pub struct _Int;

impl Atomic<i32> for AtomicI32 {
    fn new(v: i32) -> Self {
        Self::new(v)
    }

    fn get(&self) -> i32 {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, v: i32) {
        self.store(v, Ordering::Relaxed);
    }
}

impl ParamKind for _Int {
    type Scale = Linear;
    type Storage = AtomicI32;
    type Value = i32;

    fn to_f32(v: Self::Value) -> f32 {
        v as f32
    }

    fn from_f32(v: f32) -> Self::Value {
        v.round() as i32
    }

    fn default_min() -> Self::Value {
        0
    }

    fn default_max() -> Self::Value {
        1
    }
}

impl TypedParam for IntParam {
    type Type = i32;

    fn value(&self) -> Self::Type {
        _Int::from_f32(self.plain())
    }

    fn set_value(&self, value: Self::Type) {
        // Might be better to use i32 clamping ...
        // ok for now
        self.set_plain(_Int::to_f32(value))
    }
}

pub type IntParam = AtomicParam<_Int>;

#[cfg(test)]
mod tests {
    use clack_extensions::params::ParamInfoFlags;
    use clack_plugin::utils::ClapId;

    use crate::assert_approx_eq;

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = IntParam::builder()
            .unit("st")
            .default(0)
            .min_value(-12)
            .max_value(12)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .build();

        assert_eq!(param.unit(), "st");
        assert_eq!(param.default_plain(), 0.0);
        assert_approx_eq!(param.normalized(), 0.5);
        assert_eq!(param.min_value(), -12.0);
        assert_eq!(param.max_value(), 12.0);
        assert_eq!(param.id(), ClapId::new(0));
        assert_eq!(param.name(), "");
        assert_eq!(param.module(), None);
        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn set_value_typed() {
        let param = IntParam::builder()
            .default(0)
            .min_value(-10)
            .max_value(10)
            .build();

        param.set_value(7);
        assert_eq!(param.value(), 7);
    }

    #[test]
    fn set_plain_clamps() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(5)
            .build();

        param.set_plain(10.0);
        assert_eq!(param.value(), 5);

        param.set_plain(-3.0);
        assert_eq!(param.value(), 0);
    }

    #[test]
    fn set_plain_truncates_float() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(10)
            .build();

        param.set_plain(3.9);
        assert_eq!(param.value(), 4);
    }

    #[test]
    fn normalize_denormalize_roundtrip() {
        let param = IntParam::builder()
            .default(0)
            .min_value(-12)
            .max_value(12)
            .build();

        assert_approx_eq!(param.normalize(0.0), 0.5);
        assert_approx_eq!(param.normalize(-12.0), 0.0);
        assert_approx_eq!(param.normalize(12.0), 1.0);

        assert_approx_eq!(param.denormalize(0.0), -12.0);
        assert_approx_eq!(param.denormalize(0.5), 0.0);
        assert_approx_eq!(param.denormalize(1.0), 12.0);
    }

    #[test]
    fn ptr_change_param() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(10)
            .build();

        let ptr = param.as_ptr();
        ptr.set_normalized(1.0);
        assert_eq!(param.value(), 10);

        ptr.set_normalized(0.0);
        assert_eq!(param.value(), 0);
    }
}
