use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use rift_plugin_buffers::Buffer;
use rift_plugin_types::transport::*;

use crate::{accumulator::channel::RingBuffer, consumers::ConsumerDispatcher};

mod anchor;
mod channel;

use anchor::Anchor;
pub struct AudioAccumulator {
    /// One producer ring per allocated channel
    channels: Arc<[RingBuffer]>,
    capacity: usize,

    written: AtomicU64,
    read: AtomicU64,

    /// Timing of the latest written buffer.
    anchor: Anchor,
}

impl AudioAccumulator {
    pub fn new(channels: usize, capacity: usize) -> Self {
        let mut producers = Vec::new();
        producers.resize_with(channels, || RingBuffer::new(capacity));

        Self {
            channels: producers.into(),
            capacity,
            written: AtomicU64::new(0),
            read: AtomicU64::new(0),
            anchor: Anchor::new(),
        }
    }

    pub fn audio_thread_writes(&self, buffer: &Buffer, info: TransportInfo) {
        // TODO
        // Actually handle those case rather than doing nothing on
        // release builds.
        debug_assert_eq!(buffer.channels(), self.channels.len());
        debug_assert!(buffer.samples() < self.capacity);

        let start = self.written.load(Ordering::Relaxed);
        let w = (start % self.capacity as u64) as usize;
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

        self.anchor.store(start, &info);
        self.written
            .store(start + buffer.samples() as u64, Ordering::Release);
    }

    pub fn ui_thread_read(&self, consumers: &mut ConsumerDispatcher) {
        let written = self.written.load(Ordering::Acquire);
        let mut read = self.read.load(Ordering::Relaxed); // only UI thread touches this

        // Fell behind (GUI was closed)
        read = read.max(written.saturating_sub(self.capacity as u64));

        let available = (written - read) as usize;
        if available == 0 {
            return;
        }

        let r = (read % self.capacity as u64) as usize;
        let total_channels = self.channels.len();
        let time = self.anchor.time_at(read);

        if r + available <= self.capacity {
            self.dispatch_segment(consumers, r, available, total_channels, time);
        } else {
            let first_len = self.capacity - r;
            self.dispatch_segment(consumers, r, first_len, total_channels, time);
            self.dispatch_segment(
                consumers,
                0,
                available - first_len,
                total_channels,
                time.advanced_by(first_len as i64),
            );
        }

        self.read.store(written, Ordering::Relaxed);
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
