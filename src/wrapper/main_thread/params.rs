pub use clack_extensions::audio_ports::*;
pub use clack_extensions::params::*;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::prelude::*;

use rift_plugin_shared::params::Params;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginMainThreadParams for super::WrapperMainThread<'a, P> {
    fn count(&mut self) -> u32 {
        log::debug!(
            "PluginMainThreadParams::count {}",
            self.shared.params.count()
        );
        self.shared.params.count()
    }

    fn flush(&mut self, inputs: &InputEvents, _outputs: &mut OutputEvents) {
        for event in inputs.iter() {
            if let Some(param_event) = event.as_event::<ParamValueEvent>() {
                let Some(id) = param_event.param_id() else {
                    continue;
                };
                let value = param_event.value();
                self.shared.params.set_value(id, value);
            };
        }
        // todo!()
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        log::debug!("PluginMainThreadParams::get_info {param_index}");
        if let Some(inf) = self.shared.params.get_param_info(param_index) {
            info.set(&inf);
        }
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        log::debug!("PluginMainThreadParams::get_value");
        self.shared.params.get_value(param_id)
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &std::ffi::CStr) -> Option<f64> {
        log::debug!("PluginMainThreadParams::text_to_value");
        self.shared.params.text_to_value(param_id, text)
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        log::debug!("PluginMainThreadParams::value_to_text");
        self.shared.params.value_to_text(param_id, value, writer)
    }
}
