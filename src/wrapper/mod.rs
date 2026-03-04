use std::{ffi::CStr, sync::Arc};

pub use clack_plugin::prelude::*;

use crate::context::{InitContext, ProcessContext};
use crate::gui::GuiFactory;
use crate::params::param_trait::Params;
use crate::prelude::{Buffers, MainAudioPort};
use crate::type_wrapper::AudioPort;

pub mod factory;
pub mod main_thread;
pub mod processor;
pub mod shared;
pub mod shared_states;

pub trait ClapPlugin: Send + Sync + Sized + 'static {
    /// Params for the plugin
    type ParamType: Params + Default + Send + Sync + 'static;

    /// Anything else that should be shared, must just be thread safe
    type SharedType: Send + Sync + Default + 'static;

    fn create(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Self;

    /// Processes audio data for one block of time.
    ///
    /// The host owns the lifetime of `buffers` and `context`; they are invalid
    /// once this function returns.
    ///
    /// #Note
    /// You MUST NEVER allocate during the process funciton, as it might block and cracks the audio thread.
    /// Use scratch buffer you initialized during [`Self::activate`]
    fn process(
        &mut self,
        buffers: Buffers,
        context: ProcessContext<Self>,
    ) -> Result<ProcessStatus, PluginError>;

    /// Called by the host once initialization is complete.
    ///
    /// # Responsibility
    /// Sets up internal audio graphs, buffer sizes, and prepares for processing.
    /// You may allocate during this function
    fn activate(&mut self, config: PluginAudioConfiguration, context: InitContext);

    /// Called by the host to create a GUI factory for this plugin.
    ///
    /// # Note
    /// The returned `Box<dyn GuiFactory>` is owned by the host.
    /// Do not keep references to internal state created here across GUI redraws;
    /// rely on [`Self::SharedType`] or external state management instead.
    fn gui(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Box<dyn GuiFactory>;

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
