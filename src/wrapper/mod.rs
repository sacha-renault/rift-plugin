use std::sync::Arc;

pub use clack_plugin::prelude::*;

use crate::params::param_trait::Params;

pub mod main;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub trait ClapPlugin: Send + Sync + 'static {
    type ParamType: Params + Send + Sync + Default + 'static;
    type Shared: Send + Sync + Default + 'static;

    fn create(params: Arc<Self::ParamType>, shared: Arc<Self::Shared>) -> Self;
    fn info() -> PluginDescriptor;
    fn process(&mut self, audio: &mut [&mut [f32]]) -> Result<ProcessStatus, PluginError>;
    fn activate(&mut self, audio_config: PluginAudioConfiguration);
}
