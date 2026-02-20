use std::sync::Arc;

use clack_extensions::gui::{GuiSize, Window};
use clack_plugin::plugin::PluginError;

use crate::context::GuiContext;

pub trait ClapGui {
    fn spawn(&mut self);

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
}

pub trait IntoGui {
    fn into_gui(self: Box<Self>, states: Arc<GuiContext>) -> Box<dyn ClapGui>;
}
