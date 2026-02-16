use std::sync::atomic::Ordering;

use clack_extensions::params::ParamDisplayWriter;

use super::atomic_f32::AtomicF32;
use super::param_trait::Param;

#[derive(bon::Builder)]
pub struct FloatParam {
    /// Default value for the param
    default: f32,

    #[builder(default = AtomicF32::new(default))]
    value: AtomicF32,
    name: String,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = 0.0)]
    min_value: f64,

    #[builder(default = 1.0)]
    max_value: f64,
}

impl Param for FloatParam {
    type Value = f64;

    fn name(&self) -> &str {
        &self.name
    }

    fn get(&self) -> f64 {
        self.value.load(Ordering::SeqCst) as f64
    }

    fn set(&self, value: f64) {
        self.value.store(value as f32, Ordering::SeqCst);
    }

    fn normalize(&self, value: Self::Value) -> f64 {
        let range = self.max_value - self.min_value;
        (value - self.min_value) / range
    }

    fn denormalize(&self, normalized: f64) -> Self::Value {
        let range = self.max_value - self.min_value;
        normalized * range + self.min_value
    }

    fn get_normalized(&self) -> f64 {
        let value = self.get();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f64) {
        self.set(self.denormalize(normalized));
    }

    fn text_to_value(&self, value: &std::ffi::CStr) -> Option<f64> {
        None // todo!()
    }

    fn value_to_text(
        &mut self,
        param_id: clack_plugin::prelude::ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result {
        std::fmt::Result::Ok(()) // todo!()
    }

    fn flags(&self) -> clack_extensions::params::ParamInfoFlags {
        clack_extensions::params::ParamInfoFlags::empty() // todo!()
    }
}
