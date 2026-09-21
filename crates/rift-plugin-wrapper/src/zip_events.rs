use std::marker::PhantomData;

use clack_plugin::{
    events::event_types::{MidiEvent, ParamValueEvent},
    prelude::InputEvents,
};

use rift_plugin_buffers::frame::{Frame, SampleFrames};
use rift_plugin_types::MidiMessage;

use crate::ClapPlugin;

/// Pairs each audio frame with its corresponding CLAP input events.
///
/// This is an extension over [`SampleFrames`]: it zips the frame iterator with
/// an event stream, yielding `(FrameEvents, Frame)` pairs where each
/// `FrameEvents` contains only the events whose timestamp matches that frame's
/// position.
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
pub trait ZipEvents<'a> {
    fn zip_events<P: ClapPlugin>(self, events: &'a InputEvents) -> FramesEventZipped<'a, P>;
}

impl<'a> ZipEvents<'a> for SampleFrames<'a> {
    fn zip_events<P: ClapPlugin>(self, events: &'a InputEvents) -> FramesEventZipped<'a, P> {
        FramesEventZipped::from_frame_iter(self, events)
    }
}

pub enum InputEvent {
    MidiEvent(MidiMessage),
    ParamEvent(ParamValueEvent),
}

pub struct FramesEventZipped<'a, P: ClapPlugin> {
    inner: SampleFrames<'a>,
    events: &'a InputEvents<'a>,
    events_position: usize,
    _p: PhantomData<P>,
}

impl<'a, P: ClapPlugin> FramesEventZipped<'a, P> {
    fn from_frame_iter(frames: SampleFrames<'a>, events: &'a InputEvents) -> Self {
        Self {
            inner: frames,
            events,
            events_position: 0,
            _p: PhantomData,
        }
    }

    fn skip_until_time(&mut self, time: u32) {
        while self.events_position < self.events.len() as usize
            && self.events[self.events_position].header().time() <= time
        {
            self.events_position += 1;
        }
    }
}

impl<'a, P: ClapPlugin> Iterator for FramesEventZipped<'a, P> {
    type Item = (FrameEvents<'a, P>, Frame<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let time = self.inner.position() as u32;
        if let Some(frame) = self.inner.next() {
            let start = self.events_position;
            self.skip_until_time(time);

            let event_iter = FrameEvents {
                events: self.events,
                position: start,
                end: self.events_position,
                _p: PhantomData,
            };

            Some((event_iter, frame))
        } else {
            None
        }
    }
}

pub struct FrameEvents<'a, P: ClapPlugin> {
    events: &'a InputEvents<'a>,
    position: usize,
    end: usize,
    _p: PhantomData<P>,
}

impl<'a, P: ClapPlugin> Iterator for FrameEvents<'a, P> {
    type Item = InputEvent;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < self.end {
            let event = &self.events[self.position];
            self.position += 1;

            // We yield events here ONLY if the wrapper didn't already
            // "consume" them in the pre-process flush.

            // We don't wanna collapse those, because if one as_event is true
            // any other won't be
            #[allow(clippy::collapsible_if)]
            if let Some(&param_event) = event.as_event::<ParamValueEvent>() {
                if !P::PARAM_EVENT_AUTO_HANDLING {
                    return Some(InputEvent::ParamEvent(param_event));
                }
            } else if let Some(&midi_event) = event.as_event::<MidiEvent>() {
                if !P::MIDI_EVENT_AUTO_HANDLING {
                    return Some(InputEvent::MidiEvent(midi_event.into()));
                }
            }
            // Unknown event type, skip it, try next
        }

        None
    }
}
