//! Extra helpers on top of [`fundsp`].
//!
//! Currently re-exports [`fun_dsp_ext`], which adds the [`AtomicTableExt`]
//! trait for building band-limited [`fundsp`] wavetables.

macro_rules! mod_exp {
    ($mod_name:ident) => {
        pub mod $mod_name;
        pub use $mod_name::*;
    };
}

mod_exp!(atomic_table_ext);
mod_exp!(audio_unit_ext);
