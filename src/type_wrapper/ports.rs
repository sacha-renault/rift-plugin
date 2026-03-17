use clack_extensions::audio_ports::{AudioPortFlags, AudioPortInfo, AudioPortType};
use clack_plugin::utils::ClapId;

pub const PAIR_PORT_ID: ClapId = ClapId::new(0);

/// Represents a single audio port with name, channel count, flags, and type info.
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
    pub(crate) const fn new(name: &'a [u8], channel_count: u32, is_input: bool) -> Self {
        Self {
            name,
            channel_count,
            flags: AudioPortFlags::empty(),
            port_type: None,
            in_place_pair: None,
            is_input,
        }
    }

    pub const fn input(name: &'a [u8], channel_count: u32) -> Self {
        Self::new(name, channel_count, true)
    }

    pub const fn output(name: &'a [u8], channel_count: u32) -> Self {
        Self::new(name, channel_count, false)
    }

    pub const fn set_port_flags(mut self, flags: AudioPortFlags) -> Self {
        self.flags = self.flags.union(flags);
        self
    }

    pub const fn set_port_type(mut self, port_type: AudioPortType<'a>) -> Self {
        self.port_type = Some(port_type);
        self
    }

    pub const fn set_in_place(mut self, in_place_id: ClapId) -> Self {
        self.in_place_pair = Some(in_place_id);
        self
    }

    /// Converts this builder-like instance into an owned [`AudioPortInfo`] tagged with a plugin-local ID.
    pub fn into_audio_port_info(&self, index: u32) -> AudioPortInfo<'a> {
        AudioPortInfo {
            id: ClapId::new(index),
            name: self.name,
            channel_count: self.channel_count,
            flags: self.flags,
            port_type: self.port_type,
            in_place_pair: self.in_place_pair,
        }
    }
}

/// Specifies whether main port is inputs, outputs, or bidirectional IO for the plugin host.
#[derive(Debug, Clone, Copy)]
pub enum MainAudioPort {
    InputOnly(u32),
    OutputOnly(u32),
    InputOutput(u32),
}

impl MainAudioPort {
    /// Returns the number of channels associated with this I/O configuration variant.
    pub fn capacity(&self) -> u32 {
        match self {
            MainAudioPort::InputOnly(c)
            | MainAudioPort::OutputOnly(c)
            | MainAudioPort::InputOutput(c) => *c,
        }
    }
}
