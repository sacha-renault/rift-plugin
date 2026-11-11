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
}

impl<'a> Iterator for BufferFrameIter<'a> {
    type Item = Frame<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame_position.next().map(|position| Frame {
            raw_data: self.raw_data,
            frame_position: position,
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
/// Iterating yields a reference to each channel's sample,
/// in channel order.
pub struct Frame<'a> {
    ///Raw data, not interleaved
    raw_data: &'a [*mut f32],

    /// Absolute position in the buffer.
    frame_position: usize,
}

impl<'a> Frame<'a> {
    /// Absolute time in the frame.
    #[inline]
    pub fn time(&self) -> u32 {
        self.frame_position as u32
    }

    /// Number of channels in this frame.
    #[inline]
    pub fn channels(&self) -> usize {
        self.raw_data.len()
    }

    pub fn copy_from_slice(&mut self, frame: &[f32]) {
        debug_assert_eq!(self.channels(), frame.len());

        for (sample, value) in self.iter_mut().zip(frame) {
            *sample = *value;
        }
    }

    pub fn copy_from_frame(&mut self, frame: &Frame) {
        debug_assert_eq!(self.channels(), frame.channels());

        for (sample, value) in self.iter_mut().zip(frame) {
            *sample = *value;
        }
    }

    /// Copy the frame's samples into `frame`, in channel order.
    pub fn copy_to_slice(&self, frame: &mut [f32]) {
        debug_assert_eq!(self.channels(), frame.len());

        for (sample, value) in self.iter().zip(frame) {
            *value = *sample;
        }
    }

    /// Fill the entire frame with `value`.
    pub fn fill(&mut self, value: f32) {
        for sample in self.iter_mut() {
            *sample = value;
        }
    }

    pub fn iter_mut(&mut self) -> FrameIterMut<'_> {
        FrameIterMut {
            channels: self.raw_data.iter(),
            offset: self.frame_position,
        }
    }

    pub fn iter(&self) -> FrameIter<'_> {
        FrameIter {
            channels: self.raw_data.iter(),
            offset: self.frame_position,
        }
    }
}

/// Shared iterator over a frame's samples, in channel order.
pub struct FrameIter<'a> {
    channels: std::slice::Iter<'a, *mut f32>,
    /// Frame position, added to each channel pointer.
    offset: usize,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = &'a f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: every channel pointer is valid for `offset` (guaranteed by
        // the buffer that built the frame).
        self.channels
            .next()
            .map(|&ptr| unsafe { &*ptr.add(self.offset) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.channels.size_hint()
    }
}

impl<'a> ExactSizeIterator for FrameIter<'a> {}

impl<'a> std::iter::FusedIterator for FrameIter<'a> {}

/// Mutable iterator over a frame's samples, in channel order.
pub struct FrameIterMut<'a> {
    channels: std::slice::Iter<'a, *mut f32>,
    /// Frame position, added to each channel pointer.
    offset: usize,
}

impl<'a> Iterator for FrameIterMut<'a> {
    type Item = &'a mut f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: every channel pointer is valid for `offset`, and the frame
        // is the only holder of this position's samples.
        self.channels
            .next()
            .map(|&ptr| unsafe { &mut *ptr.add(self.offset) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.channels.size_hint()
    }
}

impl<'a> ExactSizeIterator for FrameIterMut<'a> {}

impl<'a> std::iter::FusedIterator for FrameIterMut<'a> {}

impl<'a> IntoIterator for Frame<'a> {
    type Item = &'a mut f32;
    type IntoIter = FrameIterMut<'a>;

    fn into_iter(self) -> FrameIterMut<'a> {
        FrameIterMut {
            channels: self.raw_data.iter(),
            offset: self.frame_position,
        }
    }
}

impl<'a> IntoIterator for &'a Frame<'a> {
    type Item = &'a f32;
    type IntoIter = FrameIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FrameIter {
            channels: self.raw_data.iter(),
            offset: self.frame_position,
        }
    }
}

impl<'a, 'b> IntoIterator for &'b mut Frame<'a> {
    type Item = &'b mut f32;
    type IntoIter = FrameIterMut<'b>;

    fn into_iter(self) -> FrameIterMut<'b> {
        self.iter_mut()
    }
}
