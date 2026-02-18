use clack_plugin::{
    plugin::PluginError,
    prelude::ChannelPair,
    process::{
        Audio,
        audio::{InputChannels, OutputChannels, PairedChannels},
    },
};

use crate::prelude::MainAudioPort;

pub struct Buffers<'a> {
    audio: Audio<'a>,
    main_config: MainAudioPort,
}

impl<'a> Buffers<'a> {
    pub(crate) fn new(audio: Audio<'a>, main_config: MainAudioPort) -> Self {
        Self { audio, main_config }
    }

    fn get_main_input(&mut self) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .input_port(0)
            .ok_or(PluginError::Message("No output ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        Ok(Buffer::input(data))
    }

    fn get_main_output(&mut self) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .output_port(0)
            .ok_or(PluginError::Message("No output ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        Ok(Buffer::output(data))
    }

    fn get_main_io(&mut self) -> Result<Buffer<'_>, PluginError> {
        let mut port_pair = self
            .audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut paired_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        for paired in paired_channels.iter_mut() {
            match paired {
                // ChannelPair::InputOutput(i, o) => o.copy_from_slice(i),
                _ => unreachable!(),
            }
        }

        Err(PluginError::Message("()"))
    }

    pub fn main(&mut self) -> Result<Buffer<'_>, PluginError> {
        match self.main_config {
            MainAudioPort::InputOnly(_) => self.get_main_input(),
            MainAudioPort::OutputOnly(_) => self.get_main_output(),
            MainAudioPort::InputOutput(_) => self.get_main_io(),
        }
    }
}

pub enum Buffer<'a> {
    OutputChannels(OutputChannels<'a, f32>),
    InputChannels(InputChannels<'a, f32>),
}

impl<'a> Buffer<'a> {
    #[inline]
    pub(crate) fn output(data: OutputChannels<'a, f32>) -> Self {
        Self::OutputChannels(data)
    }

    #[inline]
    pub(crate) fn input(data: InputChannels<'a, f32>) -> Self {
        Self::InputChannels(data)
    }

    #[inline]
    pub fn channels(&self) -> usize {
        match self {
            Self::OutputChannels(data) => data.channel_count() as usize,
            Self::InputChannels(data) => data.channel_count() as usize,
        }
    }

    #[inline]
    pub fn samples(&self) -> usize {
        match self {
            Self::OutputChannels(data) => data.frames_count() as usize,
            Self::InputChannels(data) => data.frames_count() as usize,
        }
    }

    #[inline]
    pub fn raw_data(&'a self) -> &'a [*mut f32] {
        match self {
            Self::OutputChannels(data) => data.raw_data(),
            Self::InputChannels(data) => data.raw_data(),
        }
    }

    pub fn iter_samples(&'a self) -> SamplesIterator<'a> {
        let samples = self.samples();
        let channels = self.channels();
        SamplesIterator {
            vec: self.raw_data(),
            position: 0,
            channels,
            samples,
        }
    }

    pub fn iter_channels(&'a self) -> ChannelIterator<'a> {
        ChannelIterator {
            vec: self.raw_data(),
            position: 0,
            samples: self.samples(),
        }
    }
}

/// This struct iter over the buffer, yielding all
/// channels at time n
/// Ex: [
///     [1,2,3],
///     [4,5,6]
/// ]
/// Will yield [(1, 4), (2, 5), (3, 6)]
pub struct SamplesIterator<'a> {
    vec: &'a [*mut f32],
    position: usize,
    channels: usize,
    samples: usize,
}

impl<'a> Iterator for SamplesIterator<'a> {
    type Item = ChannelSamples<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples {
            let item = Some(ChannelSamples {
                vec: self.vec,
                position: 0,
                channel_position: self.position,
                channels: self.channels,
            });
            self.position += 1;
            item
        } else {
            None
        }
    }
}

pub struct ChannelSamples<'a> {
    vec: &'a [*mut f32],
    channel_position: usize,
    position: usize,
    channels: usize,
}

impl<'a> Iterator for ChannelSamples<'a> {
    type Item = &'a mut f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.channels {
            let position = self.position;
            self.position += 1;
            let ptr = self.vec[position];

            unsafe { Some(&mut (*ptr.add(self.channel_position))) }
        } else {
            None
        }
    }
}

pub struct ChannelIterator<'a> {
    vec: &'a [*mut f32],
    position: usize,
    samples: usize,
}

impl<'a> Iterator for ChannelIterator<'a> {
    type Item = &'a mut [f32];

    fn next(&mut self) -> Option<Self::Item> {
        let length = self.vec.len();
        if self.position < length {
            let position = self.position;
            self.position += 1;
            let ptr = self.vec[position];
            todo!()
            // let mut slice = unsafe { std::slice::from_raw_parts_mut(ptr, self.samples) };
            // unsafe { Some(&mut slice) }
        } else {
            None
        }
    }
}
