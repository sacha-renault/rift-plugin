use std::sync::atomic::{AtomicI32, Ordering};

use crate::Scale;
use crate::atomic_param::{Atomic, AtomicParam, ParamKind};
use crate::range::IntRange;

/// Marker for [`IntParam`].
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
    type Value = i32;
    type Storage = AtomicI32;
    type Range = IntRange;

    fn to_f32(v: Self::Value) -> f32 {
        v as f32
    }

    fn from_f32(v: f32) -> Self::Value {
        v.round() as i32
    }

    fn min(range: &IntRange) -> f32 {
        range.min as f32
    }

    fn max(range: &IntRange) -> f32 {
        range.max as f32
    }

    fn normalize(range: &IntRange, value: f32) -> f32 {
        Scale::Linear.normalize(value, range.min as f32, range.max as f32)
    }

    fn denormalize(range: &IntRange, normalized: f32) -> f32 {
        Scale::Linear.denormalize(normalized, range.min as f32, range.max as f32)
    }

    fn clamp(range: &IntRange, value: i32) -> i32 {
        value.clamp(range.min, range.max)
    }
}

pub type IntParam = AtomicParam<_Int>;

#[cfg(test)]
mod tests {
    use clack_extensions::params::ParamInfoFlags;
    use clack_plugin::utils::ClapId;

    use crate::{Param, TypedParam, assert_approx_eq};

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = IntParam::builder()
            .unit("st")
            .default(0)
            .range(IntRange::linear(-12, 12))
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
            .range(IntRange::linear(-10, 10))
            .build();

        param.set_value(7);
        assert_eq!(param.value(), 7);
    }

    #[test]
    fn set_value_keeps_full_precision() {
        let param = IntParam::builder()
            .default(0)
            .range(IntRange::linear(0, 20_000_000))
            .build();

        // 2^24 + 1 is not representable in `f32`; the typed path must not
        // round-trip the value through it.
        param.set_value(16_777_217);
        assert_eq!(param.value(), 16_777_217);
    }

    #[test]
    fn set_plain_clamps() {
        let param = IntParam::builder()
            .default(0)
            .range(IntRange::linear(0, 5))
            .build();

        param.set_plain(10.0);
        assert_eq!(param.value(), 5);

        param.set_plain(-3.0);
        assert_eq!(param.value(), 0);
    }

    #[test]
    fn set_plain_clamps_large_values_in_the_int_domain() {
        let param = IntParam::builder()
            .default(0)
            .range(IntRange::linear(0, 1_000_000))
            .build();

        param.set_plain(1e30);
        assert_eq!(param.value(), 1_000_000);

        param.set_plain(-1e30);
        assert_eq!(param.value(), 0);
    }

    #[test]
    fn set_plain_rounds_float() {
        let param = IntParam::builder()
            .default(0)
            .range(IntRange::linear(0, 10))
            .build();

        param.set_plain(3.9);
        assert_eq!(param.value(), 4);
    }

    #[test]
    fn normalize_denormalize_roundtrip() {
        let param = IntParam::builder()
            .default(0)
            .range(IntRange::linear(-12, 12))
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
            .range(IntRange::linear(0, 10))
            .build();

        let ptr = param.as_ptr();
        ptr.set_normalized(1.0);
        assert_eq!(param.value(), 10);

        ptr.set_normalized(0.0);
        assert_eq!(param.value(), 0);
    }
}
