use std::ffi::CStr;
use std::fmt::Write;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags};
use clack_plugin::prelude::*;

pub trait ClapParam {
    // Identity
    fn name(&self) -> &str;
    fn id(&self) -> ClapId;
    fn unit(&self) -> &str;

    fn get_raw(&self) -> f64;
    fn set_raw(&self, value: f64);

    fn get_normalized(&self) -> f64;
    fn set_normalized(&self, normalized: f64);

    // Display formatting
    fn value_to_text(&self, value: f64, writer: &mut ParamDisplayWriter) -> std::fmt::Result {
        write!(writer, "{}{}", value, self.unit())
    }

    fn text_to_value(&self, value: &std::ffi::CStr) -> Option<f64> {
        let str_val = value.to_str().ok()?.trim();
        let unit = self.unit();
        if !str_val.ends_with(unit) {
            None
        } else {
            let no_unit_val = str_val.strip_suffix(unit).unwrap_or(str_val);
            no_unit_val.trim().parse::<f64>().ok()
        }
    }

    fn flags(&self) -> ParamInfoFlags;

    fn normalize(&self, value: f64) -> f64;
    fn denormalize(&self, normalized: f64) -> f64;
}

pub trait TypedParam {
    type Value;

    // Current value
    fn value(&self) -> Self::Value;
    fn set_value(&self, value: Self::Value);
}

pub trait Params: Sync + Send + 'static {
    fn count(&self) -> u32;
    fn get_param_info<'a>(&'a self, index: u32) -> Option<ParamInfo<'a>>;
    fn get_value(&self, id: ClapId) -> Option<f64>;
    fn set_value(&self, id: ClapId, value: f64);
    fn set_value_normalized(&self, id: ClapId, value: f64);
    fn text_to_value(&self, id: ClapId, text: &CStr) -> Option<f64>;
    fn value_to_text(
        &self,
        id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result;
}
