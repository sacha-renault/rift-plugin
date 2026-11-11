use std::ops::Range;

/// Iterates over the buffer one frame at a time.
///
/// Each frame yields all channels at a given time step. For a stereo buffer
/// of 3 samples:
///
/// ```text
/// L: [1, 2, 3]
/// R: [4, 5, 6]
/// ```
///
/// `iter_frames()` → `Frame[1, 4], Frame[2, 5], Frame[3, 6]`
pub struct BufferFrameIter<'a> {
    pub(crate) raw_data: &'a [*mut f32],
    pub(crate) frame_position: Range<usize>,
    pub(crate) channels: usize,
}

impl<'a> Iterator for BufferFrameIter<'a> {
    type Item = Frame<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame_position.next().map(|position| Frame {
            raw_data: self.raw_data,
            frame_position: position,
            channels: self.channels,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> DoubleEndedIterator for BufferFrameIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.frame_position.next_back().map(|position| Frame {
            raw_data: self.raw_data,
            frame_position: position,
            channels: self.channels,
        })
    }
}

impl<'a> ExactSizeIterator for BufferFrameIter<'a> {
    fn len(&self) -> usize {
        self.frame_position.len()
    }
}

impl<'a> std::iter::FusedIterator for BufferFrameIter<'a> {}

/// A single frame: one sample per channel at a given time step.
///
/// Iterating yields a mutable reference to each channel's sample,
/// in channel order.
pub struct Frame<'a> {
    ///Raw data, not interleaved
    raw_data: &'a [*mut f32],

    /// Absolute position in the buffer.
    frame_position: usize,

    /// Number of channels in the buffer
    channels: usize,
}

impl<'a> Frame<'a> {
    /// Absolute time in the frame.
    #[inline]
    pub fn time(&self) -> u32 {
        self.frame_position as u32
    }

    pub fn copy_from_slice(&mut self, frame: &[f32]) {
        debug_assert_eq!(self.channels, frame.len());

        for (sample, value) in self.into_iter().zip(frame) {
            *sample = *value;
        }
    }

    pub fn fill(&mut self, value: f32) {
        for sample in self.into_iter() {
            *sample = value;
        }
    }

    /// This function is only visible in crate
    /// because it requires a mutable ref (there is only
    /// an iterator for mutable value.). TODO: add a const
    /// iterator and make this function with &self.
    pub(crate) fn copy_to_slice(&mut self, frame: &mut [f32]) {
        debug_assert_eq!(self.channels, frame.len());

        for (sample, value) in self.into_iter().zip(frame) {
            *value = *sample;
        }
    }

    pub fn iter_mut(&mut self) -> FrameIter<'_> {
        FrameIter {
            position: 0,
            frame: Frame {
                // &'a [..] shrinks to &'b [..]
                raw_data: self.raw_data,
                frame_position: self.frame_position,
                channels: self.channels,
            },
        }
    }
}

impl<'a> IntoIterator for Frame<'a> {
    type Item = &'a mut f32;
    type IntoIter = FrameIter<'a>;

    fn into_iter(self) -> FrameIter<'a> {
        FrameIter {
            position: 0,
            frame: self,
        }
    }
}

impl<'a, 'b> IntoIterator for &'b mut Frame<'a> {
    type Item = &'b mut f32;
    type IntoIter = FrameIter<'b>;

    fn into_iter(self) -> FrameIter<'b> {
        self.iter_mut()
    }
}

pub struct FrameIter<'a> {
    /// Position in the frame iter.
    /// Since FrameIterator yield all sample
    /// at a given position. This is equal to the
    /// next `channel` to yield.
    position: usize,
    frame: Frame<'a>,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = &'a mut f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.frame.channels {
            let position = self.position;
            self.position += 1;
            let ptr = self.frame.raw_data[position];

            unsafe { Some(&mut (*ptr.add(self.frame.frame_position))) }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for FrameIter<'a> {
    fn len(&self) -> usize {
        self.frame.channels - self.position
    }
}

impl<'a> std::iter::FusedIterator for FrameIter<'a> {}
