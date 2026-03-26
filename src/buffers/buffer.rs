use clack_plugin::process::audio::{InputChannels, OutputChannels};

use crate::buffers::frame::SampleFrames;

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
    /// See the benchmark example for more info.
    #[allow(unused_mut)]
    pub fn iter_samples(&'a mut self) -> SampleFrames<'a> {
        let samples = self.samples();
        let channels = self.channels();
        SampleFrames {
            vec: self.raw_data(),
            position: 0,
            channels,
            samples,
        }
    }

    pub fn iter_channels(&self) -> impl Iterator<Item = &[f32]> {
        let samples = self.samples();
        self.raw_data()
            .iter()
            .map(move |&ptr| unsafe { std::slice::from_raw_parts(ptr, samples) })
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
