//! Extra helpers on top of [`fundsp`].
//!
//! Currently re-exports [`fun_dsp_ext`], which adds the [`AtomicTableExt`]
//! trait for building band-limited [`fundsp`] wavetables.

pub mod atomic_table_ext;
pub mod audio_unit_ext;

pub use atomic_table_ext::*;
pub use audio_unit_ext::*;
