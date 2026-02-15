use clack_extensions::{
    audio_ports::{AudioPortFlags, AudioPortInfo, AudioPortType},
    gui::{GuiConfiguration, GuiResizeHints, GuiSize, Window},
};
use clack_plugin::{plugin::PluginError, utils::ClapId};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AudioPort<'a> {
    pub(crate) name: &'a [u8],
    pub(crate) channel_count: u32,
    pub(crate) flags: AudioPortFlags,
    pub(crate) port_type: Option<AudioPortType<'a>>,
    pub(crate) in_place_pair: Option<ClapId>,
    pub(crate) is_input: bool,
}

impl<'a> AudioPort<'a> {
    pub const fn input(name: &'a [u8], channel_count: u32) -> Self {
        AudioPort {
            name,
            channel_count,
            flags: AudioPortFlags::IS_MAIN,
            port_type: None,
            in_place_pair: None,
            is_input: true,
        }
    }

    pub const fn output(name: &'a [u8], channel_count: u32) -> Self {
        AudioPort {
            name,
            channel_count,
            flags: AudioPortFlags::IS_MAIN,
            port_type: None,
            in_place_pair: None,
            is_input: false,
        }
    }

    pub const fn set_port_flags(mut self, flags: AudioPortFlags) -> Self {
        self.flags = flags;
        self
    }

    pub const fn set_port_type(mut self, port_type: AudioPortType<'a>) -> Self {
        self.port_type = Some(port_type);
        self
    }

    pub fn into_audio_port_info(&self, index: u32) -> AudioPortInfo<'a> {
        AudioPortInfo {
            id: ClapId::new(index),
            name: self.name,
            channel_count: self.channel_count,
            flags: self.flags,
            port_type: self.port_type,
            in_place_pair: None,
        }
    }
}

pub trait ClapGui {
    /// Create and allocate all resources needed for the GUI
    ///
    /// If `is_floating` is true, the window will not be managed by the host. The plugin can set
    /// its window to stay above the parent window via [`Self::set_transient`].
    ///
    /// If `is_floating` is false, the plugin must embed its window in the parent (host).
    fn create(&mut self, configuration: GuiConfiguration) -> Result<(), PluginError>;

    /// Free all resources associated with the GUI
    fn destroy(&mut self);

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
    fn can_resize(&mut self) -> bool {
        false
    }

    /// Provide hints on the resize-ability of the GUI
    fn get_resize_hints(&mut self) -> Option<GuiResizeHints> {
        None
    }

    /// Calculate the closest possible size for the GUI
    ///
    /// Only applies if the GUI is resizable and embedded in a parent window. Must return
    /// dimensions smaller than or equal to the requested dimensions.
    fn adjust_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        None
    }

    /// Set the size of an embedded window
    fn set_size(&mut self, size: GuiSize) -> Result<(), PluginError>;

    /// Embed UI into the given parent window
    fn set_parent(&mut self, window: Window) -> Result<(), PluginError>;

    /// Receive instruction to stay above the given window
    ///
    /// Only applies to floating windows.
    fn set_transient(&mut self, window: Window) -> Result<(), PluginError>;

    /// Receive a suggested window title from the host
    ///
    /// Only applies to floating windows.
    fn suggest_title(&mut self, title: &str) {}

    /// Show the window
    fn show(&mut self) -> Result<(), PluginError>;

    /// Hide the window
    ///
    /// This should not free the resources associated with the GUI, just hide it.
    fn hide(&mut self) -> Result<(), PluginError>;
}
