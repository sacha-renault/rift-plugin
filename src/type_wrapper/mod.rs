use clack_extensions::audio_ports::{AudioPortFlags, AudioPortInfo, AudioPortType};
use clack_plugin::utils::ClapId;

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
