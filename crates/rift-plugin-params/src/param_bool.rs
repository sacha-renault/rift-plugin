use std::sync::atomic::{AtomicBool, Ordering};

use crate::atomic_param::{Atomic, AtomicParam, ParamKind};
use crate::scale::Linear;
use crate::{Param, TypedParam};

pub struct _Bool;

impl Atomic<bool> for AtomicBool {
    fn new(v: bool) -> Self {
        Self::new(v)
    }

    fn get(&self) -> bool {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, v: bool) {
        self.store(v, Ordering::Relaxed);
    }
}

impl ParamKind for _Bool {
    type Scale = Linear;
    type Storage = AtomicBool;
    type Value = bool;

    fn to_f32(v: Self::Value) -> f32 {
        if v { 1.0 } else { 0.0 }
    }

    fn from_f32(v: f32) -> Self::Value {
        v >= 0.5
    }

    fn default_min() -> Self::Value {
        false
    }

    fn default_max() -> Self::Value {
        true
    }
}

impl TypedParam for BoolParam {
    type Type = bool;

    fn value(&self) -> Self::Type {
        _Bool::from_f32(self.plain())
    }

    fn set_value(&self, value: Self::Type) {
        self.set_plain(_Bool::to_f32(value))
    }
}

pub type BoolParam = AtomicParam<_Bool>;

#[cfg(test)]
mod tests {
    use clack_extensions::params::ParamInfoFlags;
    use clack_plugin::utils::ClapId;

    use super::*;

    #[test]
    fn builder_create_correct_param() {
        let param = BoolParam::builder()
            .unit("on/off")
            .default(true)
            .flags(ParamInfoFlags::IS_AUTOMATABLE)
            .build();

        assert_eq!(param.unit(), "on/off");
        assert!(param.value());
        assert_eq!(param.default_plain(), 1.0);
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

        assert!(!param.value());
        param.set_value(true);
        assert!(param.value());
    }

    #[test]
    fn set_plain_threshold() {
        let param = BoolParam::builder().default(false).build();

        param.set_plain(0.49);
        assert!(!param.value());

        param.set_plain(0.5);
        assert!(param.value());

        param.set_plain(1.0);
        assert!(param.value());

        param.set_plain(0.0);
        assert!(!param.value());
    }

    #[test]
    fn plain_returns_0_or_1() {
        let param = BoolParam::builder().default(false).build();
        assert_eq!(param.plain(), 0.0);

        param.set_value(true);
        assert_eq!(param.plain(), 1.0);
    }

    #[test]
    fn ptr_change_param() {
        let param = BoolParam::builder().default(false).build();
        let ptr = param.as_ptr();

        ptr.set_normalized(1.0);
        assert!(param.value());

        ptr.set_normalized(0.0);
        assert!(!param.value());
    }
}
