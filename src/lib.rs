//! Rift: a thin facade over the `rift-plugin-*` crates.
//!
//! Everything of substance lives in a dedicated crate; this crate only
//! re-exports them through [`prelude`] and exposes the [`export_clap_plugin!`]
//! helper.

#[macro_export]
macro_rules! export_clap_plugin {
    ($PluginType:ty) => {
        use clack_plugin::clack_export_entry;
        use rift_plugin::prelude::PluginWrapper;

        clack_export_entry! {
            clack_plugin::prelude::SinglePluginEntry<PluginWrapper<$PluginType>>
        }
    };
}

pub mod prelude {
    // Reexport
    pub use clack_extensions;
    pub use clack_plugin;

    pub use clack_extensions::audio_ports::{AudioPortFlags, AudioPortType};
    pub use clack_plugin::prelude::PluginError;

    // Reexport the rift-plugin crates
    pub use rift_plugin_buffers::*;
    pub use rift_plugin_context::*;
    pub use rift_plugin_core::gui::{ClapGui, GuiFactory};
    pub use rift_plugin_core::prelude::*;
    pub use rift_plugin_core::utils;
    pub use rift_plugin_params::*;
    pub use rift_plugin_types::*;
    pub use rift_plugin_wrapper::*;

    pub use rift_plugin_derive::params;
    pub use rift_plugin_derive::{DeriveEnumValues, DeriveParams};
    pub use rift_plugin_derive::{HandleExtension, ParamViewBuilder};

    pub use super::export_clap_plugin;
}

#[doc(hidden)]
pub mod _sealed {
    // reexport of serde_json needed for derive
    #[doc(hidden)]
    pub use serde_json;
}
