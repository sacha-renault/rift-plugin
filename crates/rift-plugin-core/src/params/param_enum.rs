use std::marker::PhantomData;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::param_int::IntParam;
use super::ptr::ParamPtr;
use super::traits::{__ParamInitializer, ClapParam, TypedParam};

pub trait EnumValues: std::fmt::Display + Default + Sized + Copy + 'static {
    fn to_index(self) -> u32;
    fn from_index(index: u32) -> Option<Self>;
    fn count() -> u32;
}

pub struct EnumParam<E: EnumValues> {
    inner: IntParam,
    _p: PhantomData<E>,
}

impl<E: EnumValues> EnumParam<E> {
    /// Notes: IMPORTANT, default flags are EMPTY
    pub fn new(default: E) -> Self {
        let total = E::count() as i32;
        let default = default.to_index() as i32;
        let inner = IntParam::builder()
            .default(default)
            .max_value(total - 1)
            // Kinda have to make them empty, since
            // We use union in the with_ setter, otherwise we have problems
            .flags(ParamInfoFlags::empty())
            .build();
        Self {
            inner,
            _p: PhantomData,
        }
    }

    pub fn with_flags(mut self, param_flags: ParamInfoFlags) -> Self {
        self.inner.flags = self.inner.flags.union(param_flags);
        self
    }
}

impl<E: EnumValues> TypedParam for EnumParam<E> {
    type Value = E;

    fn set_value(&self, value: Self::Value) {
        self.set_raw(value.to_index() as f64);
    }

    fn value(&self) -> Self::Value {
        let enum_idx = self.get_raw().round() as u32;
        if let Some(v) = E::from_index(enum_idx) {
            v
        } else {
            // This isn't supposed to ever happen
            let msg = "E::from_index in EnumParam::value returned variant NONE. This is not supposed to ever happen. FIx this";

            if cfg!(debug_assertions) {
                panic!("{msg}")
            } else {
                log::error!("{msg}");
                E::default()
            }
        }
    }
}

impl<E: EnumValues> ClapParam for EnumParam<E> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn module(&self) -> &str {
        self.inner.module()
    }

    fn id(&self) -> ClapId {
        self.inner.id()
    }

    fn unit(&self) -> &str {
        self.inner.unit()
    }

    fn get_raw(&self) -> f64 {
        self.inner.get_raw()
    }

    fn get_normalized(&self) -> f64 {
        self.inner.get_normalized()
    }

    fn default_raw(&self) -> f64 {
        self.inner.default_raw()
    }

    fn flags(&self) -> ParamInfoFlags {
        self.inner.flags()
    }

    fn normalize(&self, value: f64) -> f64 {
        self.inner.normalize(value)
    }

    fn denormalize(&self, normalized: f64) -> f64 {
        self.inner.denormalize(normalized)
    }

    fn min_value(&self) -> f64 {
        self.inner.min_value as f64
    }

    fn max_value(&self) -> f64 {
        self.inner.max_value as f64
    }

    fn set_raw(&self, value: f64) {
        self.inner.set_raw(value);
    }

    fn set_normalized(&self, normalized: f64) {
        self.inner.set_normalized(normalized);
    }

    fn as_ptr(&self) -> ParamPtr {
        self.inner.as_ptr()
    }

    fn value_to_text(&self, value: f64, writer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let variant = E::from_index(value.round() as u32).unwrap_or_default();
        writer.write_str(&format!("{variant}"))
    }
}

impl<E: EnumValues> __ParamInitializer for EnumParam<E> {
    fn __initialize(&mut self, name: String, id: ClapId, module: Option<String>) {
        self.inner.__initialize(name, id, module);
    }
}
