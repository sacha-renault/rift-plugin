use std::sync::Arc;

use clack_extensions::{gui::*, params::HostParams};
use clack_plugin::{host::HostSharedHandle, plugin::PluginError, utils::ClapId};

use rift_plugin_params::ParamCollection;
use rift_plugin_types::TransportInfo;

use crate::{HostParamChange, ParamSync};

use super::events::GuiParamEvent;

pub trait GuiContext: Send + Sync {
    fn param_event(&self, event: GuiParamEvent);
    fn params(&self) -> Arc<dyn ParamCollection>;
    fn param_context_menu(&self, param_id: ClapId, x: i32, y: i32, screen: i32);
    fn global_context_menu(&self, x: i32, y: i32, screen: i32);
    fn is_playing(&self) -> bool;
    /// Returns the next host-side parameter change, oldest first.
    fn pop_in_gui(&self) -> Option<HostParamChange>;

    /// Returns `true` once if the GUI must reload every parameter value,
    /// because changes were dropped or the host loaded a new state.
    fn take_refresh_request(&self) -> bool;
    fn handle(&self) -> Option<HostSharedHandle<'static>>;
    fn latest_transport_info(&self) -> Option<TransportInfo>;

    #[inline]
    fn request_flush(&self) {
        let Some(host) = self.handle() else {
            return;
        };

        if let Some(ext) = host.get_extension::<HostParams>() {
            ext.request_flush(&host);
        }
    }
}

pub trait ClapGui {
    /// Set absolute scaling factor for GUI
    ///
    /// Overrides OS settings, and should not be used if the windowing API uses logical pixels. Can
    /// be ignored if the plugin will query the OS directly and perform its own calculations.
    fn set_scale(&mut self, scale: f64) -> Result<(), PluginError>;

    /// Get current size of GUI
    fn get_size(&mut self) -> Option<GuiSize>;

    /// Tell host if GUI can be resized
    ///
    /// Only applies to embedded windows.
    fn can_resize(&mut self) -> bool;

    /// Calculate the closest possible size for the GUI
    ///
    /// Only applies if the GUI is resizable and embedded in a parent window. Must return
    /// dimensions smaller than or equal to the requested dimensions.
    fn adjust_size(&mut self, size: GuiSize) -> Option<GuiSize>;

    /// Set the size of an embedded window
    fn set_size(&mut self, size: GuiSize) -> Result<(), PluginError>;

    /// Embed UI into the given parent window
    fn set_parent(&mut self, window: Window) -> Result<(), PluginError>;

    /// Receive instruction to stay above the given window
    ///
    /// Only applies to floating windows.
    fn set_transient(&mut self, window: Window) -> Result<(), PluginError>;

    /// Show the window
    fn show(&mut self) -> Result<(), PluginError>;

    /// Hide the window
    ///
    /// This should not free the resources associated with the GUI, just hide it.
    fn hide(&mut self) -> Result<(), PluginError>;

    /// allocates gui resources.
    ///
    /// After it the GUI may not be visible yet, show() is required.
    fn create(&mut self, configuration: GuiConfiguration) -> Result<(), PluginError>;

    /// Free all the resources of the GUI.
    fn destroy(&mut self);

    /// How this GUI wants param updates delivered.
    fn param_sync(&self) -> ParamSync;
}

pub trait GuiFactory {
    fn build(self: Box<Self>, ctx: Arc<dyn GuiContext>) -> Box<dyn ClapGui>;
}
