mod context;
mod gui;
mod params;
mod processing;
mod type_wrapper;
mod utils;
mod wrapper;

#[macro_export]
macro_rules! export_clap_plugin {
    ($PluginType:ty) => {
        use clack_hug::prelude::PluginWrapper;
        use clack_plugin::clack_export_entry;

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
    pub use vizia;

    pub use clack_extensions::audio_ports::{AudioPortFlags, AudioPortType};
    pub use clack_plugin::prelude::PluginError;

    pub use params::param_float::FloatParam;
    pub use params::param_trait::{ClapParam, Params, TypedParam};

    pub use super::wrapper::main::PluginWrapper;
    pub use wrapper::ClapPlugin;

    pub use type_wrapper::{AudioPort, MainAudioPort};

    pub use gui::{ClapGui, GuiFactory, GuiParamEvent, ViziaGui};

    pub use processing::*;

    pub use context::*;
}
