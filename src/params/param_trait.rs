use std::ffi::CStr;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags};
use clack_plugin::prelude::*;

pub trait Param {
    type Value;

    // Identity
    fn name(&self) -> &str;

    // Current value
    fn get(&self) -> f64;
    fn set(&self, value: f64);

    // Normalization (CLAP often wants [0.0, 1.0])
    fn normalize(&self, value: Self::Value) -> f64;
    fn denormalize(&self, normalized: f64) -> Self::Value;

    // For automation/modulation
    fn get_normalized(&self) -> f64;
    fn set_normalized(&self, normalized: f64);

    // Display formatting
    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result;

    fn text_to_value(&self, value: &CStr) -> Option<f64>;
    fn flags(&self) -> ParamInfoFlags;
}

pub trait Params: Default + Sync + Send + 'static {
    fn count(&self) -> u32;
    fn get_param_info<'a>(&'a self, index: u32) -> Option<ParamInfo<'a>>;
    fn get_value(&self, id: ClapId) -> Option<f64>;
    fn set_value(&self, id: ClapId, value: f64);
    fn text_to_value(&self, id: ClapId, text: &CStr) -> Option<f64>;
    fn value_to_text(
        &self,
        id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result;
}
