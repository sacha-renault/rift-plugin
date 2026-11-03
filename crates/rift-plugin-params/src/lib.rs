//! Plugin parameters.
//!
//! This crate owns the parameter model that used to live in `rift-plugin-core`:
//! the [`Param`] / [`Persistent`] traits, the concrete parameter types
//! ([`FloatParam`], [`IntParam`], [`BoolParam`], [`EnumParam`], [`StringParam`])
//! and the [`ParamCollection`] / [`UserParams`] collection traits.

mod id;
mod param_string;
mod param_swap;
mod ptr;
mod range;
mod scale;
mod tags;
mod traits;

mod atomic_param;
mod test_macros;

#[doc(hidden)]
pub mod params_wrapper;

pub use atomic_param::{BoolParam, EnumParam, EnumValues, FloatParam, IntParam};
pub use clack_plugin::utils::ClapId;
pub use id::param_id;
pub use param_string::{StringParam, StringParamValue};
pub use param_swap::{PersistValue, SwapParam, SwapParamValue};
pub use ptr::ParamPtr;
pub use range::{FloatRange, IntRange};
pub use scale::Scale;
pub use tags::Tags;
pub use traits::{Param, ParamCollection, PersistentParam, TypedParam, UserParams};

#[cfg(feature = "fun-dsp")]
pub use atomic_param::SharedFloatParam;
