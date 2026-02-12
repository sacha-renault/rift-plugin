use std::sync::Arc;

pub use clack_plugin::prelude::*;

use crate::params::param_trait::Params;

pub mod main;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub trait ClapPlugin: Send + Sync + 'static {
    /// Params for the plugin
    type ParamType: Params + Send + Sync + Default + 'static;

    /// Anything else that should be shared, must just be thread safe
    type SharedType: Send + Sync + Default + 'static;

    // LATER, define the gui here ...
    // type GuiType: Gui + Send + Sync + Default + 'static;

    fn create(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Self;
    fn info() -> PluginDescriptor;
    fn process(&mut self, audio: &mut [&mut [f32]]) -> Result<ProcessStatus, PluginError>;
    fn activate(&mut self, audio_config: PluginAudioConfiguration);

    // ... Later more methods :)
}
