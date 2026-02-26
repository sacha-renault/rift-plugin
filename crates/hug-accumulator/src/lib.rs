use crossbeam_queue::ArrayQueue;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

pub struct AudioBlock<const N: usize> {
    raw: [f32; N],
    slice_length: usize,
}

impl<const N: usize> AudioBlock<N> {
    pub fn new(slice: &[f32]) -> Self {
        let slice_length = slice.len();
        assert!(slice_length <= N);

        let mut raw = [0.0; N];
        raw[..slice_length].copy_from_slice(slice);
        AudioBlock { raw, slice_length }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.raw[..self.slice_length]
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.slice_length
    }
}

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

pub trait AudioConsumer: Send + Sync + 'static {
    fn consume(&mut self, block: &[f32], channel_idx: usize, total_channels: usize);
}

pub struct AudioAccumulator<const N: usize> {
    channels: Vec<ChannelProducer<N>>,
    consumers: Mutex<Vec<Box<dyn AudioConsumer>>>,
    new_data: AtomicBool,
}

impl<const N: usize> AudioAccumulator<N> {
    pub fn new(count: usize, block_count: usize) -> Self {
        let mut channels = Vec::with_capacity(count);
        for _ in 0..count {
            channels.push(ChannelProducer::new(block_count));
        }

        Self {
            channels: channels,
            consumers: Mutex::new(Vec::new()),
            new_data: AtomicBool::new(false),
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

        self.new_data.store(true, Ordering::Relaxed);
    }

    fn clear(&self) {
        for drain_idx in 0..self.channels() {
            while self.channels[drain_idx].buf.pop().is_some() {}
        }
    }

    pub fn drop_listeners(&self) {
        let mut consumer_lock = self.consumers.lock().unwrap();
        consumer_lock.clear();
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

        let mut consumer_lock = self.consumers.lock().unwrap();

        if consumer_lock.is_empty() {
            return self.clear();
        }

        loop {
            // pop on block per channel
            for idx in 0..n_channels {
                let Some(block) = self.channels[idx].buf.pop() else {
                    self.clear();
                    return;
                };

                for consumer in consumer_lock.iter_mut() {
                    consumer.consume(block.as_slice(), idx, n_channels);
                }
            }
        }

        // guard lock will fall here, therefore
        // the first UI element that comes here drain all
        // data being written by audio thread.
    }
}
