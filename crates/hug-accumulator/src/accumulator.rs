use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::audio_block::AudioBlock;

struct ChannelProducer<const N: usize> {
    buf: ArrayQueue<AudioBlock<N>>,
}

impl<const N: usize> ChannelProducer<N> {
    fn new(capacity: usize) -> Self {
        Self {
            buf: ArrayQueue::new(capacity),
        }
    }

    fn copy_slice_as_blocks(&self, slice: &[f32]) {
        for chunk in slice.chunks(N) {
            let _ = self.buf.push(AudioBlock::new(chunk));
        }
    }
}

pub struct AudioAccumulator<const N: usize> {
    channels: Vec<ChannelProducer<N>>,
    num_writes: AtomicU64,
}

impl<const N: usize> AudioAccumulator<N> {
    pub fn new(count: usize, block_count: usize) -> Self {
        let mut channels = Vec::with_capacity(count);
        for _ in 0..count {
            channels.push(ChannelProducer::new(block_count));
        }

        Self {
            channels,
            num_writes: AtomicU64::new(0),
        }
    }

    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    /// This function is totally lock free, audio thread
    /// can push slices here with no lock, mutex guard or alloc
    pub fn push_slices<'a>(&self, slices: impl Iterator<Item = &'a [f32]>) {
        for (channel, slice) in self.channels.iter().zip(slices) {
            channel.copy_slice_as_blocks(slice);
        }

        self.num_writes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn num_writes(&self) -> u64 {
        self.num_writes.load(Ordering::Relaxed)
    }

    /// This function is meant to be called on the UI thread
    /// locks are fine here
    pub fn drain<F>(&self, mut consume: F)
    where
        F: FnMut(&[f32], usize, usize),
    {
        let total_channels = self.channels();
        if total_channels == 0 {
            return;
        }

        loop {
            // pop one block per channel
            for idx in 0..total_channels {
                let Some(block) = self.channels[idx].buf.pop() else {
                    self.clear();
                    return;
                };

                consume(block.as_slice(), idx, total_channels);
            }
        }

        // guard lock will fall here, therefore
        // the first UI element that comes here drains all
        // data being written by audio thread.
    }

    fn clear(&self) {
        for drain_idx in 0..self.channels() {
            while self.channels[drain_idx].buf.pop().is_some() {}
        }
    }
}
