use std::marker::PhantomData;

use clack_plugin::{
    events::event_types::{MidiEvent, ParamValueEvent},
    prelude::InputEvents,
};

use crate::{
    event_handling::{InputEvent, ZipEventConfig},
    frame::{Frame, SampleFrames},
};

pub struct FramesEventZipped<'a, C: ZipEventConfig> {
    inner: SampleFrames<'a>,
    events: &'a InputEvents<'a>,
    events_position: usize,
    _p: PhantomData<C>,
}

impl<'a, C: ZipEventConfig> FramesEventZipped<'a, C> {
    pub(crate) fn from_frame_iter(frames: SampleFrames<'a>, events: &'a InputEvents) -> Self {
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

impl<'a, C: ZipEventConfig> Iterator for FramesEventZipped<'a, C> {
    type Item = (FrameEvents<'a, C>, Frame<'a>);

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

pub struct FrameEvents<'a, C: ZipEventConfig> {
    events: &'a InputEvents<'a>,
    position: usize,
    end: usize,
    _p: PhantomData<C>,
}

impl<'a, C: ZipEventConfig> Iterator for FrameEvents<'a, C> {
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
                if !C::PARAM_EVENT_AUTO_HANDLING {
                    return Some(InputEvent::ParamEvent(param_event));
                }
            } else if let Some(&midi_event) = event.as_event::<MidiEvent>() {
                if !C::MIDI_EVENT_AUTO_HANDLING {
                    return Some(InputEvent::MidiEvent(midi_event.into()));
                }
            }
            // Unknown event type, skip it, try next
        }

        None
    }
}
