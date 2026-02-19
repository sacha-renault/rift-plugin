use std::ffi::CStr;
use std::fmt::Write;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags};
use clack_plugin::prelude::*;

use crate::gui::ParamGuiEvent;

pub trait InnerParam {
    type Value;

    // Identity
    fn name(&self) -> &str;
    fn id(&self) -> ClapId;
    fn unit<'a>(&'a self) -> &'a str;

    // Current value
    fn value(&self) -> Self::Value;
    fn get(&self) -> f64;
    fn set(&self, value: f64);

    // Normalization
    fn normalize(&self, value: Self::Value) -> f64;
    fn denormalize(&self, normalized: f64) -> Self::Value;

    // For automation/modulation
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

    /// Add an event into the queue that says the user has made parameter events
    /// This must be add from GUI and read from audio thread
    fn add_gui_event(&self, event: ParamGuiEvent);
    fn process_event<F>(&self, func: F) -> usize
    where
        F: FnMut(ParamGuiEvent);
}
