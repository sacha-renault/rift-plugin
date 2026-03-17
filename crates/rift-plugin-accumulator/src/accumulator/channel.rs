use crossbeam_queue::ArrayQueue;
use rift_plugin_shared::transport::{BlockInfo, BlockTime};

use crate::prelude::TimedAudioBlock;

pub(crate) struct ChannelProducer<const N: usize> {
    pub buf: ArrayQueue<TimedAudioBlock<N>>,
}

impl<const N: usize> ChannelProducer<N> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: ArrayQueue::new(capacity),
        }
    }

    pub fn copy_slice_into_blocks_no_info(&self, slice: &[f32]) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::none();
            let audio_data = TimedAudioBlock::new(chunk, time);
            let _ = self.buf.push(audio_data);
        }
    }

    pub fn copy_slice_into_blocks(&self, slice: &[f32], mut block_info: BlockInfo) {
        for chunk in slice.chunks(N) {
            let time = BlockTime::new(block_info.seconds, block_info.beats);
            let audio_data = TimedAudioBlock::new(chunk, time);
            block_info.advance_by_samples(audio_data.len());
            let _ = self.buf.push(audio_data);
        }
    }
}
