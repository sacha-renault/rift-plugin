use clack_plugin::{
    events::event_types::{MidiEvent, ParamValueEvent},
    prelude::InputEvents,
};

use crate::{
    buffers::frame::{Frame, SampleFrames},
    prelude::MidiMessage,
};

pub enum InputEvent {
    MidiEvent(MidiMessage),
    ParamEvent(ParamValueEvent),
}

pub struct FramesEventZipped<'a> {
    inner: SampleFrames<'a>,
    events: &'a InputEvents<'a>,
    events_position: usize,
}

impl<'a> FramesEventZipped<'a> {
    pub(crate) fn from_frame_iter(frames: SampleFrames<'a>, events: &'a InputEvents) -> Self {
        Self {
            inner: frames,
            events,
            events_position: 0,
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

impl<'a> Iterator for FramesEventZipped<'a> {
    type Item = (FrameEvents<'a>, Frame<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let time = self.inner.position as u32;
        if let Some(frame) = self.inner.next() {
            let start = self.events_position;
            self.skip_until_time(time);

            let event_iter = FrameEvents {
                events: self.events,
                position: start,
                end: self.events_position,
            };

            Some((event_iter, frame))
        } else {
            None
        }
    }
}

pub struct FrameEvents<'a> {
    events: &'a InputEvents<'a>,
    position: usize,
    end: usize,
}

impl<'a> Iterator for FrameEvents<'a> {
    type Item = InputEvent;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < self.end {
            let event = &self.events[self.position];
            self.position += 1;

            if let Some(&param_event) = event.as_event::<ParamValueEvent>() {
                return Some(InputEvent::ParamEvent(param_event));
            } else if let Some(&midi_event) = event.as_event::<MidiEvent>() {
                return Some(InputEvent::MidiEvent(midi_event.into()));
            }
            // Unknown event type, skip it, try next
        }

        None
    }
}
