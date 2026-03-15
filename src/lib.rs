mod context;
mod params;
mod processing;
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
    pub use rift_plugin_derive::{
        DeriveEnumValues, DeriveParams, HandleExtension, ParamViewBuilder,
    };
    pub use rift_plugin_shared::gui::{ClapGui, GuiFactory};
    pub use rift_plugin_shared::params::{ClapParam, ParamPtr, Params, TypedParam};
    pub use rift_plugin_shared::utils;

    pub use params::param_bool::BoolParam;
    pub use params::param_enum::{EnumParam, EnumValues};
    pub use params::param_float::FloatParam;
    pub use params::param_int::IntParam;

    pub use super::export_clap_plugin;

    pub use super::wrapper::factory::PluginWrapper;
    pub use wrapper::ClapPlugin;

    pub use type_wrapper::{AudioPort, MainAudioPort};

    pub use context::*;
    pub use processing::*;
}

#[doc(hidden)]
pub mod _sealed {
    //! todo!()
    //!
    //! I didn't find an other way yet to initialize params id, name and module
    //! in a nice way. I will come back later on this. This needs to be public
    //! otherwise it can't be implemented by client side but that should NEVER be used
    //! for ant Plugin that uses Rift. Meant only for internal stuff.
    #[doc(hidden)]
    pub use rift_plugin_shared::params::{__ParamInitializer, __ParamsInitializer};
}
