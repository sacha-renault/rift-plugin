use std::sync::atomic::{AtomicI32, Ordering};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::params::param_ptr::ParamPtr;
use crate::params::param_trait::__ParamInitializer;
use crate::prelude::ClapParam;

use super::param_trait::TypedParam;

#[derive(bon::Builder)]
pub struct IntParam {
    /// Default value for the param
    #[allow(unused)]
    pub(crate) default: i32,

    #[builder(skip = AtomicI32::new(default))]
    pub(crate) value: AtomicI32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0)]
    pub(crate) min_value: i32,

    #[builder(default = 1)]
    pub(crate) max_value: i32,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for IntParam {
    type Value = i32;

    fn value(&self) -> Self::Value {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Value) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl ClapParam for IntParam {
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
        let int_value = (value as i32).clamp(self.min_value, self.max_value);
        self.value.store(int_value, Ordering::SeqCst);
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

    fn min_value(&self) -> f64 {
        self.min_value as f64
    }

    fn max_value(&self) -> f64 {
        self.max_value as f64
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn normalize(&self, value: f64) -> f64 {
        let range = (self.max_value - self.min_value) as f64;
        (value - self.min_value as f64) / range
    }

    fn denormalize(&self, normalized: f64) -> f64 {
        let range = (self.max_value - self.min_value) as f64;
        normalized * range + self.min_value as f64
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr {
            ptr: self as *const dyn ClapParam,
        }
    }
}

impl __ParamInitializer for IntParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}
