use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

use rift_plugin_buffers::Buffer;
use rift_plugin_types::transport::*;

use crate::{accumulator::channel::RingBuffer, consumers::ConsumerDispatcher};

mod channel;

// #[cfg(test)]
// mod tests;

/// The concrete, const-generic implementation behind [`AudioAccumulator`].
///
/// Each channel gets its own [`ChannelProducer<N>`] ring, where `N` is the
/// fixed frame-block size chosen at compile time. The struct is intentionally
/// kept private; callers interact with it only through the type-erased
/// [`AudioAccumulatorErased`] trait, which hides `N` behind an [`Arc`].
pub struct AudioAccumulator {
    /// One producer ring per allocated channel
    channels: Arc<[RingBuffer]>,
    capacity: usize,

    /// Monotonically increasing counter incremented on every [`push_slices`] call.
    /// Read by the UI thread to detect new data without holding a lock.
    num_writes: AtomicU64,

    read_position: AtomicUsize,
    write_position: AtomicUsize,
    // --
    // TODO
    // add an anchor with a block position so the reader knows the actual time
    // of what he gets.
}

impl AudioAccumulator {
    pub fn new(channels: usize, capacity: usize) -> Self {
        let mut producers = Vec::new();
        producers.resize_with(channels, || RingBuffer::new(capacity));

        Self {
            channels: producers.into(),
            capacity,
            num_writes: AtomicU64::new(0),
            read_position: AtomicUsize::new(0),
            write_position: AtomicUsize::new(0),
        }
    }

    pub fn audio_thread_writes(&self, buffer: &Buffer) {
        // TODO
        // Actually handle those case rather than doing nothing on
        // release builds.
        debug_assert_eq!(buffer.channels(), self.channels.len());
        debug_assert!(buffer.samples() < self.capacity);

        let w = self.write_position.load(Ordering::Relaxed);
        let end = w + buffer.samples();

        let write_idx = if end <= self.capacity {
            WriteIdx::Contiguous { start: w }
        } else {
            WriteIdx::Splitted {
                start: w,
                end_wrap: self.capacity - w,
            }
        };

        for (ch, slice) in self.channels.iter().zip(buffer.iter_channels()) {
            ch.push_slice(write_idx, slice);
        }

        let new_pos = end % self.capacity;
        self.write_position.store(new_pos, Ordering::Release);
        self.num_writes.fetch_add(1, Ordering::Release);
    }

    pub fn ui_thread_read(&self, consumers: &mut ConsumerDispatcher) {
        let w = self.write_position.load(Ordering::Acquire);
        let r = self.read_position.load(Ordering::Relaxed); // only UI thread touches this

        if r == w {
            return; // nothing new since last read
        }

        let available = if w > r { w - r } else { self.capacity - r + w };
        let total_channels = self.channels.len();
        let time = BlockTime::none(); // TODO once transport timing lands

        if r + available <= self.capacity {
            self.dispatch_segment(consumers, r, available, total_channels, time);
        } else {
            let first_len = self.capacity - r;
            self.dispatch_segment(consumers, r, first_len, total_channels, time);
            self.dispatch_segment(consumers, 0, available - first_len, total_channels, time);
        }

        self.read_position.store(w, Ordering::Relaxed);
    }

    fn dispatch_segment(
        &self,
        consumers: &mut ConsumerDispatcher,
        start: usize,
        len: usize,
        total_channels: usize,
        time: BlockTime,
    ) {
        for (idx, ch) in self.channels.iter().enumerate() {
            let infos = ChannelsInfo {
                current: idx,
                total_channels,
            };
            ch.read_slice(start, len, |slice| consumers.dispatch(slice, infos, time));
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum WriteIdx {
    Contiguous { start: usize },
    Splitted { start: usize, end_wrap: usize },
}
