use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::scale::Scaling;

use super::ptr::ParamPtr;
use super::traits::Param;

/// Lock-free cell holding a `T`.
pub trait Atomic<T>: Send + Sync {
    fn new(v: T) -> Self;
    fn get(&self) -> T;
    fn set(&self, v: T);
}

pub trait ParamKind: 'static {
    type Value: Copy + Default + std::fmt::Display + Send + Sync + 'static;
    type Storage: Atomic<Self::Value>;
    type Scale: Scaling + Default;

    fn to_f32(v: Self::Value) -> f32;
    fn from_f32(v: f32) -> Self::Value;
    fn default_min() -> Self::Value;
    fn default_max() -> Self::Value;
}

#[derive(bon::Builder)]
pub struct AtomicParam<K: ParamKind> {
    #[builder(default)]
    pub(crate) default: K::Value,

    #[builder(skip = <K::Storage as Atomic<K::Value>>::new(default))]
    pub(crate) storage: K::Storage,

    #[builder(default)]
    name: String,
    module: Option<String>,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = K::to_f32(K::default_min()), with = |v: K::Value| K::to_f32(v))]
    pub(crate) min_value: f32,
    #[builder(default = K::to_f32(K::default_max()), with = |v: K::Value| K::to_f32(v))]
    pub(crate) max_value: f32,

    #[builder(default)]
    pub(crate) scale: K::Scale,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl<K: ParamKind> crate::traits::__private::Sealed for AtomicParam<K> {}

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
        let v = K::from_f32(value.clamp(self.min_value, self.max_value));
        self.storage.set(v);
    }

    fn plain(&self) -> f32 {
        K::to_f32(self.storage.get())
    }

    fn default_plain(&self) -> f32 {
        K::to_f32(self.default)
    }

    fn normalized(&self) -> f32 {
        let value = self.plain();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(self.denormalize(normalized));
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
        self.scale.normalize(value, self.min_value, self.max_value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.scale
            .denormalize(normalized, self.min_value, self.max_value)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }
}
