use crossbeam_queue::ArrayQueue;
use hug_shared::{BlockInfo, BlockTime, ChannelsInfo};
use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::audio_block::TimedAudioBlock;

struct ChannelProducer<const N: usize> {
    buf: ArrayQueue<TimedAudioBlock<N>>,
}

impl<const N: usize> ChannelProducer<N> {
    fn new(capacity: usize) -> Self {
        Self {
            buf: ArrayQueue::new(capacity),
        }
    }

    fn copy_slice_into_blocks_no_info(&self, slice: &[f32]) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::none();
            let audio_data = TimedAudioBlock::new(chunk, time);
            let _ = self.buf.push(audio_data);
        }
    }

    fn copy_slice_into_blocks(&self, slice: &[f32], mut block_info: BlockInfo) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::new(block_info.seconds, block_info.beats);
            let audio_data = TimedAudioBlock::new(chunk, time);
            block_info.advance_by_samples(audio_data.len());
            let _ = self.buf.push(audio_data);
        }
    }
}

#[derive(Clone)]
pub struct AudioAccumulator<const N: usize> {
    inner: Arc<InnerAudioAccumulator<N>>,
}

impl<const N: usize> Deref for AudioAccumulator<N> {
    type Target = InnerAudioAccumulator<N>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct InnerAudioAccumulator<const N: usize> {
    channels: Vec<ChannelProducer<N>>,
    num_writes: AtomicU64,
    // TODO
    // We might want here to add a scratch to wait when buffer is smaller than N
    // (i mean much smaller, in case of buffer.len() == 8 and N = 512, we want to wait to fill a scratch before
    // sending it ...)
}

impl<const N: usize> InnerAudioAccumulator<N> {
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
    pub fn push_slices<'a>(
        &self,
        slices: impl Iterator<Item = &'a [f32]>,
        block_info_opt: Option<BlockInfo>,
    ) {
        if let Some(block_info) = block_info_opt {
            for (channel, slice) in self.channels.iter().zip(slices) {
                channel.copy_slice_into_blocks(slice, block_info.clone());
            }
        } else {
            for (channel, slice) in self.channels.iter().zip(slices) {
                channel.copy_slice_into_blocks_no_info(slice);
            }
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
