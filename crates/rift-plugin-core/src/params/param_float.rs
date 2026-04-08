use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;

use super::ptr::ParamPtr;
use super::traits::{__ParamInitializer, ClapParam, TypedParam};

use crate::params::{NamedParam, Persistent};
use crate::utils::atomic_f32::AtomicF32;

#[derive(bon::Builder)]
pub struct FloatParam {
    /// Default value for the param
    #[allow(unused)]
    pub(crate) default: f32,

    #[builder(skip = AtomicF32::new(default))]
    pub(crate) value: AtomicF32,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(skip = String::from(""))]
    name: String,

    #[builder(skip = None)]
    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0.0)]
    pub(crate) min_value: f32,

    #[builder(default = 1.0)]
    pub(crate) max_value: f32,

    #[builder(default)]
    pub(crate) mapping: RangeMapping,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(skip = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for FloatParam {
    type Type = f32;

    fn value(&self) -> Self::Type {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Type) {
        self.value.store(
            value.clamp(self.min_value, self.max_value),
            Ordering::SeqCst,
        );
    }
}

impl NamedParam for FloatParam {
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

impl ClapParam for FloatParam {
    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f32) {
        self.value.store(value, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f32 {
        self.value.load(Ordering::SeqCst)
    }

    fn default_raw(&self) -> f32 {
        self.default
    }

    fn normalized(&self) -> f32 {
        let value = self.get_raw();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_raw(self.denormalize(normalized));
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        self.min_value
    }

    fn max_value(&self) -> f32 {
        self.max_value
    }

    fn normalize(&self, value: f32) -> f32 {
        self.mapping
            .normalize(value, self.min_value, self.max_value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.mapping
            .denormalize(normalized, self.min_value, self.max_value)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn ClapParam)
    }
}

#[derive(Default, Clone, Copy)]
pub enum RangeMapping {
    #[default]
    Linear,

    /// Power curve: more resolution at bottom (skew > 1) or top (skew < 1)
    Skew(f32),
}

impl RangeMapping {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let t = match self {
            Self::Linear => normalized,
            Self::Skew(s) => normalized.powf(*s),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        match self {
            Self::Linear => (value - min) / (max - min),
            Self::Skew(s) => ((value - min) / (max - min)).powf(1.0 / s),
        }
    }
}

impl Persistent for FloatParam {
    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: f32 = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;
        self.set_value(value);
        Ok(())
    }

    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        serde_json::to_writer(writer, &self.value())
            .map_err(|_| PluginError::Message("serialize error"))
    }
}

impl __ParamInitializer for FloatParam {
    #[doc(hidden)]
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
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .mapping(RangeMapping::Linear)
            .build();

        assert_eq!(param.unit(), "dB");
        assert_eq!(param.default_raw(), 0.);
        assert_eq!(param.normalized(), 0.5);
        assert_eq!(param.max_value(), 1.);
        assert_eq!(param.min_value(), -1.);
        assert_eq!(param.id(), ClapId::new(0)); // param aren't initialized at this point.
        assert_eq!(param.name(), ""); // param aren't initialized at this point.
        assert_eq!(param.module(), None); // param aren't initialized at this point.

        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
        assert!(
            !param
                .flags()
                .contains(ParamInfoFlags::IS_AUTOMATABLE_PER_CHANNEL)
        );
    }

    #[test]
    fn ptr_change_param() {
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .build();

        let ptr = param.as_ptr();
        ptr.set_normalized(0.);

        assert_approx_eq!(param.value(), -1.);
    }

    #[test]
    fn set_value_typed_param() {
        let param = FloatParam::builder()
            .unit("dB")
            .default(0.)
            .min_value(-1.)
            .max_value(1.)
            .build();

        param.set_value(0.5);
        assert_approx_eq!(param.value(), 0.5);
        param.set_value(1.5);
        assert_approx_eq!(param.value(), 1.);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let param = FloatParam::builder().default(0.5).build();

        param.set_value(0.75);

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let param2 = FloatParam::builder().default(0.0).build();

        let mut reader = Cursor::new(&buf);
        param2.deserialize(&mut reader).unwrap();

        assert_approx_eq!(param2.value(), 0.75);
    }

    #[test]
    fn test_serialize_default_value() {
        let param = FloatParam::builder().default(0.42).build();

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let param2 = FloatParam::builder().default(0.0).build();

        let mut reader = Cursor::new(&buf);
        param2.deserialize(&mut reader).unwrap();

        assert_approx_eq!(param2.value(), 0.42);
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let param = FloatParam::builder().default(0.5).build();

        let mut reader = Cursor::new(b"not a number");
        assert!(param.deserialize(&mut reader).is_err());
    }

    #[test]
    fn test_initializer() {
        let mut param = FloatParam::builder().default(0.0).build();
        param.__initialize(
            "Gain".to_string(),
            ClapId::new(42),
            Some("mixer".to_string()),
        );

        assert_eq!(param.name, "Gain");
        assert_eq!(param.id, ClapId::new(42));
        assert_eq!(param.module.as_deref(), Some("mixer"));
    }

    #[test]
    fn skew_range_mapping() {
        // Linear sanity check
        let linear = RangeMapping::Linear;
        assert_approx_eq!(linear.denormalize(0.0, 0.0, 100.0), 0.0);
        assert_approx_eq!(linear.denormalize(0.5, 0.0, 100.0), 50.0);
        assert_approx_eq!(linear.denormalize(1.0, 0.0, 100.0), 100.0);

        // Skew: endpoints should always map exactly
        let skew = RangeMapping::Skew(3.0);
        assert_approx_eq!(skew.denormalize(0.0, 20.0, 20000.0), 20.0);
        assert_approx_eq!(skew.denormalize(1.0, 20.0, 20000.0), 20000.0);

        // Skew > 1: midpoint should map below the linear midpoint
        let mid = skew.denormalize(0.5, 0.0, 1000.0);
        assert!(mid < 500.0);

        // Skew < 1: midpoint should map above the linear midpoint
        let skew_inv = RangeMapping::Skew(0.3);
        let mid_inv = skew_inv.denormalize(0.5, 0.0, 1000.0);
        assert!(mid_inv > 500.0);

        // Roundtrip: normalize(denormalize(x)) == x
        for &s in &[0.3_f32, 1.0, 2.0, 3.0] {
            let mapping = RangeMapping::Skew(s);
            for &n in &[0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
                let value = mapping.denormalize(n, 20.0, 20000.0);
                let back = mapping.normalize(value, 20.0, 20000.0);
                assert_approx_eq!(back, n, 1e-5);
            }
        }
    }
}
