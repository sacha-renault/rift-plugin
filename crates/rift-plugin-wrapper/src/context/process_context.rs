use std::sync::Arc;

use clack_plugin::events::event_types::MidiEvent;
use clack_plugin::events::io::InputEvents;
use clack_plugin::host::HostAudioProcessorHandle;
use clack_plugin::prelude::OutputEvents;

use rift_plugin_buffers::{EventsCursor, InputEvent};
use rift_plugin_params::params_wrapper::ParamsWrapper;
use rift_plugin_types::{EventPreProcess, EventSource, MidiMessage, TransportInfo};

use crate::ClapPlugin;
use crate::shared::{PluginSharedState, apply_host_param};

pub struct ProcessContext<'a, 'e, P: ClapPlugin> {
    pub(crate) host: &'a HostAudioProcessorHandle<'a>,
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) transport_info: &'a TransportInfo,

    pub(crate) cursor_events: EventsCursor<'a>,
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

    pub fn input_events(&self) -> &InputEvents<'_> {
        self.cursor_events.events()
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
    pub fn set_parameter_from_host_event(&mut self, id: ClapId, value: f32) {
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

    pub fn handle_events(&mut self, plugin: &mut P, time: u32) {
        let flags = P::EVENT_PRE_PROCESS;

        // Noop in case all flags are set
        if flags.is_all() {
            return;
        }

        // Flags contain the expected behavior for auto process BEFORE
        // the audio block. As now we are DURING the audio block. We
        // wanna do what wasn't done before the block.
        let call_midi_change = !flags.contains(EventPreProcess::MIDI_NOTIFY_EVENT);
        let call_parm_change = !flags.contains(EventPreProcess::PARAM_NOTIFY_CHANGE);
        let set_param_change = !flags.contains(EventPreProcess::PARAM_APPLY_CHANGE);
        let any_param_event = call_parm_change || set_param_change;

        for event in self.cursor_events.iter_events_at(time) {
            match event {
                InputEvent::MidiEvent(midi) if call_midi_change => {
                    plugin.on_midi_message(midi, self.user_params.as_ref(), self.shared.as_ref())
                }
                InputEvent::ParamEvent(param) if any_param_event => {
                    let Some(id) = param.param_id() else {
                        continue;
                    };

                    apply_host_param(
                        &self.host_params,
                        &self.states,
                        set_param_change,
                        set_param_change && self.notify_gui,
                        id,
                        param.value() as f32,
                    );

                    if call_parm_change {
                        plugin.on_param_change(
                            id,
                            self.user_params.as_ref(),
                            self.shared.as_ref(),
                            EventSource::Host,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
