use std::sync::atomic::{AtomicI32, Ordering};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

#[derive(bon::Builder)]
pub struct IntParam {
    /// Default value for the param
    #[allow(unused)]
    #[builder(default)]
    pub(crate) default: i32,

    #[builder(skip = AtomicI32::new(default))]
    pub(crate) value: AtomicI32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0)]
    pub(crate) min_value: i32,

    #[builder(default = 1)]
    pub(crate) max_value: i32,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for IntParam {
    type Type = i32;

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

impl crate::traits::__private::Sealed for IntParam {}

impl Param for IntParam {
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
        self.set_value(value as i32);
    }

    fn plain(&self) -> f32 {
        self.value.load(Ordering::Relaxed) as f32
    }

    fn default_plain(&self) -> f32 {
        self.default as f32
    }

    fn normalized(&self) -> f32 {
        let value = self.plain();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(self.denormalize(normalized));
    }

    fn min_value(&self) -> f32 {
        self.min_value as f32
    }

    fn max_value(&self) -> f32 {
        self.max_value as f32
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn normalize(&self, value: f32) -> f32 {
        let range = (self.max_value - self.min_value) as f32;
        (value - self.min_value as f32) / range
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        let range = (self.max_value - self.min_value) as f32;
        normalized * range + self.min_value as f32
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
        assert_eq!(param.value(), 3);
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
