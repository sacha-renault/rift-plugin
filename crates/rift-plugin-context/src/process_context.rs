use std::sync::Arc;

use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent, TransportFlags};
use clack_plugin::host::HostAudioProcessorHandle;
use clack_plugin::prelude::OutputEvents;
use clack_plugin::process::Process;

use rift_plugin_gui::GuiTask;
use rift_plugin_params::ParamCollection;
use rift_plugin_params::params_wrapper::ParamsWrapper;
use rift_plugin_types::transport::{BlockIndex, BlockInfo};

use crate::PluginSharedState;
use rift_plugin_types::MidiMessage;

#[derive(bon::Builder)]
pub struct ProcessContext<'a, 'e> {
    host: &'a HostAudioProcessorHandle<'a>,
    states: Arc<PluginSharedState>,
    process: Process<'a>,
    samplerate: f64,
    block_index: BlockIndex,

    outputs_events: &'e mut OutputEvents<'e>,
    host_params: Arc<ParamsWrapper>,
    notify_gui: bool,
}

impl<'a, 'e> ProcessContext<'a, 'e> {
    /// Returns playback progress info (seconds/beats) if currently playing, otherwise None.
    pub fn block_info(&self) -> Option<BlockInfo> {
        if let Some(transport) = self.process.transport {
            let info = BlockInfo {
                seconds: transport.song_pos_seconds.to_float(),
                beats: transport.song_pos_beats.to_float(),
                samplerate: self.samplerate,
                tempo: transport.tempo,
                flags: TransportFlags::IS_PLAYING,
            };
            Some(info)
        } else {
            None
        }
    }

    pub fn block_index(&self) -> BlockIndex {
        self.block_index
    }

    /// Add a midi message as output event
    pub fn add_output_midi_event(&mut self, event: MidiMessage) {
        let _ = self.outputs_events.try_push::<MidiEvent>(event.into());
    }

    /// Asks the host to restart the plugin.
    ///
    /// CLAP's `request_restart` is thread-safe, so this does not need to be
    /// deferred to the main thread.
    pub fn request_restart(&mut self) {
        self.host.request_restart();
    }

    /// Reports a new processing latency, in samples.
    pub fn set_latency(&mut self, latency: u32) {
        self.states.set_latency(latency);

        // On process context, latency triggers a restart.
        self.host.request_restart();
    }

    pub fn set_parameter(&mut self, param_event: ParamValueEvent) {
        if let Some(id) = param_event.param_id() {
            let value = param_event.value() as f32;
            self.host_params.set_value(id, value);

            // Notify GUI & Host about the param change.
            if let Err(err) = self.outputs_events.try_push(param_event) {
                log::error!("Host failed to read event {err}");
            }

            if self.notify_gui && self.states.is_gui_opened() {
                _ = self.states.post_to_gui(GuiTask::ParamChanged { id, value });
            }
        }
    }
}
