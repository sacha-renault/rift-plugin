pub use clack_extensions::audio_ports::*;
pub use clack_extensions::gui::{GuiApiType, GuiConfiguration, PluginGuiImpl};
pub use clack_extensions::params::*;
use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;

use crate::wrapper::{ClapPlugin, shared::WrapperShared};

pub struct WrapperMainThread<P: ClapPlugin> {
    pub(crate) shared: WrapperShared<P>,
    pub(crate) gui: Option<f32>, // TODO an actual gui thing here
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<P> {}

impl<P: ClapPlugin> PluginAudioPortsImpl for WrapperMainThread<P> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input { 1 } else { 1 }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        todo!()
    }
}

impl<P: ClapPlugin> PluginStateImpl for WrapperMainThread<P> {
    fn load(&mut self, input: &mut clack_plugin::stream::InputStream) -> Result<(), PluginError> {
        todo!()
    }

    fn save(&mut self, output: &mut clack_plugin::stream::OutputStream) -> Result<(), PluginError> {
        todo!()
    }
}

impl<P: ClapPlugin> PluginMainThreadParams for WrapperMainThread<P> {
    fn count(&mut self) -> u32 {
        todo!()
    }

    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    ) {
        todo!()
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        todo!()
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        todo!()
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &std::ffi::CStr) -> Option<f64> {
        todo!()
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        todo!()
    }
}

impl<P: ClapPlugin> PluginGuiImpl for WrapperMainThread<P> {
    fn is_api_supported(&mut self, configuration: clack_extensions::gui::GuiConfiguration) -> bool {
        configuration.api_type
            == GuiApiType::default_for_current_platform().expect("Unsupported platform")
            && !configuration.is_floating
    }

    fn get_preferred_api(&'_ mut self) -> Option<clack_extensions::gui::GuiConfiguration<'_>> {
        Some(GuiConfiguration {
            api_type: GuiApiType::default_for_current_platform().expect("Unsupported platform"),
            is_floating: false,
        })
    }

    fn create(
        &mut self,
        _configuration: clack_extensions::gui::GuiConfiguration,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn destroy(&mut self) {
        todo!()
    }

    fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
        todo!()
    }

    fn get_size(&mut self) -> Option<clack_extensions::gui::GuiSize> {
        todo!()
    }

    fn set_size(&mut self, _size: clack_extensions::gui::GuiSize) -> Result<(), PluginError> {
        todo!()
    }

    fn set_parent(&mut self, window: clack_extensions::gui::Window) -> Result<(), PluginError> {
        todo!()
    }

    fn set_transient(&mut self, _window: clack_extensions::gui::Window) -> Result<(), PluginError> {
        todo!()
    }

    fn show(&mut self) -> Result<(), PluginError> {
        todo!()
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        todo!()
    }
}
