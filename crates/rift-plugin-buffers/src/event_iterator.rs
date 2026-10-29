use std::{iter::FusedIterator, ops::Range};

use clack_plugin::events::{
    UnknownEvent,
    event_types::{MidiEvent, ParamValueEvent},
    io::InputEvents,
};

use crate::InputEvent;

/// Walks a block's input events frame by frame.
///
/// `next_frame` and `iter_events_at` share one scan hint, so interleaving them
/// changes how much rescanning happens.
pub struct EventsCursor<'a> {
    /// `(start, end, frame)` of the last [`Self::bounds`] query.
    hint: Option<(usize, usize, u32)>,
    events: &'a InputEvents<'a>,
    /// `events.len()`, cached for the scan loops.
    events_len: usize,
    /// Frame the next [`Self::next_frame`] hands out.
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

    #[inline]
    pub fn events(&self) -> &'a InputEvents<'a> {
        &self.events
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
    pub fn iter_events_at(&mut self, time: u32) -> FrameEventsIter<'a> {
        let (start, end) = self.bounds(time);
        FrameEventsIter {
            events: self.events,
            indices: start..end,
        }
    }

    /// Indices of the events at `time`, resuming from the last query when going
    /// forward. Events are time ordered, so only a backward jump rescans.
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
    /// Indices into `events`; every one has the same `header().time()`.
    indices: Range<usize>,
}

impl<'a> Iterator for FrameEventsIter<'a> {
    type Item = InputEvent;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.by_ref().find_map(|i| convert(&self.events[i]))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.indices.len()))
    }
}

impl<'a> DoubleEndedIterator for FrameEventsIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices
            .by_ref()
            .rev()
            .find_map(|i| convert(&self.events[i]))
    }
}

impl<'a> FusedIterator for FrameEventsIter<'a> {}

/// Only param and MIDI events are modeled; the rest are skipped.
fn convert(event: &UnknownEvent) -> Option<InputEvent> {
    if let Some(&p) = event.as_event::<ParamValueEvent>() {
        Some(InputEvent::ParamEvent(p))
    } else if let Some(&m) = event.as_event::<MidiEvent>() {
        Some(InputEvent::MidiEvent(m.into()))
    } else {
        None
    }
}
