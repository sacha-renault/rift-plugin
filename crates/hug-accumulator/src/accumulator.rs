use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{BlockTime, audio_block::TimedAudioBlock};

pub struct ChannelsInfo {
    pub current: usize,
    pub total_channels: usize,
}

struct ChannelProducer<const N: usize> {
    buf: ArrayQueue<TimedAudioBlock<N>>,
}

impl<const N: usize> ChannelProducer<N> {
    fn new(capacity: usize) -> Self {
        Self {
            buf: ArrayQueue::new(capacity),
        }
    }

    fn copy_slice_into_blocks(&self, slice: &[f32], seconds: f64, beats: f64) {
        for chunk in slice.chunks(N) {
            let audio_data = TimedAudioBlock::new(chunk, seconds, beats);
            let _ = self.buf.push(audio_data);
        }
    }
}

pub struct AudioAccumulator<const N: usize> {
    channels: Vec<ChannelProducer<N>>,
    num_writes: AtomicU64,
    // TODO
    // We might want here to add a scratch to wait when buffer is smaller than N
    // (i mean much smaller, in case of buffer.len() == 8 and N = 512, we want to wait to fill a scratch before
    // sending it ...)
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

    #[inline]
    pub fn push_slices_no_transport<'a>(&self, slices: impl Iterator<Item = &'a [f32]>) {
        self.push_slices(slices, f64::NAN, f64::NAN);
    }

    /// This function is totally lock free, audio thread
    /// can push slices here with no lock, mutex guard or alloc
    pub fn push_slices<'a>(
        &self,
        slices: impl Iterator<Item = &'a [f32]>,
        seconds: f64,
        beats: f64,
    ) {
        for (channel, slice) in self.channels.iter().zip(slices) {
            channel.copy_slice_into_blocks(slice, seconds, beats);
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
        F: FnMut(&[f32], ChannelsInfo, BlockTime),
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

                consume(
                    block.as_slice(),
                    ChannelsInfo {
                        current: idx,
                        total_channels,
                    },
                    block.time(),
                );
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
