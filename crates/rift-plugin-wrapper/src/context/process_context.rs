use std::sync::Arc;

use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent};
use clack_plugin::host::HostAudioProcessorHandle;
use clack_plugin::prelude::OutputEvents;

use rift_plugin_buffers::InputEvent;
use rift_plugin_gui::HostParamChange;
use rift_plugin_params::ParamCollection;
use rift_plugin_params::params_wrapper::ParamsWrapper;
use rift_plugin_types::{EventSource, MidiMessage, TransportInfo};

use crate::ClapPlugin;
use crate::shared::PluginSharedState;

pub struct ProcessContext<'a, 'e, P: ClapPlugin> {
    pub(crate) host: &'a HostAudioProcessorHandle<'a>,
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) transport_info: &'a TransportInfo,

    pub(crate) output_events: &'e mut OutputEvents<'e>,
    pub(crate) user_params: Arc<P::Params>,
    pub(crate) shared: Arc<P::SharedData>,
    pub(crate) host_params: Arc<ParamsWrapper>,
    pub(crate) notify_gui: bool,
}

impl<'a, 'e, P: ClapPlugin> ProcessContext<'a, 'e, P> {
    pub fn transport_info(&self) -> &TransportInfo {
        self.transport_info
    }

    /// Add a midi message as output event
    pub fn add_output_midi_event(&mut self, event: MidiMessage) {
        let _ = self.output_events.try_push::<MidiEvent>(event.into());
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
    #[inline]
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

    pub fn handle_events<I>(&mut self, plugin: &mut P, events: I)
    where
        I: Iterator<Item = InputEvent>,
    {
        for event in events {
            match event {
                InputEvent::MidiEvent(midi) => {
                    plugin.on_midi_message(midi, self.user_params.as_ref(), self.shared.as_ref())
                }
                InputEvent::ParamEvent(param) => {
                    let Some(id) = param.param_id() else {
                        continue;
                    };

                    plugin.on_param_change(
                        id,
                        self.user_params.as_ref(),
                        self.shared.as_ref(),
                        EventSource::Host,
                    );
                }
            }
        }
    }
}
