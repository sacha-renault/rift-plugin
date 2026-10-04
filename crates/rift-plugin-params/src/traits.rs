use std::{ffi::CStr, io::Read, io::Write};

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags};
use clack_plugin::{prelude::*, utils::Cookie};

use super::ptr::ParamPtr;

/// Core abstraction for audio plugin parameters.
///
/// Represents a single control within a plugin (e.g., volume, cutoff).
pub trait Param: __private::Sealed {
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
    fn value_to_text(&self, value: f32, writer: &mut dyn core::fmt::Write) -> std::fmt::Result {
        write!(writer, "{}{}", value, self.unit())
    }

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

    /// Get a closure that can normalize any f32
    fn normalize_fn(&self) -> Box<dyn Fn(f32) -> f32>;

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
}

/// A generic trait for parameters that expose their underlying type at compile time.
///
/// Useful when you need to interact with parameters of specific types (e.g., `f32` vs `bool`)
/// without casting, allowing for strongly-typed parameter sets in the UI layer.
pub trait TypedParam {
    /// The concrete type of the value held by this parameter.
    type Type;

    fn value(&self) -> Self::Type;

    fn set_value(&self, value: Self::Type);
}

/// Trait for plugin state persistence (preset save/load). This represent any param that
/// cannot be represented as a float. Host will never know about this parameter. This is all internal
/// in this plugin.
///
/// Each implementor is responsible for writing a single valid JSON value to the writer and reading it back.
pub trait Persistent {
    fn create(id: ClapId, name: String, module: Option<String>) -> Self;
    fn name(&self) -> &str;
    fn module(&self) -> Option<&str>;
    fn path(&self) -> String {
        if let Some(module) = self.module() {
            format!("{}.{}", module, self.name())
        } else {
            self.name().to_string()
        }
    }
    fn id(&self) -> ClapId;
    fn serialize(&self, writer: &mut dyn Write) -> Result<(), PluginError>;
    fn deserialize(&self, reader: &mut dyn Read) -> Result<(), PluginError>;
}

/// Implemented by the `#[derive(Params)]` structs written by the user.
///
/// It exposes every leaf parameter reachable from the struct (walking nested
/// structs and arrays) as a type-erased [`ParamPtr`], which the wrapper then
/// collects into a [`ParamCollection`] map. It also owns the (de)serialization
/// of both the parameter values and any extra `#[persist]` state.
///
/// Only [`UserParams::all_params`] has to be implemented by the derive; the
/// parameter methods below are provided from it. The persist methods are
/// defaulted no-ops and get overridden once `#[persist]` fields exist.
pub trait UserParams {
    /// Every parameter reachable from this struct, in declaration order.
    fn all_params(&self) -> Vec<ParamPtr>;

    /// Write every reachable parameter into `out`, keyed by its stable
    /// [`ClapId`], as its plain value.
    ///
    /// [`ClapId`]: clack_plugin::utils::ClapId
    fn serialize_params(
        &self,
        out: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        for param in self.all_params() {
            // Keyed by the stable `ClapId` rather than by name, so that renaming
            // a label does not invalidate previously saved state.
            out.insert(
                param.id().get().to_string(),
                serde_json::Value::from(param.plain() as f64),
            );
        }

        Ok(())
    }

    /// Read parameters from `section`, keyed by their stable [`ClapId`].
    ///
    /// Missing, unknown and non-numeric entries are ignored: this keeps state
    /// loadable across plugin updates that add or remove parameters.
    ///
    /// [`ClapId`]: clack_plugin::utils::ClapId
    fn deserialize_params(
        &self,
        section: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        for param in self.all_params() {
            let key = param.id().get().to_string();

            if let Some(value) = section.get(&key).and_then(serde_json::Value::as_f64) {
                param.set_plain(value as f32);
            }
        }

        Ok(())
    }

    /// Write extra, non-parameter state into `out`.
    ///
    /// Defaulted to a no-op; `#[derive(Params)]` overrides it once `#[persist]`
    /// fields are supported.
    fn serialize_persist(
        &self,
        _out: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// Read extra, non-parameter state from `section`.
    ///
    /// Defaulted to a no-op; `#[derive(Params)]` overrides it once `#[persist]`
    /// fields are supported.
    fn deserialize_persist(
        &self,
        _section: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Collection trait for accessing parameters in a plugin.
///
/// This trait acts as the bridge between the host and a group of parameters, allowing
/// retrieval of values by ID and batch operations like text formatting.
///
/// It is implemented by the single host-facing wrapper (see
/// [`crate::params_wrapper::ParamsWrapper`]), not by the user's
/// `#[derive(Params)]` structs (those implement [`UserParams`]).
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

#[doc(hidden)]
pub(crate) mod __private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FloatParam;
    use clack_plugin::utils::ClapId;

    /// A hand-written stand-in for a `#[derive(Params)]` struct.
    struct MockParams {
        gain: FloatParam,
        level: FloatParam,
    }

    impl UserParams for MockParams {
        fn all_params(&self) -> Vec<ParamPtr> {
            vec![self.gain.as_ptr(), self.level.as_ptr()]
        }
    }

    fn mock() -> MockParams {
        let build = |id: u32, name: &str| {
            FloatParam::builder()
                .id(ClapId::new(id))
                .name(name.to_string())
                .min_value(0.0)
                .max_value(1.0)
                .build()
        };

        MockParams {
            gain: build(1, "gain"),
            level: build(2, "level"),
        }
    }

    #[test]
    fn params_roundtrip() {
        let params = mock();
        params.gain.set_plain(0.25);
        params.level.set_plain(0.75);

        let mut section = serde_json::Map::new();
        params.serialize_params(&mut section).unwrap();

        let restored = mock();
        restored.deserialize_params(&section).unwrap();

        assert_eq!(restored.gain.plain(), 0.25);
        assert_eq!(restored.level.plain(), 0.75);
    }

    #[test]
    fn missing_and_unknown_keys_are_tolerated() {
        let params = mock();
        params.level.set_plain(0.7);

        // Key "1" (gain) is present, key "2" (level) is absent, and "999" is unknown.
        let section: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"1": 0.5, "999": 0.123}"#).unwrap();

        params.deserialize_params(&section).unwrap();

        assert_eq!(params.gain.plain(), 0.5);
        // An absent key leaves the parameter untouched rather than resetting it.
        assert_eq!(params.level.plain(), 0.7);
    }

    #[test]
    fn persist_sections_default_to_empty_noops() {
        let params = mock();
        params.gain.set_plain(0.42);

        let mut section = serde_json::Map::new();
        params.serialize_persist(&mut section).unwrap();
        assert!(section.is_empty());

        // Deserializing an unrelated section is a no-op and does not error.
        let section: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"anything": 1}"#).unwrap();
        params.deserialize_persist(&section).unwrap();
        assert_eq!(params.gain.plain(), 0.42);
    }
}
