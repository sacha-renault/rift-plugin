use clack_extensions::params::*;
use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent};
use clack_plugin::prelude::*;

use rift_plugin_context::AudioThreadTask;
use rift_plugin_gui::{GuiParamEvent, GuiParamEventKind, GuiTasks};
use rift_plugin_params::ParamCollection;
use rift_plugin_types::{EventPreProcess, EventSource};

use crate::ClapPlugin;

use super::WrapperProcessor;

impl<'a, P: ClapPlugin> WrapperProcessor<'a, P> {
    pub(super) fn handle_audio_thread_tasks(&mut self, outputs: &mut OutputEvents) {
        while let Some(task) = self.shared.states.pop_in_audio() {
            use AudioThreadTask::*;

            match task {
                GuiParamEvent(event) => self.handle_gui_param_change(event, outputs),
                RequestCallback => self.host.request_callback(),
            }
        }
    }

    fn request_flush(&self) {
        if let Some(ext) = self.host.get_extension::<HostParams>() {
            ext.request_flush(self.host.as_shared());
        } else {
            log::error!("Flush failed")
        }
    }

    pub(super) fn handle_input_events(&mut self, events: &InputEvents) {
        let flags = P::EVENT_PRE_PROCESS;
        let should_notify_gui = self.notify_gui && self.shared.states.is_gui_opened();

        if flags.is_empty() && !should_notify_gui {
            return;
        }

        for event in events.iter() {
            if let Some(event) = event.as_event::<ParamValueEvent>() {
                let Some(id) = event.param_id() else {
                    continue;
                };

                if flags.contains(EventPreProcess::PARAM_APPLY_CHANGE) {
                    let value = event.value();
                    self.shared.host_params.set_value(id, value as f32);
                }

                if flags.contains(EventPreProcess::PARAM_NOTIFY_CHANGE) {
                    self.plugin.on_param_change(
                        id,
                        &self.shared.params,
                        &self.shared.data,
                        EventSource::Host,
                    );
                }

                if should_notify_gui {
                    _ = self.shared.states.post_to_gui(GuiTasks::ParamChanged {
                        id,
                        value: event.value() as f32,
                    });
                }
            } else if flags.contains(EventPreProcess::MIDI_NOTIFY_EVENT)
                && let Some(event) = event.as_event::<MidiEvent>()
            {
                self.plugin.on_midi_message(
                    (*event).into(),
                    &self.shared.params,
                    &self.shared.data,
                );
            }
        }
    }

    #[inline]
    fn handle_gui_param_change(&mut self, event: GuiParamEvent, outputs: &mut OutputEvents) {
        if let Some(raw_event) = event.maybe_to_raw()
            && let err @ Err(..) = outputs.try_push(raw_event)
        {
            log::error!("There was an error push event {err:?}")
        }

        match event.kind {
            GuiParamEventKind::GestureBegin | GuiParamEventKind::GestureEnd => self.request_flush(),
            GuiParamEventKind::Value(value) => {
                let id = event.param_id;

                self.shared.host_params.set_value(id, value);
                self.plugin.on_param_change(
                    id,
                    &self.shared.params,
                    &self.shared.data,
                    EventSource::GUI,
                );
            }
            GuiParamEventKind::ValueLess => {
                self.plugin.on_param_change(
                    event.param_id,
                    &self.shared.params,
                    &self.shared.data,
                    EventSource::GUI,
                );
            }
        }
    }
}
