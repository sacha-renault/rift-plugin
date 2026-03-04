use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::params::param_ptr::ParamPtr;
use crate::prelude::ClapParam;
use crate::utils::id_generator::get_next_param_id;

use super::atomic_f32::AtomicF32;
use super::param_trait::TypedParam;

#[derive(bon::Builder)]
pub struct FloatParam {
    /// Default value for the param
    #[allow(unused)]
    pub(crate) default: f32,

    #[builder(skip = AtomicF32::new(default))]
    pub(crate) value: AtomicF32,

    /// Name of the param
    pub(crate) name: &'static str,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0.0)]
    pub(crate) min_value: f64,

    #[builder(default = 1.0)]
    pub(crate) max_value: f64,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(skip = get_next_param_id())]
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
        self.name
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
        ParamPtr {
            ptr: self as *const dyn ClapParam,
        }
    }
}
