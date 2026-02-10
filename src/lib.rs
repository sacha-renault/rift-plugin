mod params;
mod wrapper;

pub mod prelude {
    use super::*;

    pub use clack_extensions;
    pub use clack_plugin;

    pub use params::param_float::FloatParam;
    pub use params::param_trait::Param;

    pub use wrapper::ClapPlugin;

    #[allow(unused)]
    macro_rules! export_clap_plugin {
        (PluginType) => {
            clack_plugin::export_clap_plugin!(
                clack_plugin::prelude::SinglePluginEntry<crate::wrapper::main::Wrapper<PluginType>>
            )
        };
    }
}

// TODO
// Handle wrapper somewhere
pub mod _internal {
    pub use super::wrapper::main::Wrapper;
}
