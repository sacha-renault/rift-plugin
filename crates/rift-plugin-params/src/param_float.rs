use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::atomic_param::{Atomic, AtomicParam, ParamKind};
use crate::{Param, Scale, TypedParam};

pub struct _Float<S>(PhantomData<fn() -> S>);

impl Atomic<f32> for AtomicU32 {
    fn new(v: f32) -> Self {
        Self::new(v.to_bits())
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.load(Ordering::Relaxed))
    }

    fn set(&self, v: f32) {
        self.store(v.to_bits(), Ordering::Relaxed);
    }
}

impl<S: Atomic<f32> + 'static> ParamKind for _Float<S> {
    type Scale = Scale;
    type Storage = S;
    type Value = f32;

    fn from_f32(v: f32) -> Self::Value {
        v
    }

    fn to_f32(v: Self::Value) -> f32 {
        v
    }

    fn default_min() -> Self::Value {
        0.
    }

    fn default_max() -> Self::Value {
        1.
    }
}

impl TypedParam for FloatParam {
    type Type = f32;

    fn value(&self) -> Self::Type {
        self.plain()
    }

    fn set_value(&self, value: Self::Type) {
        self.set_plain(value);
    }
}

pub type FloatParam = AtomicParam<_Float<AtomicU32>>;

#[cfg(test)]
mod tests {
    use clack_extensions::params::ParamInfoFlags;
    use clack_plugin::utils::ClapId;

    use crate::{Param, assert_approx_eq};

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .scale(Scale::Linear)
            .build();

        assert_eq!(param.unit(), "dB");
        assert_eq!(param.default_plain(), 0.);
        assert_eq!(param.normalized(), 0.5);
        assert_eq!(param.max_value(), 1.);
        assert_eq!(param.min_value(), -1.);
        assert_eq!(param.id(), ClapId::new(0)); // param aren't initialized at this point.
        assert_eq!(param.name(), ""); // param aren't initialized at this point.
        assert_eq!(param.module(), None); // param aren't initialized at this point.

        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
        assert!(
            !param
                .flags()
                .contains(ParamInfoFlags::IS_AUTOMATABLE_PER_CHANNEL)
        );
    }

    #[test]
    fn ptr_change_param() {
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .build();

        let ptr = param.as_ptr();
        ptr.set_normalized(0.);

        assert_approx_eq!(param.value(), -1.);
    }

    #[test]
    fn set_value_typed_param() {
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .build();

        param.set_value(0.5);
        assert_approx_eq!(param.value(), 0.5);
        param.set_value(1.5);
        assert_approx_eq!(param.value(), 1.);
    }
}
