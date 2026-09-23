//! Extra helpers on top of [`fundsp`].
//!
//! Currently re-exports [`fun_dsp_ext`], which adds the [`AtomicTableExt`]
//! trait for building band-limited [`fundsp`] wavetables.

pub mod fun_dsp_ext;
pub mod process_rift_buffer;

pub use fun_dsp_ext::*;
pub use process_rift_buffer::*;
