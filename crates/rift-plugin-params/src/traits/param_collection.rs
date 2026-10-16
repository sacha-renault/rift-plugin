use std::ffi::CStr;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo};
use clack_plugin::utils::ClapId;

/// Collection trait for accessing parameters in a plugin.
///
/// This trait acts as the bridge between the host and a group of parameters, allowing
/// retrieval of values by ID and batch operations like text formatting.
///
/// It is implemented by the single host-facing wrapper (see
/// [`crate::params_wrapper::ParamsWrapper`]), not by the user's
/// `#[derive(Params)]` structs (those implement [`UserParams`](crate::UserParams)).
pub trait ParamCollection: Sync + Send + 'static {
    /// Get the total count of parameters available.
    fn count(&self) -> u32;

    /// Retrieve metadata for a specific parameter index.
    ///
    /// Note: Host should query param in range 0..self.count()
    fn get_param_info<'a>(&'a self, index: u32) -> Option<ParamInfo<'a>>;

    /// Get the plain value for a parameter by its `ClapId`.
    fn get_value(&self, id: ClapId) -> Option<f32>;

    /// Set the plain value for a parameter.
    fn set_value(&self, id: ClapId, value: f32);

    /// Set the normalized value (0.0–1.0).
    fn set_value_normalized(&self, id: ClapId, value: f32);

    /// Get the value normalized (0.0-1.0).
    fn value_normalized(&self, id: ClapId) -> Option<f32>;

    /// Parse text into a plain value for a specific parameter ID.
    fn text_to_value(&self, id: ClapId, text: &CStr) -> Option<f32>;

    /// Format a plain value into a display buffer.
    fn value_to_text(
        &self,
        id: ClapId,
        value: f32,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result;
}
