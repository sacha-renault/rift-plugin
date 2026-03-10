use std::sync::atomic::{AtomicBool, Ordering};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::_sealed::__ParamInitializer;
use crate::prelude::{ClapParam, ParamPtr, TypedParam};

#[derive(bon::Builder)]
pub struct BoolParam {
    /// Default value for the param
    #[allow(unused)]
    default: bool,

    #[builder(skip = AtomicBool::new(default))]
    value: AtomicBool,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    id: ClapId,
}

impl TypedParam for BoolParam {
    type Value = bool;

    fn value(&self) -> Self::Value {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Value) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl ClapParam for BoolParam {
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
        self.value.store(value >= 0.5, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f64 {
        if self.value.load(Ordering::SeqCst) {
            1.0
        } else {
            0.0
        }
    }

    fn default_raw(&self) -> f64 {
        if self.default { 1.0 } else { 0.0 }
    }

    fn get_normalized(&self) -> f64 {
        self.get_raw()
    }

    fn set_normalized(&self, normalized: f64) {
        self.set_raw(normalized);
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f64 {
        0.0
    }

    fn max_value(&self) -> f64 {
        1.0
    }

    #[inline]
    fn normalize(&self, value: f64) -> f64 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        value
    }

    #[inline]
    fn denormalize(&self, normalized: f64) -> f64 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        normalized
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn ClapParam)
    }
}
impl __ParamInitializer for BoolParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}
