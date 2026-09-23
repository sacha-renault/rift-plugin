//! Extra helpers on top of [`fundsp`].
//!
//! Currently re-exports [`fun_dsp_ext`], which adds the [`AtomicTableExt`]
//! trait for building band-limited [`fundsp`] wavetables.

pub mod fun_dsp_ext;

pub use fun_dsp_ext::*;
