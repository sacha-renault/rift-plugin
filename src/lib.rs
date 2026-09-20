mod buffers;
mod context;
mod ty;
mod type_wrapper;
mod wrapper;

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
    use super::*;

    // Reexport
    pub use clack_extensions;
    pub use clack_plugin;

    pub use clack_extensions::audio_ports::{AudioPortFlags, AudioPortType};
    pub use clack_plugin::prelude::PluginError;

    // reexport inner
    pub use rift_plugin_core::gui::{ClapGui, GuiFactory};
    pub use rift_plugin_core::prelude::*;
    pub use rift_plugin_core::utils;
    pub use rift_plugin_derive::params;
    pub use rift_plugin_derive::{DeriveEnumValues, DeriveParams};
    pub use rift_plugin_derive::{HandleExtension, ParamViewBuilder};
    pub use rift_plugin_params::*;

    pub use super::export_clap_plugin;

    pub use super::wrapper::factory::PluginWrapper;
    pub use wrapper::ClapPlugin;

    pub use type_wrapper::*;

    pub use buffers::*;
    pub use context::*;

    pub use ty::*;
}

#[doc(hidden)]
pub mod _sealed {
    // reexport of serde_json needed for derive
    #[doc(hidden)]
    pub use serde_json;
}
