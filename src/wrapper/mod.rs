use std::{ffi::CStr, sync::Arc};

pub use clack_plugin::prelude::*;

use crate::{
    params::param_trait::Params,
    type_wrapper::{AudioPort, ClapGui},
};

pub mod main;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub trait ClapPlugin: Send + Sync + 'static {
    /// Params for the plugin
    type ParamType: Params + Send + Sync + Default + 'static;

    /// Anything else that should be shared, must just be thread safe
    type SharedType: Send + Sync + Default + 'static;

    /// The GUI type
    type GuiType: ClapGui + Send + Sync + Default + 'static;
    const HAS_GUI: bool;

    // LATER, define the gui here ...
    // type GuiType: Gui + Send + Sync + Default + 'static;

    fn create(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Self;
    fn process(&mut self, audio: &mut [&mut [f32]]) -> Result<ProcessStatus, PluginError>;
    fn activate(&mut self, audio_config: PluginAudioConfiguration);

    // ... Later more methods :)
    const ID: &str;
    const NAME: &str;
    const FEATURES: &[&CStr];
    const VERSION: &str;
    const DESCRIPTION: &str = "";
    const URL: &str = "";
    const VENDOR: &str = "";
    const SUPPORT_URL: &str = "";
    const MANUAL_URL: &str = "";

    const AUDIO_PORTS: &[AudioPort<'_>] = &[];

    // DEBUG RN
    const INIT_LOG_FN: Option<fn() -> Result<(), Box<dyn std::error::Error>>> = None;
}
