use std::{ffi::CStr, sync::Arc};

pub use clack_plugin::prelude::*;

use rift_plugin_core::gui::GuiFactory;
use rift_plugin_core::params::Params;

use crate::_sealed::__ParamsInitializer;
use crate::prelude::*;

pub mod factory;
pub mod main_thread;
pub mod processor;
pub mod shared;
pub mod shared_states;

pub trait ClapPlugin: Send + Sync + Sized + 'static {
    /// The parameters for the plugin.
    /// These are automatically synchronized between the GUI and Audio threads.
    type ParamType: Params + __ParamsInitializer + Default + Send + Sync + 'static;

    /// Shared state accessible by the GUI (=Main), and Audio threads.
    /// Use this for non-parameter state like preset data or analysis results.
    type SharedType: Send + Sync + Default + 'static;

    /// If `true`, the wrapper automatically updates `ParamType` and calls [`Self::param_changed`]
    /// for every parameter event before [`Self::process`] is called.
    ///
    /// If `false`, parameter events are ignored by the wrapper and must be handled manually
    /// via the [`ProcessContext`] events iterator.
    ///
    /// **Notes**:
    /// Param events from GUI cannot really be sample accurate and GUI change will trigger
    /// [`Self::param_changed`] calls even if this is false!
    const PARAM_EVENT_AUTO_HANDLING: bool;

    /// If `true`, the wrapper automatically calls [`Self::on_midi_message`] for every MIDI
    /// event before [`Self::process`] is called.
    ///
    /// If `false`, MIDI events must be handled manually (sample-accurately)
    /// via the [`ProcessContext`] events iterator.
    const MIDI_EVENT_AUTO_HANDLING: bool;

    /// Define the maximum number of task the plugin can hold at the same time, before dropping
    /// events. See [`MainThreadTask`] and [`AudioThreadTask`].
    ///
    /// Default to `2048`.
    const TASKS_CAPACITY: usize = 2048;

    /// Creates a new instance of the plugin.
    ///
    /// Use this to prepare internal DSP (filters, oscillators) for a specific sample rate.
    /// **Notes**:
    /// You may allocate memory during this call.
    fn create(
        params: Arc<Self::ParamType>,
        shared: Arc<Self::SharedType>,
        config: PluginAudioConfiguration,
        context: InitContext,
    ) -> Self;

    /// Processes one block of audio and events.
    ///
    /// This is the "Hot Path." **Strictly avoid any operations that can block**,
    /// such as memory allocation, file I/O, or acquiring non-recursive mutexes.
    ///
    /// To handle events sample-accurately, use `context.zipped_events()`.
    fn process(
        &mut self,
        buffers: Buffers,
        context: ProcessContext<Self>,
        input_events: &InputEvents,
    ) -> Result<ProcessStatus, PluginError>;

    /// Called when a MIDI message is received.
    ///
    /// This is only triggered if [`Self::MIDI_EVENT_AUTO_HANDLING`] is set to `true`.
    /// Messages are delivered once per block, before the call to [`Self::process`].
    fn on_midi_message(&mut self, midi: MidiMessage);

    /// Called when a parameter value is changed.
    ///
    /// If [`Self::PARAM_EVENT_AUTO_HANDLING`] is `true`, this is called automatically.
    /// If `false`, this is called only for GUI events.
    fn param_changed(&mut self, id: ClapId, source: EventSource);

    /// Creates the GUI factory for this plugin.
    ///
    /// Since the GUI runs on a separate thread (or even a separate process),
    /// communication with the processor must happen via `params` or `shared`.
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
    const MIDI_PORTS: &[MidiPort<'_>] = &[];
}
