use std::sync::atomic::{AtomicI32, Ordering};

use clack_extensions::params::*;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;

use crate::params::{NamedParam, Persistent};

use super::ptr::ParamPtr;
use super::traits::{__ParamInitializer, ClapParam, TypedParam};

#[derive(bon::Builder)]
pub struct IntParam {
    /// Default value for the param
    #[allow(unused)]
    pub(crate) default: i32,

    #[builder(skip = AtomicI32::new(default))]
    pub(crate) value: AtomicI32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0)]
    pub(crate) min_value: i32,

    #[builder(default = 1)]
    pub(crate) max_value: i32,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for IntParam {
    type Type = i32;

    fn value(&self) -> Self::Type {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Type) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl NamedParam for IntParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }
}

impl ClapParam for IntParam {
    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f32) {
        let int_value = (value as i32).clamp(self.min_value, self.max_value);
        self.value.store(int_value, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f32 {
        self.value.load(Ordering::SeqCst) as f32
    }

    fn default_raw(&self) -> f32 {
        self.default as f32
    }

    fn normalized(&self) -> f32 {
        let value = self.get_raw();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_raw(self.denormalize(normalized));
    }

    fn min_value(&self) -> f32 {
        self.min_value as f32
    }

    fn max_value(&self) -> f32 {
        self.max_value as f32
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn normalize(&self, value: f32) -> f32 {
        let range = (self.max_value - self.min_value) as f32;
        (value - self.min_value as f32) / range
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        let range = (self.max_value - self.min_value) as f32;
        normalized * range + self.min_value as f32
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn ClapParam)
    }
}

impl Persistent for IntParam {
    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: i32 = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;
        self.set_value(value);
        Ok(())
    }

    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        serde_json::to_writer(writer, &self.value())
            .map_err(|_| PluginError::Message("serialize error"))
    }
}

impl __ParamInitializer for IntParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::assert_approx_eq;

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = IntParam::builder()
            .unit("st")
            .default(0)
            .min_value(-12)
            .max_value(12)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .build();

        assert_eq!(param.unit(), "st");
        assert_eq!(param.default_raw(), 0.0);
        assert_approx_eq!(param.normalized(), 0.5);
        assert_eq!(param.min_value(), -12.0);
        assert_eq!(param.max_value(), 12.0);
        assert_eq!(param.id(), ClapId::new(0));
        assert_eq!(param.name(), "");
        assert_eq!(param.module(), None);
        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn set_value_typed() {
        let param = IntParam::builder()
            .default(0)
            .min_value(-10)
            .max_value(10)
            .build();

        param.set_value(7);
        assert_eq!(param.value(), 7);
    }

    #[test]
    fn set_raw_clamps() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(5)
            .build();

        param.set_raw(10.0);
        assert_eq!(param.value(), 5);

        param.set_raw(-3.0);
        assert_eq!(param.value(), 0);
    }

    #[test]
    fn set_raw_truncates_float() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(10)
            .build();

        param.set_raw(3.9);
        assert_eq!(param.value(), 3);
    }

    #[test]
    fn normalize_denormalize_roundtrip() {
        let param = IntParam::builder()
            .default(0)
            .min_value(-12)
            .max_value(12)
            .build();

        assert_approx_eq!(param.normalize(0.0), 0.5);
        assert_approx_eq!(param.normalize(-12.0), 0.0);
        assert_approx_eq!(param.normalize(12.0), 1.0);

        assert_approx_eq!(param.denormalize(0.0), -12.0);
        assert_approx_eq!(param.denormalize(0.5), 0.0);
        assert_approx_eq!(param.denormalize(1.0), 12.0);
    }

    #[test]
    fn ptr_change_param() {
        let param = IntParam::builder()
            .default(0)
            .min_value(0)
            .max_value(10)
            .build();

        let ptr = param.as_ptr();
        ptr.set_normalized(1.0);
        assert_eq!(param.value(), 10);

        ptr.set_normalized(0.0);
        assert_eq!(param.value(), 0);
    }

    #[test]
    fn serialize_roundtrip() {
        let param = IntParam::builder()
            .default(0)
            .min_value(-100)
            .max_value(100)
            .build();

        param.set_value(42);

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let param2 = IntParam::builder()
            .default(0)
            .min_value(-100)
            .max_value(100)
            .build();

        let mut reader = Cursor::new(&buf);
        param2.deserialize(&mut reader).unwrap();

        assert_eq!(param2.value(), 42);
    }

    #[test]
    fn deserialize_invalid_data() {
        let param = IntParam::builder().default(0).build();
        let mut reader = Cursor::new(b"not a number");
        assert!(param.deserialize(&mut reader).is_err());
    }

    #[test]
    fn initializer() {
        let mut param = IntParam::builder().default(0).build();
        param.__initialize(
            "Semitones".to_string(),
            ClapId::new(13),
            Some("pitch".to_string()),
        );

        assert_eq!(param.name, "Semitones");
        assert_eq!(param.id, ClapId::new(13));
        assert_eq!(param.module.as_deref(), Some("pitch"));
    }
}
