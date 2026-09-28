//! The CLAP wrapper: the [`ClapPlugin`] trait implemented by plugins, plus the
//! glue that turns it into a CLAP plugin.

use std::{ffi::CStr, sync::Arc};

pub use clack_plugin::prelude::*;

use rift_plugin_buffers::Buffers;
use rift_plugin_context::{InitContext, ProcessContext};
use rift_plugin_gui::GuiFactory;
use rift_plugin_params::UserParams;
use rift_plugin_types::{AudioPort, EventSource, MainAudioPort, MidiMessage, MidiPort};

pub mod factory;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub use factory::PluginWrapper;

pub trait ClapPlugin: Send + Sync + Sized + 'static {
    /// The parameters for the plugin.
    /// These are automatically synchronized between the GUI and Audio threads.
    ///
    /// Implemented with `#[derive(Params)]`; the wrapper builds a host-facing
    /// [`rift_plugin_params::ParamCollection`] view over it automatically.
    type Params: UserParams + Default + Send + Sync + 'static;

    /// Shared data (non param) between audio thread and gui thread.
    type SharedData: Default + Send + Sync + 'static;

    /// If `true`, the wrapper automatically writes incoming host parameter values into
    /// the parameter store during the pre-[`Self::process`] flush, so the parameters
    /// already hold the new value when [`Self::process`] runs.
    ///
    /// If `false`, the wrapper leaves the values untouched: the plugin is responsible for
    /// applying them itself, typically sample-accurately via the events yielded by
    /// `zip_events`.
    ///
    /// This flag only controls whether the *value* is applied automatically.
    /// [`Self::param_changed`] is notified for host parameter events regardless of this
    /// setting (see its documentation).
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
        params: &Self::Params,
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
        ctx: ProcessContext,
        events: &InputEvents,
        params: &Self::Params,
        data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError>;

    /// Called when a MIDI message is received.
    ///
    /// This is only triggered if [`Self::MIDI_EVENT_AUTO_HANDLING`] is set to `true`.
    /// Messages are delivered once per block, before the call to [`Self::process`].
    fn on_midi_message(&mut self, midi: MidiMessage);

    /// Called when a parameter value changes.
    ///
    /// - Changes coming from the GUI always notify, with [`EventSource::GUI`]. GUI edits
    ///   are not sample-accurate, so this fires independently of
    ///   [`Self::PARAM_EVENT_AUTO_HANDLING`].
    /// - Changes coming from the host/automation notify once per block, before
    ///   [`Self::process`], with [`EventSource::Host`] - even when
    ///   [`Self::PARAM_EVENT_AUTO_HANDLING`] is `false` (in which case the wrapper does
    ///   not apply the value itself, it only tells you it changed).
    ///
    /// Host notifications are skipped only when both [`Self::PARAM_EVENT_AUTO_HANDLING`]
    /// and [`Self::MIDI_EVENT_AUTO_HANDLING`] are `false`, since the wrapper then leaves
    /// all incoming events to the plugin.
    #[allow(unused)]
    fn param_changed(&mut self, id: ClapId, params: &Self::Params, source: EventSource) {}

    /// Creates the GUI factory for this plugin.
    ///
    /// Since the GUI runs on a separate thread (or even a separate process),
    /// communication with the processor must happen via `params` or `shared`.
    fn gui(params: Arc<Self::Params>, data: Arc<Self::SharedData>) -> Box<dyn GuiFactory>;

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

// Opt-in, debug-only global allocator used by `assert_no_alloc` to catch
// allocations on the realtime thread. Only installed when the feature is on.
#[cfg(all(feature = "assert-no-alloc", debug_assertions))]
#[global_allocator]
static A: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;
