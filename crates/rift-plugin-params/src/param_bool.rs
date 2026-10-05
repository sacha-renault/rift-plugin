use std::sync::atomic::{AtomicBool, Ordering};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

#[derive(bon::Builder)]
pub struct BoolParam {
    /// Default value for the param
    #[allow(unused)]
    #[builder(default)]
    default: bool,

    #[builder(skip = AtomicBool::new(default))]
    value: AtomicBool,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    id: ClapId,
}

impl TypedParam for BoolParam {
    type Type = bool;

    fn value(&self) -> Self::Type {
        self.value.load(Ordering::Relaxed)
    }

    #[inline]
    fn set_value(&self, value: Self::Type) {
        self.value.store(value, Ordering::Relaxed);
    }
}

impl crate::traits::__private::Sealed for BoolParam {}

impl Param for BoolParam {
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
        self.set_value(value >= 0.5);
    }

    fn plain(&self) -> f32 {
        if self.value.load(Ordering::Relaxed) {
            1.0
        } else {
            0.0
        }
    }

    fn default_plain(&self) -> f32 {
        if self.default { 1.0 } else { 0.0 }
    }

    fn normalized(&self) -> f32 {
        self.plain()
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(normalized);
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        0.0
    }

    fn max_value(&self) -> f32 {
        1.0
    }

    #[inline]
    fn normalize(&self, value: f32) -> f32 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        value
    }

    #[inline]
    fn denormalize(&self, normalized: f32) -> f32 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        normalized
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = BoolParam::builder()
            .unit("on/off")
            .default(true)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .build();

        assert_eq!(param.unit(), "on/off");
        assert_eq!(param.value(), true);
        assert_eq!(param.default_plain(), 1.0);
        assert_eq!(param.min_value(), 0.0);
        assert_eq!(param.max_value(), 1.0);
        assert_eq!(param.id(), ClapId::new(0));
        assert_eq!(param.name(), "");
        assert_eq!(param.module(), None);
        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn set_value_typed() {
        let param = BoolParam::builder().default(false).build();

        assert_eq!(param.value(), false);
        param.set_value(true);
        assert_eq!(param.value(), true);
    }

    #[test]
    fn set_plain_threshold() {
        let param = BoolParam::builder().default(false).build();

        param.set_plain(0.49);
        assert_eq!(param.value(), false);

        param.set_plain(0.5);
        assert_eq!(param.value(), true);

        param.set_plain(1.0);
        assert_eq!(param.value(), true);

        param.set_plain(0.0);
        assert_eq!(param.value(), false);
    }

    #[test]
    fn plain_returns_0_or_1() {
        let param = BoolParam::builder().default(false).build();
        assert_eq!(param.plain(), 0.0);

        param.set_value(true);
        assert_eq!(param.plain(), 1.0);
    }

    #[test]
    fn ptr_change_param() {
        let param = BoolParam::builder().default(false).build();
        let ptr = param.as_ptr();

        ptr.set_normalized(1.0);
        assert_eq!(param.value(), true);

        ptr.set_normalized(0.0);
        assert_eq!(param.value(), false);
    }
}
