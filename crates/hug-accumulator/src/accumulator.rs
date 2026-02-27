use crossbeam_queue::ArrayQueue;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use parking_lot::Mutex;

use crate::audio_block::AudioBlock;
use crate::consumer::AudioConsumer;

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
    consumers: Mutex<Vec<Arc<Mutex<dyn AudioConsumer>>>>,
    new_data: AtomicBool,
}

impl<const N: usize> AudioAccumulator<N> {
    pub fn new(count: usize, block_count: usize) -> Self {
        let mut channels = Vec::with_capacity(count);
        for _ in 0..count {
            channels.push(ChannelProducer::new(block_count));
        }

        Self {
            channels,
            consumers: Mutex::new(Vec::new()),
            new_data: AtomicBool::new(false),
        }
    }

    pub fn add_consumer<C: AudioConsumer>(&self, consumer: Arc<Mutex<C>>) {
        self.consumers.lock().push(consumer);
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

        self.new_data.store(true, Ordering::Relaxed);
    }

    fn clear(&self) {
        for drain_idx in 0..self.channels() {
            while self.channels[drain_idx].buf.pop().is_some() {}
        }
    }

    /// This function is meant to be called on the UI thread
    /// locks are fine here
    pub fn drain(&self) {
        if !self.new_data.swap(false, Ordering::Relaxed) {
            return;
        }

        let n_channels = self.channels();

        if n_channels == 0 {
            return;
        }

        let mut consumer_lock = self.consumers.lock();

        if consumer_lock.is_empty() {
            return self.clear();
        }

        loop {
            // pop one block per channel
            for idx in 0..n_channels {
                let Some(block) = self.channels[idx].buf.pop() else {
                    self.clear();
                    return;
                };

                for consumer in consumer_lock.iter_mut() {
                    consumer.lock().consume(block.as_slice(), idx, n_channels);
                }
            }
        }

        // guard lock will fall here, therefore
        // the first UI element that comes here drains all
        // data being written by audio thread.
    }
}
