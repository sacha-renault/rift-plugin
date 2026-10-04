use std::ops::Deref;

use fundsp::prelude32::Shared;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

use crate::RangeMapping;

#[derive(bon::Builder)]
pub struct SharedFloatParam {
    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default)]
    pub(crate) default: f32,

    #[builder(skip = Shared::new(default))]
    pub(crate) value: Shared,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0.0)]
    pub(crate) min_value: f32,

    #[builder(default = 1.0)]
    pub(crate) max_value: f32,

    #[builder(default)]
    pub(crate) mapping: RangeMapping,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for SharedFloatParam {
    type Type = f32;

    fn value(&self) -> Self::Type {
        self.value.value()
    }

    fn set_value(&self, value: Self::Type) {
        self.value.set(value.clamp(self.min_value, self.max_value));
    }
}

impl crate::traits::__private::Sealed for SharedFloatParam {}

impl Param for SharedFloatParam {
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
        self.value.set(value);
    }

    fn plain(&self) -> f32 {
        self.value.value()
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
        self.mapping
            .normalize(value, self.min_value, self.max_value)
    }

    fn normalize_fn(&self) -> Box<dyn Fn(f32) -> f32> {
        let mapping = self.mapping;
        let min = self.min_value;
        let max = self.max_value;
        Box::new(move |value| mapping.normalize(value, min, max))
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.mapping
            .denormalize(normalized, self.min_value, self.max_value)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }
}

impl Deref for SharedFloatParam {
    type Target = Shared;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
