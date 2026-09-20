//! Plugin parameters.
//!
//! This crate owns the parameter model that used to live in `rift-plugin-core`:
//! the [`Param`] / [`Persistent`] traits, the concrete parameter types
//! ([`FloatParam`], [`IntParam`], [`BoolParam`], [`EnumParam`]) and the
//! [`Params`] collection trait.

mod atomic_f32;
mod param_bool;
mod param_enum;
mod param_float;
mod param_int;
mod ptr;
mod traits;

mod test_macros;

pub use atomic_f32::AtomicF32;
pub use param_bool::{BoolParam, BoolParamConfig};
pub use param_enum::{EnumParam, EnumParamConfig, EnumValues};
pub use param_float::{FloatParam, FloatParamConfig, RangeMapping};
pub use param_int::{IntParam, IntParamConfig};
pub use ptr::ParamPtr;
pub use traits::{Param, Params, Persistent, TypedParam};
