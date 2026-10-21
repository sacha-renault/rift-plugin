pub use clack_extensions::audio_ports::*;
pub use clack_extensions::params::*;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::prelude::*;

use rift_plugin_context::AudioThreadTask;
use rift_plugin_gui::{GuiEvent, GuiParamEventKind, ParamSync};
use rift_plugin_params::ParamCollection;

use crate::ClapPlugin;

impl<'a, P: ClapPlugin> PluginMainThreadParams for super::WrapperMainThread<'a, P> {
    fn count(&mut self) -> u32 {
        log::debug!(
            "PluginMainThreadParams::count {}",
            self.shared.host_params.count()
        );
        self.shared.host_params.count()
    }

    fn flush(&mut self, inputs: &InputEvents, outputs: &mut OutputEvents) {
        // The plugin is inactive in this path, so nothing else drains the tasks the GUI
        // queued. Report them through `outputs` before applying incoming automation.
        while let Some(task) = self.shared.states.audio_tasks().pop() {
            match task {
                AudioThreadTask::GuiParamEvent(event) => {
                    if let Some(raw_event) = event.maybe_to_raw()
                        && let Err(err) = outputs.try_push(raw_event)
                    {
                        log::error!("There was an error pushing event {err:?}");
                    }

                    if let GuiParamEventKind::Value(value) = event.kind {
                        self.shared.host_params.set_value(event.param_id, value);
                    }
                }
            }
        }

        let notify_gui =
            matches!(self.gui.param_sync(), ParamSync::Push) && self.shared.states.is_gui_opened();

        for event in inputs.iter() {
            if let Some(param_event) = event.as_event::<ParamValueEvent>() {
                let Some(id) = param_event.param_id() else {
                    continue;
                };
                let value = param_event.value() as f32;
                self.shared.host_params.set_value(id, value);

                if notify_gui {
                    self.shared
                        .states
                        .gui_tasks()
                        .post(GuiEvent::ParamChanged { id, value });
                }
            };
        }
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        log::debug!("PluginMainThreadParams::get_info {param_index}");
        if let Some(inf) = self.shared.host_params.get_param_info(param_index) {
            info.set(&inf);
        }
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        log::debug!("PluginMainThreadParams::get_value");
        self.shared
            .host_params
            .get_value(param_id)
            .map(|x| x as f64)
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &std::ffi::CStr) -> Option<f64> {
        log::debug!("PluginMainThreadParams::text_to_value");
        self.shared
            .host_params
            .text_to_value(param_id, text)
            .map(|x| x as f64)
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        log::debug!("PluginMainThreadParams::value_to_text");
        self.shared
            .host_params
            .value_to_text(param_id, value as f32, writer)
    }
}
