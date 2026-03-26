use clack_plugin::{
    plugin::PluginError,
    prelude::ChannelPair,
    process::{
        Audio,
        audio::{InputChannels, OutputChannels},
    },
};

use crate::prelude::MainAudioPort;

/// Handles audio buffer management for a plugin instance.
///
/// Wraps [`Audio`] and [`MainAudioPort`] to provide access to inputs, outputs, and the main I/O port.
/// Centralizes logic for retrieving buffers while accounting for host limitations (e.g., copying input to output).
///
/// # Note:
/// (and todo!()) since accessing any port requires a mutable reference, it isn't possible to use main and auxiliary port
/// in the same time. The plugin needs to hold a scratch buffer (allocated during activation) and copy required auxiliary port
/// into it.
pub struct Buffers<'a> {
    audio: Audio<'a>,
    main_config: MainAudioPort,
    is_main_copied: bool,
}

impl<'a> Buffers<'a> {
    /// Create a new view on [`clack_plugin::process::Audio`] struct.
    pub(crate) fn new(audio: Audio<'a>, main_config: MainAudioPort) -> Self {
        Self {
            audio,
            main_config,
            is_main_copied: false,
        }
    }

    /// Get the (not shifted by main port) input at `index`.
    fn get_input(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .input_port(index)
            .ok_or(PluginError::Message("No input ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input"))?;

        Ok(Buffer::input(data))
    }

    /// Get the (not shifted by main port) output at `index`.
    fn get_output(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let data = self
            .audio
            .output_port(index)
            .ok_or(PluginError::Message("No output ports found"))?
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 output"))?;

        Ok(Buffer::output(data))
    }

    /// Retrieve port pair 0 and copy, if needed, input into output.
    fn main_input_into_output(&mut self) -> Result<(), PluginError> {
        if self.is_main_copied {
            return Ok(());
        }

        let mut port_pair = self
            .audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut paired_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        self.is_main_copied = true;
        for paired in paired_channels.iter_mut() {
            // There is 4 cases
            // either InputOutput => handled with copy
            // Input only, should never happens
            // Output only, should never happens
            // Inplace ... In that case the output bfr
            // is already correct
            if let ChannelPair::InputOutput(i, o) = paired {
                o.copy_from_slice(i)
            }
        }

        Ok(())
    }

    /// First copy input into output and return the main output.
    ///
    /// This function must be called only in the case of [`MainAudioPort::InputOutput`].
    fn get_main_io(&mut self) -> Result<Buffer<'_>, PluginError> {
        self.main_input_into_output()?;
        self.get_output(0)
    }

    /// Get the declared main buffer.
    ///
    /// Depeding on [`MainAudioPort`], it can return different kinds of buffers. Wrapped in a
    /// convinient struct for consistant api calls.
    /// - [`MainAudioPort::InputOnly`]: the input channels.
    /// - [`MainAudioPort::OutputOnly`]: the output channels. This channels has no
    ///   certainty to be empty, so if you don't process it, you might want to set 0s at least
    ///   to avoid noizy output.
    /// - [`MainAudioPort::InputOutput`]: copies input into output then returns
    ///   the output buffer. If the host provides an in-place buffer, no copy
    ///   is performed and the single buffer is returned directly.
    pub fn main(&mut self) -> Result<Buffer<'_>, PluginError> {
        match self.main_config {
            MainAudioPort::InputOnly(_) => self.get_input(0),
            MainAudioPort::OutputOnly(_) => self.get_output(0),
            MainAudioPort::InputOutput(_) => self.get_main_io(),
        }
    }

    /// Same as [`Buffers::main`], but will panic if it returns a [`Result::Err`].
    pub fn main_unchecked(&mut self) -> Buffer<'_> {
        self.main().unwrap()
    }

    /// Returns the auxiliary input at `index`.
    ///
    /// As main ports are always 0, index may be shifted by one.
    pub fn input_aux(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let start_idx = match self.main_config {
            MainAudioPort::OutputOnly(_) => 0,
            _ => 1,
        };

        self.get_input(start_idx + index)
    }

    /// Same as [`Buffers::input_aux`], but will panic if it returns a [`Result::Err`].
    pub fn input_aux_unchecked(&mut self, index: usize) -> Buffer<'_> {
        self.input_aux(index).unwrap()
    }

    /// Returns the auxiliary output at `index`.
    ///
    /// As main ports are always 0, index may be shifted by one.
    pub fn output_aux(&mut self, index: usize) -> Result<Buffer<'_>, PluginError> {
        let start_idx = match self.main_config {
            MainAudioPort::InputOnly(_) => 0,
            _ => 1,
        };

        self.get_output(start_idx + index)
    }

    /// Same as [`Buffers::output_aux`], but will panic if it returns a [`Result::Err`].
    pub fn output_aux_unchecked(&mut self, index: usize) -> Buffer<'_> {
        self.output_aux(index).unwrap()
    }
}

pub enum Buffer<'a> {
    OutputChannels(OutputChannels<'a, f32>),
    InputChannels(InputChannels<'a, f32>),
    RawData {
        raw_data: &'a [*mut f32],
        frames_count: u32,
    },
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
    pub fn from_raw(raw_data: &'a [*mut f32], frames_count: u32) -> Self {
        Self::RawData {
            raw_data,
            frames_count,
        }
    }

    /// Return the number of channels in this buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        match self {
            Self::OutputChannels(data) => data.channel_count() as usize,
            Self::InputChannels(data) => data.channel_count() as usize,
            Self::RawData { raw_data, .. } => raw_data.len(),
        }
    }

    /// Return the number of samples per channel in this buffer
    #[inline]
    pub fn samples(&self) -> usize {
        match self {
            Self::OutputChannels(data) => data.frames_count() as usize,
            Self::InputChannels(data) => data.frames_count() as usize,
            Self::RawData { frames_count, .. } => *frames_count as usize,
        }
    }

    #[inline]
    pub(crate) fn raw_data(&'a self) -> &'a [*mut f32] {
        match self {
            Self::OutputChannels(data) => data.raw_data(),
            Self::InputChannels(data) => data.raw_data(),
            Self::RawData { raw_data, .. } => raw_data,
        }
    }

    /// Iterates sample-by-sample, yielding all channels at each time position.
    ///
    /// Note: because audio is stored in planar format (one contiguous buffer per
    /// channel), this iterator accesses non-contiguous memory at each step. The
    /// compiler cannot auto-vectorize this pattern. For pure per-channel processing
    /// (gain, EQ, distortion), prefer [`Buffer::iter_channels_mut`] which gives the
    /// compiler a straight contiguous slice to work with.
    #[allow(unused_mut)]
    pub fn iter_samples(&'a mut self) -> SamplesIterator<'a> {
        let samples = self.samples();
        let channels = self.channels();
        SamplesIterator {
            vec: self.raw_data(),
            position: 0,
            channels,
            samples,
        }
    }

    pub fn iter_channels(&self) -> impl Iterator<Item = &[f32]> {
        self.raw_data()
            .iter()
            .map(move |&ptr| unsafe { std::slice::from_raw_parts(ptr, self.samples()) })
    }

    /// Iterates channel-by-channel, yielding a contiguous `&mut [f32]` for each channel.
    ///
    /// Preferred for per-channel DSP (gain, filters, saturation) - the compiler
    /// can auto-vectorize over the contiguous slice. For inter-channel processing,
    /// see [`Buffer::iter_samples`].
    #[allow(unused_mut)]
    pub fn iter_channels_mut(&'a mut self) -> impl Iterator<Item = &'a mut [f32]> {
        let samples = self.samples();
        self.raw_data()
            .iter()
            .map(move |&ptr| unsafe { std::slice::from_raw_parts_mut(ptr, samples) })
    }
}

impl<'a> Drop for Buffers<'a> {
    fn drop(&mut self) {
        match self.main_config {
            MainAudioPort::InputOutput(_) if !self.is_main_copied => {
                let _ = self.main_input_into_output();
            }

            // Nothing to do on input only
            // We might wanna add later something
            // in the case output only, if it wasn't cleared might
            _ => {}
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
