use clack_plugin::prelude::InputEvents;

use crate::{buffers::zip_events::FramesEventZipped, wrapper::ClapPlugin};

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
    /// Pairs each audio frame with its corresponding CLAP input events.
    ///
    /// This method zips the frame iterator with an event stream, yielding
    /// `(FrameEvents, Frame)` pairs where each `FrameEvents` contains only
    /// the events whose timestamp matches that frame's position.
    ///
    /// Events that were already auto-handled by the wrapper (controlled by
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
    pub fn zip_events<P: ClapPlugin>(self, events: &'a InputEvents) -> FramesEventZipped<'a, P> {
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
