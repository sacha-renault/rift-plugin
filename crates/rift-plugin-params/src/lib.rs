//! Plugin parameters.
//!
//! This crate owns the parameter model that used to live in `rift-plugin-core`:
//! the [`Param`] / [`Persistent`] traits, the concrete parameter types
//! ([`FloatParam`], [`IntParam`], [`BoolParam`], [`EnumParam`], [`StringParam`])
//! and the [`ParamCollection`] / [`UserParams`] collection traits.

mod atomic_floats;
mod id;
mod param_bool;
mod param_enum;
mod param_float;
mod param_int;
mod param_string;
mod ptr;
mod scale;
mod traits;

#[doc(hidden)]
pub mod params_wrapper;

#[cfg(feature = "fun-dsp")]
mod param_float_fun_dsp;

mod test_macros;

pub use atomic_floats::{AtomicF32, AtomicF64};
pub use clack_plugin::utils::ClapId;
pub use id::param_id;
pub use param_bool::BoolParam;
pub use param_enum::{EnumParam, EnumParamBuilder, EnumValues};
pub use param_float::FloatParam;
pub use param_int::IntParam;
pub use param_string::{StringParam, StringParamValue};
pub use ptr::ParamPtr;
pub use scale::Scale;
pub use traits::{Param, ParamCollection, Persistent, TypedParam, UserParams};

#[cfg(feature = "fun-dsp")]
pub use param_float_fun_dsp::SharedFloatParam;
