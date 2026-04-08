use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{__ParamInitializer, ClapParam, TypedParam};

use crate::params::NamedParam;
use crate::utils::atomic_f32::AtomicF32;

#[derive(bon::Builder)]
pub struct FloatParam {
    /// Default value for the param
    #[allow(unused)]
    pub(crate) default: f32,

    #[builder(skip = AtomicF32::new(default))]
    pub(crate) value: AtomicF32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

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

    #[builder(skip = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for FloatParam {
    type ValueType = f32;

    fn value(&self) -> Self::ValueType {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::ValueType) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl NamedParam for FloatParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }
}

impl ClapParam for FloatParam {
    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f32) {
        self.value.store(value, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f32 {
        self.value.load(Ordering::SeqCst)
    }

    fn default_raw(&self) -> f32 {
        self.default
    }

    fn get_normalized(&self) -> f32 {
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
        ParamPtr::new(self as *const dyn ClapParam)
    }
}

#[derive(Default, Clone, Copy)]
pub enum RangeMapping {
    #[default]
    Linear,

    /// Power curve: more resolution at bottom (skew > 1) or top (skew < 1)
    Skew(f32),
}

impl RangeMapping {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let t = match self {
            Self::Linear => normalized,
            Self::Skew(s) => normalized.powf(*s),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        match self {
            Self::Linear => (value - min) / (max - min),
            Self::Skew(s) => ((value - min) / (max - min)).powf(1.0 / s),
        }
    }
}

impl super::Persistent for FloatParam {
    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: f32 = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;
        self.set_value(value);
        Ok(())
    }

    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        serde_json::to_writer(writer, &self.value())
            .map_err(|_| PluginError::Message("serialize error"))
    }
}

impl __ParamInitializer for FloatParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}
