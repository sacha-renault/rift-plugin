use clack_plugin::prelude::InputEvents;

use crate::{Frame, FrameEventsIter, SampleFrames, event_iterator::EventsCursor};

pub struct FramesEventZipped<'a> {
    inner: SampleFrames<'a>,
    cursor: EventsCursor<'a>,
}

impl<'a> FramesEventZipped<'a> {
    pub(crate) fn from_frame_iter(frames: SampleFrames<'a>, events: &'a InputEvents<'a>) -> Self {
        Self {
            inner: frames,
            cursor: EventsCursor::new(events),
        }
    }
}

impl<'a> Iterator for FramesEventZipped<'a> {
    type Item = (FrameEventsIter<'a>, Frame<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let time = self.inner.position() as u32;
        let frame = self.inner.next()?;
        Some((self.cursor.iter_events_at(time), frame))
    }
}
