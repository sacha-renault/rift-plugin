mod gui;
mod params;
mod type_wrapper;
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

    pub use clack_extensions;
    pub use clack_plugin;

    pub use clack_extensions::audio_ports::{AudioPortFlags, AudioPortType};
    pub use clack_plugin::prelude::PluginError;

    pub use params::param_float::FloatParam;
    pub use params::param_trait::{Param, Params};

    pub use super::wrapper::main::PluginWrapper;
    pub use wrapper::ClapPlugin;

    pub use type_wrapper::AudioPort;

    pub use gui::{ClapGui, ViziaGui};
    pub use vizia;
}
