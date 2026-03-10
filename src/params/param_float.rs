use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::_sealed::__ParamInitializer;
use crate::prelude::{ClapParam, ParamPtr, TypedParam};

use super::atomic_f32::AtomicF32;

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
    pub(crate) min_value: f64,

    #[builder(default = 1.0)]
    pub(crate) max_value: f64,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for FloatParam {
    type Value = f32;

    fn value(&self) -> Self::Value {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Value) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl ClapParam for FloatParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> &str {
        self.module.as_deref().unwrap_or("")
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f64) {
        self.value.store(value as f32, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f64 {
        self.value.load(Ordering::SeqCst) as f64
    }

    fn default_raw(&self) -> f64 {
        self.default as f64
    }

    fn get_normalized(&self) -> f64 {
        let value = self.get_raw();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f64) {
        self.set_raw(self.denormalize(normalized));
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn normalize(&self, value: f64) -> f64 {
        let range = self.max_value - self.min_value;
        (value - self.min_value) / range
    }

    fn denormalize(&self, normalized: f64) -> f64 {
        let range = self.max_value - self.min_value;
        normalized * range + self.min_value
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn ClapParam)
    }
}

impl __ParamInitializer for FloatParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}
