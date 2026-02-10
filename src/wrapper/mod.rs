pub use clack_plugin::prelude::*;

use crate::params::param_trait::Params;

pub mod main;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub trait ClapPlugin: Send + Sync + Default + 'static {
    type ParamType: Params + Send + Sync + Default + 'static;

    fn info() -> PluginDescriptor;
    fn process_audio(&mut self, audio: &mut [&mut [f32]], events: &[&UnknownEvent]);
    fn activate(&mut self, audio_config: PluginAudioConfiguration);
}
