use std::ffi::CStr;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags};
use clack_plugin::{prelude::*, utils::Cookie};

use super::ptr::ParamPtr;

/// Core abstraction for audio plugin parameters.
///
/// Represents a single control within a plugin (e.g., volume, cutoff).
///
/// # Safety & Contracts
///
/// * **Name Uniqueness**: `name()` must return a unique identifier string across all parameters in the plugin instance. Violation causes crashes.
/// * **ID Uniqueness**: `id()` returns the internal CLAP handle (`ClapId`) which is used by the host to address this parameter. It must be stable for the lifetime of the plugin.
pub trait ClapParam {
    /// Get the display name of the parameter (e.g., "Cutoff").
    ///
    /// # Panics
    /// This must return a unique string per plugin instance. Duplicate names will cause host crashes.
    fn name(&self) -> &str;

    /// Get the module of the param if specified (e.g., "Oscillator A").
    ///
    /// Can be an empty string
    fn module(&self) -> &str;

    /// Get the internal identifier (`ClapId`) for this parameter.
    ///
    /// Unlike `name()`, the `ClapId` is an opaque handle used directly by CLAP internals and must be consistent.
    fn id(&self) -> ClapId;

    /// Get the unit symbol (e.g., "Hz", "dB", ""). If not applicable, return "".
    ///
    /// The string will be appended automatically to formatted text outputs.
    fn unit(&self) -> &str;

    /// Get the raw, un-normalized value of the parameter.
    ///
    /// Raw values are often used for audio algorithms (e.g., filter cutoff in Hz) before being mapped to UI ranges.
    fn get_raw(&self) -> f64;

    /// Set the raw, un-normalized value.
    fn set_raw(&self, value: f64);

    /// Get the default raw value.
    fn default_raw(&self) -> f64;

    /// Get the minimum raw value (e.g., 20.0 Hz).
    fn min_value(&self) -> f64;

    /// Get the maximum raw value (e.g., 20000.0 Hz).
    fn max_value(&self) -> f64;

    /// Convert a raw value to its normalized range [0.0, 1.0].
    ///
    /// # Behavior
    /// * Values outside `[min_value(), max_value()]` are clamped or undefined depending on implementation.
    fn get_normalized(&self) -> f64;

    /// Set the normalized value [0.0, 1.0].
    ///
    /// # Warning
    /// If you set a value outside `[0.0, 1.0]`, the behavior is undefined. Always clamp inputs or use `set_raw()`.
    fn set_normalized(&self, normalized: f64);

    /// Format a raw value into text with optional unit suffix.
    ///
    /// By default, this simply writes `{value}{unit}`. Custom implementations should handle rounding and special cases (e.g., "120.00 Hz" vs "120 Hz").
    fn value_to_text(&self, value: f64, writer: &mut dyn core::fmt::Write) -> std::fmt::Result {
        write!(writer, "{}{}", value, self.unit())
    }

    /// Format the current raw parameter value into a `String`.
    fn to_text(&self) -> String {
        let mut s = String::new();
        self.value_to_text(self.get_raw(), &mut s).ok();
        s
    }

    /// Parse a text string back into a raw value.
    ///
    /// # Parsing Rules
    /// 1. The input string is trimmed and checked to end with the result of `unit()`.
    /// 2. If the suffix matches, it is stripped, and the remaining part is parsed as `f64`.
    /// 3. Returns `None` if the string does not match the unit or fails to parse.
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

    /// Get flags describing the parameter's properties.
    fn flags(&self) -> ParamInfoFlags;

    /// Apply the normalization curve to a raw value.
    ///
    /// Generally equivalent to `get_normalized()` but allows manual conversion.
    fn normalize(&self, value: f64) -> f64;

    /// Inverse of `normalize()`. Converts [0.0, 1.0] back to the raw scale.
    fn denormalize(&self, normalized: f64) -> f64;

    /// Build the complete [`ParamInfo`] struct for this parameter.
    ///
    /// This includes metadata like flags and a cookie (empty by default).
    /// Used internally by CLAP to register parameter state.
    fn param_info<'a>(&'a self) -> ParamInfo<'a> {
        ParamInfo {
            id: self.id(),
            flags: self.flags(),
            cookie: Cookie::empty(),
            name: self.name().as_bytes(),
            module: self.module().as_bytes(),
            min_value: self.min_value(),
            max_value: self.max_value(),
            default_value: self.default_raw(),
        }
    }

    /// Get a pointer to the parameter. That's basically for type erasure
    fn as_ptr(&self) -> ParamPtr;
}

/// A generic trait for parameters that expose their underlying type at compile time.
///
/// Useful when you need to interact with parameters of specific types (e.g., `f32` vs `bool`)
/// without casting, allowing for strongly-typed parameter sets in the UI layer.
pub trait TypedParam {
    /// The concrete type of the value held by this parameter.
    type Value;

    fn value(&self) -> Self::Value;
    fn set_value(&self, value: Self::Value);
}

/// Collection trait for accessing parameters in a plugin.
///
/// This trait acts as the bridge between the host and a group of parameters, allowing
/// retrieval of values by ID and batch operations like text formatting.
/// There is a derive macro that implement this automatically : [`crate::prelude::DeriveParams`]
pub trait Params: Sync + Send + 'static {
    /// Get the total count of parameters available.
    fn count(&self) -> u32;

    /// Retrieve metadata for a specific parameter index.
    ///
    /// Note: Host should query param in range 0..self.count()
    fn get_param_info<'a>(&'a self, index: u32) -> Option<ParamInfo<'a>>;

    /// Get the raw value for a parameter by its `ClapId`.
    fn get_value(&self, id: ClapId) -> Option<f64>;

    /// Set the raw value for a parameter.
    fn set_value(&self, id: ClapId, value: f64);

    /// Set the normalized value (0.0–1.0).
    fn set_value_normalized(&self, id: ClapId, value: f64);

    /// Parse text into a raw value for a specific parameter ID.
    fn text_to_value(&self, id: ClapId, text: &CStr) -> Option<f64>;

    /// Format a raw value into a display buffer.
    fn value_to_text(
        &self,
        id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result;
}

#[doc(hidden)]
pub trait __ParamInitializer {
    /// This function isn't meant to be called from client side
    /// it's public but is called once at param creation and then NEVER
    /// Any way, no mutable reference of Params is ever shared since it's wrapped
    /// in Arc. Don't try to do weird thing to mutate this.
    #[doc(hidden)]
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>);
}

#[doc(hidden)]
pub trait __ParamsInitializer {
    #[doc(hidden)]
    fn __initialize(&mut self);
}
