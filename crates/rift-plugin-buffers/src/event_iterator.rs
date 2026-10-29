use clack_plugin::events::{
    event_types::{MidiEvent, ParamValueEvent},
    io::InputEvents,
};

use crate::InputEvent;

pub struct EventsCursor<'a> {
    hint: Option<(usize, usize, u32)>,
    events: &'a InputEvents<'a>,
    events_len: usize,
    current_frame: u32,
}

impl<'a> EventsCursor<'a> {
    pub fn new(events: &'a InputEvents<'a>) -> Self {
        Self {
            hint: None,
            events_len: events.len() as usize,
            events,
            current_frame: 0,
        }
    }

    /// Returns an iterator for the current frame and increment
    /// the frame so that next call is on next frame
    pub fn next_frame(&mut self) -> FrameEventsIter<'a> {
        let frame = self.current_frame;
        self.current_frame += 1;
        self.iter_events_at(frame)
    }

    /// Events of `frame`. Cheapest when called with increasing frames;
    /// going backwards restarts the scan from the beginning.
    pub fn iter_events_at(&mut self, frame: u32) -> FrameEventsIter<'a> {
        let (start, end) = self.bounds(frame);
        FrameEventsIter {
            events: self.events,
            position: start,
            end,
        }
    }

    fn bounds(&mut self, time: u32) -> (usize, usize) {
        let t = |i: usize| self.events[i].header().time();

        let mut start = match self.hint {
            // Same query again: free
            Some((s, e, prev)) if prev == time => return (s, e),
            // Moving forward: resume from the previous end
            Some((_, e, prev)) if prev < time => e,
            // First call or went backwards: restart
            // Maybe think about implementing binary search ?
            // Might be overkill. Anyway start search from scratch for now.
            _ => 0,
        };

        while start < self.events_len && t(start) < time {
            start += 1;
        }
        let mut end = start;
        while end < self.events_len && t(end) == time {
            end += 1;
        }

        self.hint = Some((start, end, time));
        (start, end)
    }
}

pub struct FrameEventsIter<'a> {
    events: &'a InputEvents<'a>,
    position: usize,
    end: usize,
}

impl<'a> Iterator for FrameEventsIter<'a> {
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
