use std::sync::atomic::{AtomicBool, Ordering};

use clack_extensions::params::*;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;

use crate::params::{NamedParam, Persistent};

use super::ptr::ParamPtr;
use super::traits::{__ParamInitializer, ClapParam, TypedParam};

#[derive(bon::Builder)]
pub struct BoolParam {
    /// Default value for the param
    #[allow(unused)]
    default: bool,

    #[builder(skip = AtomicBool::new(default))]
    value: AtomicBool,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    id: ClapId,
}

impl TypedParam for BoolParam {
    type Type = bool;

    fn value(&self) -> Self::Type {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Type) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl NamedParam for BoolParam {
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

impl ClapParam for BoolParam {
    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f32) {
        self.value.store(value >= 0.5, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f32 {
        if self.value.load(Ordering::SeqCst) {
            1.0
        } else {
            0.0
        }
    }

    fn default_raw(&self) -> f32 {
        if self.default { 1.0 } else { 0.0 }
    }

    fn normalized(&self) -> f32 {
        self.get_raw()
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_raw(normalized);
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        0.0
    }

    fn max_value(&self) -> f32 {
        1.0
    }

    #[inline]
    fn normalize(&self, value: f32) -> f32 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        value
    }

    #[inline]
    fn denormalize(&self, normalized: f32) -> f32 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        normalized
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn ClapParam)
    }
}

impl Persistent for BoolParam {
    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: bool = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;
        self.set_value(value);
        Ok(())
    }

    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        serde_json::to_writer(writer, &self.value())
            .map_err(|_| PluginError::Message("serialize error"))
    }
}

impl __ParamInitializer for BoolParam {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.name = name;
        self.id = id;
        self.module = module;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = BoolParam::builder()
            .unit("on/off")
            .default(true)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .build();

        assert_eq!(param.unit(), "on/off");
        assert_eq!(param.value(), true);
        assert_eq!(param.default_raw(), 1.0);
        assert_eq!(param.min_value(), 0.0);
        assert_eq!(param.max_value(), 1.0);
        assert_eq!(param.id(), ClapId::new(0));
        assert_eq!(param.name(), "");
        assert_eq!(param.module(), None);
        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn set_value_typed() {
        let param = BoolParam::builder().default(false).build();

        assert_eq!(param.value(), false);
        param.set_value(true);
        assert_eq!(param.value(), true);
    }

    #[test]
    fn set_raw_threshold() {
        let param = BoolParam::builder().default(false).build();

        param.set_raw(0.49);
        assert_eq!(param.value(), false);

        param.set_raw(0.5);
        assert_eq!(param.value(), true);

        param.set_raw(1.0);
        assert_eq!(param.value(), true);

        param.set_raw(0.0);
        assert_eq!(param.value(), false);
    }

    #[test]
    fn get_raw_returns_0_or_1() {
        let param = BoolParam::builder().default(false).build();
        assert_eq!(param.get_raw(), 0.0);

        param.set_value(true);
        assert_eq!(param.get_raw(), 1.0);
    }

    #[test]
    fn ptr_change_param() {
        let param = BoolParam::builder().default(false).build();
        let ptr = param.as_ptr();

        ptr.set_normalized(1.0);
        assert_eq!(param.value(), true);

        ptr.set_normalized(0.0);
        assert_eq!(param.value(), false);
    }

    #[test]
    fn serialize_roundtrip() {
        let param = BoolParam::builder().default(false).build();
        param.set_value(true);

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let param2 = BoolParam::builder().default(false).build();
        let mut reader = Cursor::new(&buf);
        param2.deserialize(&mut reader).unwrap();

        assert_eq!(param2.value(), true);
    }

    #[test]
    fn serialize_default_value() {
        let param = BoolParam::builder().default(true).build();

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let param2 = BoolParam::builder().default(false).build();
        let mut reader = Cursor::new(&buf);
        param2.deserialize(&mut reader).unwrap();

        assert_eq!(param2.value(), true);
    }

    #[test]
    fn deserialize_invalid_data() {
        let param = BoolParam::builder().default(false).build();
        let mut reader = Cursor::new(b"not a bool");
        assert!(param.deserialize(&mut reader).is_err());
    }

    #[test]
    fn initializer() {
        let mut param = BoolParam::builder().default(false).build();
        param.__initialize(
            "Bypass".to_string(),
            ClapId::new(99),
            Some("fx".to_string()),
        );

        assert_eq!(param.name, "Bypass");
        assert_eq!(param.id, ClapId::new(99));
        assert_eq!(param.module.as_deref(), Some("fx"));
    }
}
