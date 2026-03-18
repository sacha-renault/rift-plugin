use std::sync::atomic::{AtomicU64, Ordering};

use rift_plugin_core::prelude::*;

use super::channel::ChannelProducer;
use crate::prelude::*;

/// The concrete, const-generic implementation behind [`AudioAccumulator`].
///
/// Each channel gets its own [`ChannelProducer<N>`] ring, where `N` is the
/// fixed frame-block size chosen at compile time. The struct is intentionally
/// kept private; callers interact with it only through the type-erased
/// [`AudioAccumulatorErased`] trait, which hides `N` behind an [`Arc`].
pub struct InnerAudioAccumulator<const N: usize> {
    /// One producer ring per allocated channel
    channels: Vec<ChannelProducer<N>>,

    /// Monotonically increasing counter incremented on every [`push_slices`] call.
    /// Read by the UI thread to detect new data without holding a lock.
    num_writes: AtomicU64,
    // TODO
    // We might want here to add a scratch to wait when buffer is smaller than N
    // (i mean much smaller, in case of buffer.len() == 8 and N = 512, we want to wait to fill a scratch before
    // sending it ...)
}

impl<const N: usize> InnerAudioAccumulator<N> {
    /// Allocates `max_channels` channel rings, each with a queue depth of
    /// `max_block_in_queue` blocks of `N` frames.
    ///
    /// This is the only place where heap allocation occurs for the accumulator;
    /// all subsequent push operations are lock-free and allocation-free.
    pub fn new(max_channels: usize, max_block_in_queue: usize) -> Self {
        let mut channels = Vec::with_capacity(max_channels);
        channels.resize_with(max_channels, || ChannelProducer::new(max_block_in_queue));

        Self {
            channels,
            num_writes: AtomicU64::new(0),
        }
    }
}

impl<const N: usize> super::private::Sealed for InnerAudioAccumulator<N> {}
impl<const N: usize> AudioAccumulatorErased for InnerAudioAccumulator<N> {
    fn channels(&self) -> usize {
        self.channels.len()
    }

    fn push_slices<'a>(
        &self,
        slices: &mut dyn Iterator<Item = &'a [f32]>,
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

    fn num_writes(&self) -> u64 {
        self.num_writes.load(Ordering::Relaxed)
    }

    fn drain(&self, consumers: &[ConsumerCell<dyn AudioConsumer>]) {
        let total_channels = self.channels();
        if total_channels == 0 {
            return;
        }

        loop {
            // pop one block per channel
            for idx in 0..total_channels {
                // todo!()
                // This might cause problem if allocated channel > bus channels
                // have to think about people allocating more than 2 channels "in case"
                // the host provide more than 2 channels. Maybe a fix would be to allocate a max
                // number of channel along with an Arc that define expected number of channels during
                // process call. This might be changed during the plugin activation phase.
                let Some(block) = self.channels[idx].buf.pop() else {
                    self.clear();
                    return;
                };

                // Create info for consumers
                let infos = ChannelsInfo {
                    current: idx,
                    total_channels,
                };
                let time = block.time();

                // Accumulate over ALL consumers
                for consumer_cell in consumers.iter() {
                    if let Ok(mut consumer) = consumer_cell.try_borrow_mut() {
                        consumer.consume(block.as_slice(), infos, time);
                    }
                }
            }
        }
    }

    fn clear(&self) {
        for drain_idx in 0..self.channels() {
            while self.channels[drain_idx].buf.pop().is_some() {}
        }
    }
}
