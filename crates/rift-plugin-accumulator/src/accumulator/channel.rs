use std::cell::UnsafeCell;

// use rift_plugin_types::transport::{BlockInfo, BlockTime};

use crate::accumulator::WriteIdx;
// use crate::prelude::TimedAudioBlock;

/// A lock-free, single-channel audio block producer.
///
/// Slices of PCM samples pushed from the audio thread are chopped into
/// fixed-size [`TimedAudioBlock<N>`] chunks and enqueued into an
/// [`ArrayQueue`]. The consumer side (UI thread) pops blocks out of
/// [`Self::buf`] directly.
pub(crate) struct RingBuffer {
    /// Fixed size array that will be used as a rb
    buffer: UnsafeCell<Box<[f32]>>,
}

impl RingBuffer {
    /// Creates a new `ChannelProducer` with a queue that can hold up to
    /// `capacity` blocks before dropping incoming data.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: UnsafeCell::new(vec![0f32; capacity].into_boxed_slice()),
        }
    }

    pub fn push_slice(&self, write_idx: WriteIdx, slice: &[f32]) {
        match write_idx {
            WriteIdx::Contiguous { start } => unsafe {
                let dst = self.ring_ptr_mut(start);
                std::ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice.len());
            },
            WriteIdx::Splitted { start, end_wrap } => unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), self.ring_ptr_mut(start), end_wrap);
                std::ptr::copy_nonoverlapping(
                    slice.as_ptr().add(end_wrap),
                    self.ring_ptr_mut(0),
                    slice.len() - end_wrap,
                );
            },
        }
    }

    /// SAFETY: only ever called from the UI/consumer thread.
    pub fn read_slice(&self, start: usize, len: usize, f: impl FnOnce(&[f32])) {
        unsafe {
            let ptr = self.ring_ptr(start);
            f(std::slice::from_raw_parts(ptr, len));
        }
    }

    /// SAFETY: Caller must be audio thread
    unsafe fn ring_ptr_mut(&self, offset: usize) -> *mut f32 {
        unsafe { (*self.buffer.get()).as_mut_ptr().add(offset) }
    }

    unsafe fn ring_ptr(&self, offset: usize) -> *const f32 {
        unsafe { (*self.buffer.get()).as_ptr().add(offset) }
    }
}
