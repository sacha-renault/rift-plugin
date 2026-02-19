use clack_extensions::gui;
pub use clack_extensions::gui::{GuiApiType, GuiConfiguration, PluginGuiImpl};
use clack_plugin::prelude::*;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginGuiImpl for super::WrapperMainThread<'a, P> {
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
