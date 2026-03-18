use crossbeam_queue::ArrayQueue;
use rift_plugin_core::transport::{BlockInfo, BlockTime};

use crate::prelude::TimedAudioBlock;

/// A lock-free, single-channel audio block producer.
///
/// Slices of PCM samples pushed from the audio thread are chopped into
/// fixed-size [`TimedAudioBlock<N>`] chunks and enqueued into an
/// [`ArrayQueue`]. The consumer side (UI thread) pops blocks out of
/// [`Self::buf`] directly.
pub(crate) struct ChannelProducer<const N: usize> {
    /// The bounded ring buffer shared between the audio thread (producer)
    /// and the UI thread (consumer).
    pub buf: ArrayQueue<TimedAudioBlock<N>>,
}

impl<const N: usize> ChannelProducer<N> {
    /// Creates a new `ChannelProducer` with a queue that can hold up to
    /// `capacity` blocks before dropping incoming data.
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: ArrayQueue::new(capacity),
        }
    }

    /// Splits `slice` into `N`-frame chunks and enqueues each one without
    /// any timing information.
    ///
    /// Blocks that cannot be enqueued because the queue is full are silently
    /// dropped.
    pub fn copy_slice_into_blocks_no_info(&self, slice: &[f32]) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::none();
            let audio_data = TimedAudioBlock::new(chunk, time);
            let _ = self.buf.push(audio_data);
        }
    }

    /// Splits `slice` into `N`-frame chunks and enqueues each one with
    /// accurate transport timing.
    ///
    /// Blocks that cannot be enqueued because the queue is full are silently
    /// dropped.
    pub fn copy_slice_into_blocks(&self, slice: &[f32], mut block_info: BlockInfo) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::new(block_info.seconds, block_info.beats);
            let audio_data = TimedAudioBlock::new(chunk, time);
            block_info.advance_by_samples(audio_data.len());
            let _ = self.buf.push(audio_data);
        }
    }
}
