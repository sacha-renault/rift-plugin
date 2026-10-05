use std::marker::PhantomData;

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use super::param_int::IntParam;
use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

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

    /// Entry point for the derive macro; mirrors the `bon` builders of the
    /// other parameter types.
    pub fn builder() -> EnumParamBuilder<E> {
        EnumParamBuilder {
            id: ClapId::new(0),
            name: String::new(),
            module: None,
            default: E::default(),
            unit: "",
            // Enum flags default to empty; `with_flags` unions on top.
            flags: ParamInfoFlags::empty(),
        }
    }
}

/// Builder for [`EnumParam`], mirroring the `bon`-generated builders of the
/// other parameter types (same setters, same `build`).
pub struct EnumParamBuilder<E: EnumValues> {
    id: ClapId,
    name: String,
    module: Option<String>,
    default: E,
    unit: &'static str,
    flags: ParamInfoFlags,
}

impl<E: EnumValues> EnumParamBuilder<E> {
    pub fn id(mut self, id: ClapId) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn maybe_module(mut self, module: Option<String>) -> Self {
        self.module = module;
        self
    }

    pub fn default(mut self, default: E) -> Self {
        self.default = default;
        self
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    pub fn flags(mut self, flags: ParamInfoFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> EnumParam<E> {
        let total = E::count() as i32;
        let inner = IntParam::builder()
            .id(self.id)
            .name(self.name)
            .maybe_module(self.module)
            .default(self.default.to_index() as i32)
            .min_value(0)
            .max_value(total - 1)
            .unit(self.unit)
            .flags(self.flags)
            .build();

        EnumParam {
            inner,
            _p: PhantomData,
        }
    }
}

impl<E: EnumValues> TypedParam for EnumParam<E> {
    type Type = E;

    fn set_value(&self, value: Self::Type) {
        self.set_plain(value.to_index() as f32);
    }

    fn value(&self) -> Self::Type {
        let enum_idx = self.plain().round() as u32;
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

impl<E: EnumValues> crate::traits::__private::Sealed for EnumParam<E> {}

impl<E: EnumValues> Param for EnumParam<E> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn module(&self) -> Option<&str> {
        self.inner.module()
    }

    fn id(&self) -> ClapId {
        self.inner.id()
    }

    fn unit(&self) -> &str {
        self.inner.unit()
    }

    fn plain(&self) -> f32 {
        self.inner.plain()
    }

    fn normalized(&self) -> f32 {
        self.inner.normalized()
    }

    fn default_plain(&self) -> f32 {
        self.inner.default_plain()
    }

    fn flags(&self) -> ParamInfoFlags {
        self.inner.flags()
    }

    fn normalize(&self, value: f32) -> f32 {
        self.inner.normalize(value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.inner.denormalize(normalized)
    }

    fn min_value(&self) -> f32 {
        self.inner.min_value as f32
    }

    fn max_value(&self) -> f32 {
        self.inner.max_value as f32
    }

    fn set_plain(&self, value: f32) {
        self.inner.set_plain(value);
    }

    fn set_normalized(&self, normalized: f32) {
        self.inner.set_normalized(normalized);
    }

    fn as_ptr(&self) -> ParamPtr {
        self.inner.as_ptr()
    }

    fn value_to_text(&self, value: f32, writer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let variant = E::from_index(value.round() as u32).unwrap_or_default();
        writer.write_str(&format!("{variant}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    enum TestEnum {
        #[default]
        A,
        B,
        C,
    }

    impl std::fmt::Display for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::A => write!(f, "A"),
                Self::B => write!(f, "B"),
                Self::C => write!(f, "C"),
            }
        }
    }

    impl EnumValues for TestEnum {
        fn to_index(self) -> u32 {
            self as u32
        }

        fn from_index(index: u32) -> Option<Self> {
            match index {
                0 => Some(Self::A),
                1 => Some(Self::B),
                2 => Some(Self::C),
                _ => None,
            }
        }

        fn count() -> u32 {
            3
        }
    }

    #[test]
    fn default_value() {
        let param = EnumParam::new(TestEnum::B);
        assert_eq!(param.value(), TestEnum::B);
        assert_eq!(param.default_plain(), 1.0);
    }

    #[test]
    fn set_value_typed() {
        let param = EnumParam::new(TestEnum::A);
        param.set_value(TestEnum::C);
        assert_eq!(param.value(), TestEnum::C);
    }

    #[test]
    fn set_plain_rounds_to_variant() {
        let param = EnumParam::new(TestEnum::A);
        param.set_plain(1.6);
        assert_eq!(param.value(), TestEnum::B);
    }

    #[test]
    fn min_max_values() {
        let param = EnumParam::new(TestEnum::A);
        assert_eq!(param.min_value(), 0.0);
        assert_eq!(param.max_value(), 2.0);
    }

    #[test]
    fn with_flags() {
        let param = EnumParam::new(TestEnum::A).with_flags(ParamInfoFlags::IS_AUTOMATABLE);

        assert!(param.flags().contains(ParamInfoFlags::IS_AUTOMATABLE));
    }

    #[test]
    fn value_to_text() {
        let param = EnumParam::new(TestEnum::A);
        let mut buf = String::new();
        param.value_to_text(2.0, &mut buf).unwrap();
        assert_eq!(buf, "C");
    }

    #[test]
    fn ptr_change_param() {
        let param = EnumParam::new(TestEnum::A);
        let ptr = param.as_ptr();
        ptr.set_normalized(1.0);
        assert_eq!(param.value(), TestEnum::C);
    }
}
