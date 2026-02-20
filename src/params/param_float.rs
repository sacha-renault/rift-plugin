use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::utils::id_generator::get_next_param_id;

use super::atomic_f32::AtomicF32;
use super::param_trait::InnerParam;

#[derive(bon::Builder)]
pub struct FloatParam {
    /// Default value for the param
    #[allow(unused)] // This is actually used for init value
    default: f32,

    #[builder(skip = AtomicF32::new(default))]
    value: AtomicF32,

    /// Name of the param
    name: &'static str,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = 0.0)]
    min_value: f64,

    #[builder(default = 1.0)]
    max_value: f64,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    flags: ParamInfoFlags,

    #[builder(default = get_next_param_id())]
    id: ClapId,
}

impl InnerParam for FloatParam {
    type Value = f32;

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit<'a>(&'a self) -> &'a str {
        self.unit
    }

    fn get(&self) -> f64 {
        self.value.load(Ordering::SeqCst) as f64
    }

    fn value(&self) -> Self::Value {
        self.value.load(Ordering::SeqCst)
    }

    fn set(&self, value: f64) {
        self.value.store(value as f32, Ordering::SeqCst);
    }

    fn normalize(&self, value: Self::Value) -> f64 {
        let range = self.max_value - self.min_value;
        let vf64 = value as f64;
        (vf64 - self.min_value) / range
    }

    fn denormalize(&self, normalized: f64) -> Self::Value {
        let range = self.max_value - self.min_value;
        let denorm = normalized * range + self.min_value;
        denorm as f32
    }

    fn get_normalized(&self) -> f64 {
        let value = self.value();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f64) {
        self.set(self.denormalize(normalized) as f64);
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }
}
