use std::ops::Deref;

use fundsp::prelude32::Shared;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

use crate::RangeMapping;

pub struct SharedFloatParam {
    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    name: String,
    module: Option<String>,

    pub(crate) default: f32,
    pub(crate) value: Shared,
    pub(crate) unit: &'static str,
    pub(crate) min_value: f32,
    pub(crate) max_value: f32,
    pub(crate) mapping: RangeMapping,
    pub(crate) flags: ParamInfoFlags,
    pub(crate) id: ClapId,
}

impl SharedFloatParam {
    pub fn create(
        id: ClapId,
        name: String,
        module: Option<String>,
        config: SharedFloatParamConfig,
    ) -> Self {
        let SharedFloatParamConfig {
            default,
            unit,
            min,
            max,
            mapping,
            flags,
        } = config;

        Self {
            default,
            value: Shared::new(default),
            name,
            module,
            unit,
            min_value: min,
            max_value: max,
            mapping,
            flags,
            id,
        }
    }
}

#[derive(Debug)]
pub struct SharedFloatParamConfig {
    pub default: f32,
    pub unit: &'static str,
    pub min: f32,
    pub max: f32,
    pub mapping: RangeMapping,
    pub flags: ParamInfoFlags,
}

impl Default for SharedFloatParamConfig {
    fn default() -> Self {
        Self {
            default: 0.0,
            unit: "",
            min: 0.0,
            max: 1.0,
            mapping: RangeMapping::Linear,
            flags: ParamInfoFlags::IS_AUTOMATABLE,
        }
    }
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

    fn set_raw(&self, value: f32) {
        self.value.set(value);
    }

    fn get_raw(&self) -> f32 {
        self.value.value()
    }

    fn default_raw(&self) -> f32 {
        self.default
    }

    fn normalized(&self) -> f32 {
        let value = self.get_raw();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_raw(self.denormalize(normalized));
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
