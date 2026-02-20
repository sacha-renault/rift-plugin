use std::{ffi::CStr, sync::Arc};

pub use clack_plugin::prelude::*;

use crate::context::{InitContext, ProcessContext};
use crate::gui::IntoGui;
use crate::params::param_trait::Params;
use crate::prelude::{Buffers, MainAudioPort};
use crate::type_wrapper::AudioPort;

pub mod main;
pub mod main_thread;
pub mod processor;
pub mod shared;
pub mod shared_states;

pub trait ClapPlugin: Send + Sync + 'static {
    /// Params for the plugin
    type ParamType: Params + Default + Send + Sync + 'static;

    /// Anything else that should be shared, must just be thread safe
    type SharedType: Send + Sync + Default + 'static;

    // LATER, define the gui here ...
    // type GuiType: Gui + Send + Sync + Default + 'static;

    fn create(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Self;

    fn process<'a>(
        &mut self,
        buffers: Buffers<'a>,
        context: ProcessContext<'a>,
    ) -> Result<ProcessStatus, PluginError>;

    fn activate(&mut self, config: PluginAudioConfiguration, context: InitContext);

    fn gui(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Box<dyn IntoGui>;

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

    const MAIN_AUDIO_PORTS: MainAudioPort;
    const AUX_AUDIO_PORTS: &[AudioPort<'_>] = &[];

    // DEBUG RN
    const INIT_LOG_FN: Option<fn() -> Result<(), Box<dyn std::error::Error>>> = None;
}
