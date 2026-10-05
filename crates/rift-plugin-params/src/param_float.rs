use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

use crate::Scale;
use crate::atomic_floats::AtomicF32;

#[derive(bon::Builder)]
pub struct FloatParam {
    #[allow(unused)]
    #[builder(default)]
    pub(crate) default: f32,

    #[builder(skip = AtomicF32::new(default))]
    pub(crate) value: AtomicF32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0.0)]
    pub(crate) min_value: f32,

    #[builder(default = 1.0)]
    pub(crate) max_value: f32,

    #[builder(default)]
    pub(crate) scale: Scale,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for FloatParam {
    type Type = f32;

    fn value(&self) -> Self::Type {
        self.value.load(Ordering::Relaxed)
    }

    #[inline]
    fn set_value(&self, value: Self::Type) {
        self.value.store(
            value.clamp(self.min_value, self.max_value),
            Ordering::Relaxed,
        );
    }
}

impl crate::traits::__private::Sealed for FloatParam {}

impl Param for FloatParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit(&self) -> &str {
        self.unit
    }

    fn set_plain(&self, value: f32) {
        self.set_value(value);
    }

    fn plain(&self) -> f32 {
        self.value.load(Ordering::Relaxed)
    }

    fn default_plain(&self) -> f32 {
        self.default
    }

    fn normalized(&self) -> f32 {
        let value = self.plain();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(self.denormalize(normalized));
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        self.min_value
    }

    fn max_value(&self) -> f32 {
        self.max_value
    }

    fn normalize(&self, value: f32) -> f32 {
        self.scale.normalize(value, self.min_value, self.max_value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.scale
            .denormalize(normalized, self.min_value, self.max_value)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_approx_eq;

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

    #[test]
    fn skew_range_mapping() {
        // Linear sanity check
        let linear = Scale::Linear;
        assert_approx_eq!(linear.denormalize(0.0, 0.0, 100.0), 0.0);
        assert_approx_eq!(linear.denormalize(0.5, 0.0, 100.0), 50.0);
        assert_approx_eq!(linear.denormalize(1.0, 0.0, 100.0), 100.0);

        // Skew: endpoints should always map exactly
        let skew = Scale::Skew(3.0);
        assert_approx_eq!(skew.denormalize(0.0, 20.0, 20000.0), 20.0);
        assert_approx_eq!(skew.denormalize(1.0, 20.0, 20000.0), 20000.0);

        // Skew > 1: midpoint should map below the linear midpoint
        let mid = skew.denormalize(0.5, 0.0, 1000.0);
        assert!(mid < 500.0);

        // Skew < 1: midpoint should map above the linear midpoint
        let skew_inv = Scale::Skew(0.3);
        let mid_inv = skew_inv.denormalize(0.5, 0.0, 1000.0);
        assert!(mid_inv > 500.0);

        // Roundtrip: normalize(denormalize(x)) == x
        for &s in &[0.3_f32, 1.0, 2.0, 3.0] {
            let mapping = Scale::Skew(s);
            for &n in &[0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
                let value = mapping.denormalize(n, 20.0, 20000.0);
                let back = mapping.normalize(value, 20.0, 20000.0);
                assert_approx_eq!(back, n, 1e-5);
            }
        }
    }
}
