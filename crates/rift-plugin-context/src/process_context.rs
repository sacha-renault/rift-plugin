use std::sync::Arc;

use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent};
use clack_plugin::host::HostAudioProcessorHandle;
use clack_plugin::prelude::OutputEvents;

use rift_plugin_gui::HostParamChange;
use rift_plugin_params::ParamCollection;
use rift_plugin_params::params_wrapper::ParamsWrapper;

use crate::PluginSharedState;
use rift_plugin_types::{MidiMessage, TransportInfo};

#[derive(bon::Builder)]
pub struct ProcessContext<'a, 'e> {
    host: &'a HostAudioProcessorHandle<'a>,
    states: Arc<PluginSharedState>,
    transport_info: &'a TransportInfo,

    outputs_events: &'e mut OutputEvents<'e>,
    host_params: Arc<ParamsWrapper>,
    notify_gui: bool,
}

impl<'a, 'e> ProcessContext<'a, 'e> {
    pub fn transport_info(&self) -> &TransportInfo {
        self.transport_info
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
    /// Takes effect at next activation
    pub fn set_latency(&mut self, latency: u32) {
        self.states.set_latency(latency);

        // On process context, latency triggers a restart.
        self.request_restart();
    }

    /// Set a parameter from a host change
    pub fn set_parameter_from_host_event(&mut self, param_event: ParamValueEvent) {
        if let Some(id) = param_event.param_id() {
            let value = param_event.value() as f32;
            self.host_params.set_value(id, value);

            // Notify GUI about the param change.
            // this function is called from ParamValueEvent from the
            // host, it would be an error to notify host too.
            if self.notify_gui && self.states.is_gui_opened() {
                self.states
                    .gui_param_changes()
                    .post(HostParamChange { id, value });
            }
        }
    }
}
