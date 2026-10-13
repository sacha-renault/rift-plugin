//! Extra helpers on top of [`fundsp`].

macro_rules! mod_exp {
    ($mod_name:ident) => {
        pub mod $mod_name;
        pub use $mod_name::*;
    };
}

mod_exp!(atomic_table_ext);
mod_exp!(audio_unit_ext);
mod_exp!(poly_voice);
mod_exp!(wavetables);
