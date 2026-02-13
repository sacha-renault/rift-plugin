pub use clack_extensions::audio_ports::*;
use clack_extensions::gui;
pub use clack_extensions::gui::{GuiApiType, GuiConfiguration, PluginGuiImpl};
pub use clack_extensions::params::*;
use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;

use crate::params::param_trait::Params;
use crate::wrapper::{ClapPlugin, shared::WrapperShared};

pub struct WrapperMainThread<'a, P: ClapPlugin> {
    pub(crate) shared: WrapperShared<P>,
    pub(crate) gui: Option<f32>, // TODO an actual gui thing here
    pub(crate) host: HostMainThreadHandle<'a>,
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<'a, P> {}

impl<'a, P: ClapPlugin> PluginAudioPortsImpl for WrapperMainThread<'a, P> {
    fn count(&mut self, is_input: bool) -> u32 {
        1 // todo!()
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        // todo!()
    }
}

impl<'a, P: ClapPlugin> PluginStateImpl for WrapperMainThread<'a, P> {
    fn load(&mut self, input: &mut clack_plugin::stream::InputStream) -> Result<(), PluginError> {
        Ok(()) // todo!()
    }

    fn save(&mut self, output: &mut clack_plugin::stream::OutputStream) -> Result<(), PluginError> {
        Ok(()) // todo!()
    }
}

impl<'a, P: ClapPlugin> PluginMainThreadParams for WrapperMainThread<'a, P> {
    fn count(&mut self) -> u32 {
        self.shared.params.count()
    }

    fn flush(&mut self, intput_events: &InputEvents, output_events: &mut OutputEvents) {
        // todo!()
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        if let Some(inf) = self.shared.params.get_param_info(param_index) {
            info.set(&inf);
        }
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        self.shared.params.get_value(param_id)
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &std::ffi::CStr) -> Option<f64> {
        self.shared.params.text_to_value(param_id, text)
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        self.shared.params.value_to_text(param_id, value, writer)
    }
}

// impl<'a, P: ClapPlugin> PluginGuiImpl for WrapperMainThread<'a, P> {
//     fn is_api_supported(&mut self, configuration: gui::GuiConfiguration) -> bool {
//         configuration.api_type
//             == GuiApiType::default_for_current_platform().expect("Unsupported platform")
//             && !configuration.is_floating
//     }

//     fn get_preferred_api(&'_ mut self) -> Option<gui::GuiConfiguration<'_>> {
//         Some(GuiConfiguration {
//             api_type: GuiApiType::default_for_current_platform().expect("Unsupported platform"),
//             is_floating: false,
//         })
//     }

//     fn create(&mut self, _configuration: gui::GuiConfiguration) -> Result<(), PluginError> {
//         Ok(())
//     }

//     fn destroy(&mut self) {
//         // todo!()
//     }

//     fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
//         Err(PluginError::Message(":(")) // todo!()
//     }

//     fn get_size(&mut self) -> Option<gui::GuiSize> {
//         None // todo!()
//     }

//     fn set_size(&mut self, _size: gui::GuiSize) -> Result<(), PluginError> {
//         Err(PluginError::Message(":(")) // todo!()
//     }

//     fn set_parent(&mut self, _window: gui::Window) -> Result<(), PluginError> {
//         Err(PluginError::Message(":(")) // todo!()
//     }

//     fn set_transient(&mut self, _window: gui::Window) -> Result<(), PluginError> {
//         Err(PluginError::Message(":(")) // todo!()
//     }

//     fn show(&mut self) -> Result<(), PluginError> {
//         Ok(()) // todo!()
//     }

//     fn hide(&mut self) -> Result<(), PluginError> {
//         Ok(()) // todo!()
//     }
// }
