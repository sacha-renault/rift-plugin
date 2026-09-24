use clack_plugin::events::io::InputEvents;

use crate::{event_handling::ZipEventConfig, zip_event::FramesEventZipped};

/// Iterates over the buffer one sample frame at a time.
///
/// Each frame yields all channels at a given time step. For a stereo buffer
/// of 3 samples:
///
/// ```text
/// L: [1, 2, 3]
/// R: [4, 5, 6]
///
/// iter_samples() → Frame[1, 4], Frame[2, 5], Frame[3, 6]
/// ```
pub struct SampleFrames<'a> {
    pub(crate) vec: &'a [*mut f32],
    pub(crate) position: usize,
    pub(crate) channels: usize,
    pub(crate) samples: usize,
}

impl<'a> Iterator for SampleFrames<'a> {
    type Item = Frame<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples {
            let item = Some(Frame {
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

impl<'a> SampleFrames<'a> {
    /// Absolute sample position of the current frame.
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Pairs each audio frame with its corresponding CLAP input events.
    ///
    /// This is an extension over [`SampleFrames`]: it zips the frame iterator with
    /// an event stream, yielding `(FrameEvents, Frame)` pairs where each
    /// `FrameEvents` contains only the events whose timestamp matches that frame's
    /// position.
    ///
    /// Your plugin needs to implement [`ZipEventConfig`] so you can use ::<Self> in the process function.
    /// ```ignore
    /// impl ZipEventConfig for YourPlugin {
    ///     const MIDI_EVENT_AUTO_HANDLING: bool = <YourPlugin as ClapPlugin>::MIDI_EVENT_AUTO_HANDLING;
    ///     const PARAM_EVENT_AUTO_HANDLING: bool = <YourPlugin as ClapPlugin>::PARAM_EVENT_AUTO_HANDLING;
    /// }
    /// ```
    ///
    /// Events already auto-handled by the wrapper (controlled by
    /// [`ClapPlugin::PARAM_EVENT_AUTO_HANDLING`] and
    /// [`ClapPlugin::MIDI_EVENT_AUTO_HANDLING`]) are silently skipped.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for (events, frame) in sample_frames.zip_events::<Self>(&input_events) {
    ///     for event in events {
    ///         match event {
    ///             InputEvent::MidiEvent(msg) => { /* handle MIDI */ }
    ///             InputEvent::ParamEvent(val) => { /* handle param change */ }
    ///         }
    ///     }
    ///     // process `frame` audio data
    /// }
    /// ```
    pub fn zip_events<C: ZipEventConfig>(
        self,
        events: &'a InputEvents,
    ) -> FramesEventZipped<'a, C> {
        FramesEventZipped::from_frame_iter(self, events)
    }
}

/// A single sample frame: one sample per channel at a given time step.
///
/// Iterating yields a mutable reference to each channel's sample,
/// in channel order.
pub struct Frame<'a> {
    vec: &'a [*mut f32],
    channel_position: usize,
    position: usize,
    channels: usize,
}

impl<'a> Iterator for Frame<'a> {
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

impl<'a> Frame<'a> {
    pub fn as_slice<const N: usize>(&mut self) -> [&'a mut f32; N] {
        assert!(N < self.channels);

        let mut samples: [&'a mut f32; N] =
            std::array::from_fn(|_| unsafe { &mut *(self.vec[0].add(0)) });

        for (i, sample) in samples.iter_mut().enumerate() {
            let ptr = self.vec[i];
            *sample = unsafe { &mut (*ptr.add(self.channel_position)) };
        }
        samples
    }

    pub fn as_stereo_slice(&mut self) -> [&'a mut f32; 2] {
        self.as_slice::<2>()
    }
}
