use std::sync::atomic::{AtomicBool, Ordering};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;

use crate::params::param_ptr::ParamPtr;
use crate::prelude::ClapParam;
use crate::utils::id_generator::get_next_param_id;

use super::param_trait::TypedParam;

#[derive(bon::Builder)]
pub struct BoolParam {
    /// Default value for the param
    #[allow(unused)]
    default: bool,

    #[builder(skip = AtomicBool::new(default))]
    value: AtomicBool,

    /// Name of the param
    name: &'static str,

    #[builder(default = "")]
    unit: &'static str,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    flags: ParamInfoFlags,

    #[builder(skip = get_next_param_id())]
    id: ClapId,
}

impl TypedParam for BoolParam {
    type Value = bool;

    fn value(&self) -> Self::Value {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: Self::Value) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl ClapParam for BoolParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit(&self) -> &str {
        self.unit
    }

    fn set_raw(&self, value: f64) {
        self.value.store(value >= 0.5, Ordering::SeqCst);
    }

    fn get_raw(&self) -> f64 {
        if self.value.load(Ordering::SeqCst) {
            1.0
        } else {
            0.0
        }
    }

    fn default_raw(&self) -> f64 {
        if self.default { 1.0 } else { 0.0 }
    }

    fn get_normalized(&self) -> f64 {
        self.get_raw()
    }

    fn set_normalized(&self, normalized: f64) {
        self.set_raw(normalized);
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f64 {
        0.0
    }

    fn max_value(&self) -> f64 {
        1.0
    }

    #[inline]
    fn normalize(&self, value: f64) -> f64 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        value
    }

    #[inline]
    fn denormalize(&self, normalized: f64) -> f64 {
        // bool param already have
        // normalized value (0.0 or 1.0)
        normalized
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr {
            ptr: self as *const dyn ClapParam,
        }
    }
}
