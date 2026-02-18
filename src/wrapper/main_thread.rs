pub use clack_extensions::audio_ports::*;
use clack_extensions::gui;
pub use clack_extensions::gui::{GuiApiType, GuiConfiguration, PluginGuiImpl};
pub use clack_extensions::params::*;
use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;

use crate::params::param_trait::Params;
use crate::prelude::*;
use crate::type_wrapper::PAIR_PORT_ID;
use crate::wrapper::{ClapPlugin, shared::WrapperShared};

pub struct WrapperMainThread<'a, P: ClapPlugin> {
    pub(crate) shared: WrapperShared<P>,
    pub(crate) gui: Box<dyn ClapGui>, // TODO an actual gui thing here
    pub(crate) host: HostMainThreadHandle<'a>,
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<'a, P> {}

impl<'a, P: ClapPlugin> PluginAudioPortsImpl for WrapperMainThread<'a, P> {
    /// Returns the total number of audio ports for the given direction.
    ///
    /// Counts 1 for the main port (if it exists in the requested direction)
    /// plus any auxiliary ports declared in `P::AUX_AUDIO_PORTS`.
    fn count(&mut self, is_input: bool) -> u32 {
        let aux_number = P::AUX_AUDIO_PORTS
            .iter()
            .filter(|port| port.is_input == is_input)
            .count() as u32;

        match (P::MAIN_AUDIO_PORTS, is_input) {
            (MainAudioPort::InputOnly(_), true) => aux_number + 1,
            (MainAudioPort::OutputOnly(_), false) => aux_number + 1,
            (MainAudioPort::InputOutput(_), _) => aux_number + 1,
            _ => aux_number,
        }
    }

    /// Returns the port info for the given index and direction.
    ///
    /// Port indices are laid out as follows:
    /// - Index 0: the main port (input or output depending on `is_input`), if it exists
    ///   in the requested direction. For `InputOutput` plugins, both directions share
    ///   the same channel count and are marked as an in-place pair.
    /// - Index 1+: auxiliary ports from `P::AUX_AUDIO_PORTS`, filtered by direction,
    ///   in declaration order.
    ///
    /// # Panics
    /// Panics if `index` is out of range for the given direction. This should never
    /// happen if the host respects the count returned by [`count`]
    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        let main_port = match (P::MAIN_AUDIO_PORTS, is_input) {
            (MainAudioPort::InputOnly(channels), true) => {
                Some(AudioPort::input(b"Input", channels).set_port_flags(AudioPortFlags::IS_MAIN))
            }
            (MainAudioPort::OutputOnly(channels), false) => {
                Some(AudioPort::output(b"Output", channels).set_port_flags(AudioPortFlags::IS_MAIN))
            }
            (MainAudioPort::InputOutput(channels), _) => Some(
                AudioPort::new(
                    if is_input { b"Input" } else { b"Output" },
                    channels,
                    is_input,
                )
                .set_port_flags(AudioPortFlags::IS_MAIN)
                .set_in_place(PAIR_PORT_ID),
            ),
            _ => None,
        };

        // Create an iterator that optionally starts with the main port, then all matching aux ports
        let mut all_ports = main_port
            .iter()
            .chain(P::AUX_AUDIO_PORTS.iter().filter(|p| p.is_input == is_input));

        if let Some(port_info) = all_ports.nth(index as usize) {
            writer.set(&port_info.into_audio_port_info(index))
        } else {
            panic!("Invalid port index {} for is_input={}", index, is_input);
        }
    }
}

impl<'a, P: ClapPlugin> PluginStateImpl for WrapperMainThread<'a, P> {
    fn load(&mut self, _input: &mut clack_plugin::stream::InputStream) -> Result<(), PluginError> {
        Err(PluginError::Message("()")) // todo!()
    }

    fn save(
        &mut self,
        _output: &mut clack_plugin::stream::OutputStream,
    ) -> Result<(), PluginError> {
        Err(PluginError::Message("()")) // todo!()
    }
}

impl<'a, P: ClapPlugin> PluginMainThreadParams for WrapperMainThread<'a, P> {
    fn count(&mut self) -> u32 {
        log::debug!(
            "PluginMainThreadParams::count {}",
            self.shared.params.count()
        );
        self.shared.params.count()
    }

    fn flush(&mut self, _intputs: &InputEvents, _outputs: &mut OutputEvents) {
        log::debug!("PluginMainThreadParams::flush");
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

impl<'a, P: ClapPlugin> PluginGuiImpl for WrapperMainThread<'a, P> {
    fn is_api_supported(&mut self, configuration: gui::GuiConfiguration) -> bool {
        log::debug!("PluginGuiImpl::is_api_supported({configuration:?})");
        configuration.api_type
            == GuiApiType::default_for_current_platform().expect("Unsupported platform")
            && !configuration.is_floating
    }

    fn get_preferred_api(&mut self) -> Option<gui::GuiConfiguration<'_>> {
        Some(GuiConfiguration {
            api_type: GuiApiType::default_for_current_platform().expect("Unsupported platform"),
            is_floating: false,
        })
    }

    fn create(&mut self, _configuration: gui::GuiConfiguration) -> Result<(), PluginError> {
        //todo!()
        Ok(())
    }

    fn destroy(&mut self) {
        // todo!()
    }

    fn set_scale(&mut self, scale: f64) -> Result<(), PluginError> {
        self.gui.set_scale(scale)
    }

    fn get_size(&mut self) -> Option<gui::GuiSize> {
        self.gui.get_size()
    }

    fn set_size(&mut self, size: gui::GuiSize) -> Result<(), PluginError> {
        self.gui.set_size(size)
    }

    fn set_parent(&mut self, window: gui::Window) -> Result<(), PluginError> {
        self.gui.set_parent(window)
    }

    fn set_transient(&mut self, window: gui::Window) -> Result<(), PluginError> {
        self.gui.set_transient(window)
    }

    fn show(&mut self) -> Result<(), PluginError> {
        self.gui.show()
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        self.gui.hide()
    }
}
