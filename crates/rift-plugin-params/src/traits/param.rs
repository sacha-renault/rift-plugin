use clack_extensions::params::{ParamInfo, ParamInfoFlags};
use clack_plugin::utils::{ClapId, Cookie};

use crate::Tags;
use crate::ptr::ParamPtr;

/// Core abstraction for audio plugin parameters.
///
/// Represents a single control within a plugin (e.g., volume, cutoff).
pub trait Param: super::__private::Sealed {
    /// Get the display name of the parameter (e.g., "Cutoff").
    ///
    /// # Panics
    /// This must return a unique string per plugin instance. Duplicate names will cause host crashes.
    fn name(&self) -> &str;

    /// Get the module of the param if specified (e.g., "Oscillator A").
    fn module(&self) -> Option<&str>;

    /// Get the internal identifier (`ClapId`) for this parameter.
    ///
    /// Unlike `name()`, the `ClapId` is an opaque handle used directly by CLAP internals and must be consistent.
    fn id(&self) -> ClapId;

    /// Get the full path of the param as `"module.name"`, or just `"name"` if no module is set.
    fn path(&self) -> String {
        if let Some(module) = self.module() {
            format!("{}.{}", module, self.name())
        } else {
            self.name().to_string()
        }
    }

    /// Get the unit symbol (e.g., "Hz", "dB", ""). If not applicable, return "".
    ///
    /// The string will be appended automatically to formatted text outputs.
    fn unit(&self) -> &str;

    /// Get the plain, un-normalized value of the parameter.
    ///
    /// Plain values are often used for audio algorithms (e.g., filter cutoff in Hz) before being mapped to UI ranges.
    fn plain(&self) -> f32;

    /// Set the plain, un-normalized value.
    fn set_plain(&self, value: f32);

    /// Get the default plain value.
    fn default_plain(&self) -> f32;

    /// Get the minimum plain value (e.g., 20.0 Hz).
    fn min_value(&self) -> f32;

    /// Get the maximum plain value (e.g., 20000.0 Hz).
    fn max_value(&self) -> f32;

    /// Convert a plain value to its normalized range [0.0, 1.0].
    ///
    /// # Behavior
    /// * Values outside `[min_value(), max_value()]` are clamped or undefined depending on implementation.
    fn normalized(&self) -> f32;

    /// Set the normalized value [0.0, 1.0].
    ///
    /// # Warning
    /// If you set a value outside `[0.0, 1.0]`, the behavior is undefined. Always clamp inputs or use `set_plain()`.
    fn set_normalized(&self, normalized: f32);

    /// Format a plain value into text with optional unit suffix.
    ///
    /// By default, this simply writes `{value}{unit}`. Custom implementations should handle rounding and special cases (e.g., "120.00 Hz" vs "120 Hz").
    fn value_to_text(&self, value: f32, writer: &mut dyn core::fmt::Write) -> std::fmt::Result;

    /// Format the current plain parameter value into a `String`.
    fn to_text(&self) -> String {
        let mut s = String::new();
        self.value_to_text(self.plain(), &mut s).ok();
        s
    }

    /// Parse a text string back into a plain value.
    ///
    /// # Parsing Rules
    /// 1. The input string is trimmed and checked to end with the result of `unit()`.
    /// 2. If the suffix matches, it is stripped, and the remaining part is parsed as `f32`.
    /// 3. Returns `None` if the string does not match the unit or fails to parse.
    fn text_to_value(&self, value: &std::ffi::CStr) -> Option<f32> {
        let str_val = value.to_str().ok()?.trim();
        let unit = self.unit();
        if !str_val.ends_with(unit) {
            None
        } else {
            let no_unit_val = str_val.strip_suffix(unit).unwrap_or(str_val);
            no_unit_val.trim().parse::<f32>().ok()
        }
    }

    /// Get flags describing the parameter's properties.
    fn flags(&self) -> ParamInfoFlags;

    /// Apply the normalization tension to a plain value.
    ///
    /// Generally equivalent to `normalized()` but allows manual conversion.
    fn normalize(&self, value: f32) -> f32;

    /// Inverse of `normalize()`. Converts [0.0, 1.0] back to the plain scale.
    fn denormalize(&self, normalized: f32) -> f32;

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
            module: self.module().unwrap_or("").as_bytes(),
            min_value: self.min_value() as f64,
            max_value: self.max_value() as f64,
            default_value: self.default_plain() as f64,
        }
    }

    /// Get a pointer to the parameter. That's basically for type erasure
    fn as_ptr(&self) -> ParamPtr;

    /// Get the tag associated to this param
    ///
    /// Tags is a very generic struct holding a u32.
    /// Using a single bittags! is probably the best ...
    fn tags(&self) -> Tags;
}
