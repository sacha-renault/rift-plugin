use std::fmt;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::Tags;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

/// Lock-free cell holding a `T`.
pub trait Atomic<T>: Send + Sync {
    fn new(v: T) -> Self;
    fn get(&self) -> T;
    fn set(&self, v: T);
}

/// A parameter flavour: its value type, how it is stored, and its configuration.
pub trait ParamKind: 'static {
    /// The value the plugin and the GUI manipulate (e.g. `f32`, `bool`, an enum).
    type Value: Copy + Default + fmt::Display + Send + Sync + 'static;

    /// The lock-free cell backing [`Self::Value`].
    type Storage: Atomic<Self::Value>;

    /// Kind-specific configuration, or `()` when the kind needs none.
    type Range: Default + Send + Sync;

    /// Convert a value to the host-facing plain `f32`.
    fn to_f32(v: Self::Value) -> f32;

    /// Convert a plain `f32` back to a value.
    fn from_f32(v: f32) -> Self::Value;

    /// Lower bound of the plain range.
    fn min(range: &Self::Range) -> f32;

    /// Upper bound of the plain range.
    fn max(range: &Self::Range) -> f32;

    /// Map a plain value to `[0, 1]`.
    fn normalize(range: &Self::Range, value: f32) -> f32;

    /// Map a `[0, 1]` value back to the plain range.
    fn denormalize(range: &Self::Range, normalized: f32) -> f32;

    /// Clamp `value` to the plain range, in the value's own domain.
    fn clamp(_range: &Self::Range, value: Self::Value) -> Self::Value {
        value
    }

    /// Format a plain value for the host, appending `unit` where relevant.
    fn write_text(
        range: &Self::Range,
        unit: &str,
        value: f32,
        writer: &mut dyn fmt::Write,
    ) -> fmt::Result {
        let _ = range;
        write!(writer, "{value}{unit}")
    }
}

/// A parameter, generic over its [`ParamKind`].
#[derive(bon::Builder)]
pub struct AtomicParam<K: ParamKind> {
    #[builder(default)]
    pub(crate) default: K::Value,

    #[builder(skip = <K::Storage as Atomic<K::Value>>::new(default))]
    pub(crate) storage: K::Storage,

    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    pub(crate) name: String,
    pub(crate) module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    /// Kind-specific configuration; `()` for kinds without one.
    #[builder(default)]
    pub(crate) range: K::Range,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,

    #[builder(default, with = |v: impl Into<Tags>| v.into())]
    pub(crate) tags: Tags,
}

impl<K: ParamKind> crate::traits::__private::Sealed for AtomicParam<K> {}

impl<K: ParamKind> TypedParam for AtomicParam<K> {
    type Type = K::Value;

    fn value(&self) -> Self::Type {
        self.storage.get()
    }

    fn set_value(&self, value: Self::Type) {
        self.storage.set(K::clamp(&self.range, value));
    }
}

impl<K: ParamKind> Param for AtomicParam<K> {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit(&self) -> &str {
        self.unit
    }

    fn set_plain(&self, value: f32) {
        let v = K::from_f32(value.clamp(self.min_value(), self.max_value()));
        self.storage.set(v);
    }

    fn plain(&self) -> f32 {
        K::to_f32(self.storage.get())
    }

    fn default_plain(&self) -> f32 {
        K::to_f32(self.default)
    }

    fn normalized(&self) -> f32 {
        self.normalize(self.plain())
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(self.denormalize(normalized));
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        K::min(&self.range)
    }

    fn max_value(&self) -> f32 {
        K::max(&self.range)
    }

    fn normalize(&self, value: f32) -> f32 {
        K::normalize(&self.range, value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        K::denormalize(&self.range, normalized)
    }

    fn value_to_text(&self, value: f32, writer: &mut dyn fmt::Write) -> fmt::Result {
        K::write_text(&self.range, self.unit, value, writer)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }

    fn tags(&self) -> Tags {
        self.tags
    }
}
