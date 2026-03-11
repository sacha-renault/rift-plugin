use core::f32;
use std::{collections::VecDeque, sync::Arc};

use super::AudioConsumer;
use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};
use rift_plugin_shared::utils::spaces::Linespace;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

fn hanning(fft_size: usize) -> Vec<f32> {
    Linespace::new(0.0, 2.0 * f32::consts::PI, fft_size)
        .map(|w| 0.5 * (1.0 - w.cos()))
        .collect()
}

pub struct DequeBuffer {
    inner: VecDeque<f32>,

    /// A cache were we can copy contigous version
    /// of [`VecDeque`] without allocating
    flat_cache: Vec<f32>,
    capacity: usize,
}

impl DequeBuffer {
    pub fn new(capacity: usize, cache_size: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            flat_cache: vec![0.0; cache_size],
            capacity,
        }
    }

    pub fn push_block_front(&mut self, block: &[f32]) {
        let expected_next_size = self.inner.len() + block.len();
        if expected_next_size > self.capacity {
            self.pop_n(expected_next_size - self.capacity);
        }
        for &sample in block {
            self.inner.push_back(sample);
        }
    }

    /// Updates the flat buffer and returns a contiguous slice
    #[inline]
    pub fn as_contiguous(&mut self) -> &[f32] {
        self.as_contiguous_latest(self.inner.len())
    }

    pub fn as_contiguous_latest(&mut self, n: usize) -> &[f32] {
        let (front, back) = self.inner.as_slices();
        let total_available = front.len() + back.len();

        // We only want the 'n' most recent samples.
        // In a VecDeque, 'back' contains the newest samples.
        let to_copy = n.min(total_available);
        let mut remaining = to_copy;
        let mut write_ptr = to_copy;

        // the newest data
        let back_len = back.len();
        let from_back = back_len.min(remaining);
        write_ptr -= from_back;
        self.flat_cache[write_ptr..write_ptr + from_back]
            .copy_from_slice(&back[back_len - from_back..]);
        remaining -= from_back;

        // If we still need more, take from the end of the 'front' slice
        if remaining > 0 {
            let front_len = front.len();
            let from_front = front_len.min(remaining);
            write_ptr -= from_front;
            self.flat_cache[write_ptr..write_ptr + from_front]
                .copy_from_slice(&front[front_len - from_front..]);
        }

        &self.flat_cache[..to_copy]
    }

    pub fn pop_n(&mut self, n: usize) {
        self.inner.drain(0..n.min(self.inner.len()));
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct StftChannelConsumer {
    channel_target: usize,

    samples: DequeBuffer,
    cache: Vec<f32>,
    window: Vec<f32>,

    fft_size: usize,
    samplerate: f32,

    fft: Arc<dyn Fft<f32>>,
    fft_workspace: Vec<Complex<f32>>,
}

impl StftChannelConsumer {
    pub fn new(channel: usize, fft_size: usize, samplerate: f32) -> Self {
        let mut fft_planner = FftPlanner::<f32>::new();
        let fft = fft_planner.plan_fft_forward(fft_size);
        let window = hanning(fft_size);

        Self {
            channel_target: channel,
            samples: DequeBuffer::new(fft_size, fft_size),
            cache: vec![0.0; fft_size / 2],
            fft,
            fft_size,
            window,
            samplerate,
            fft_workspace: vec![Complex { re: 0.0, im: 0.0 }; fft_size],
        }
    }

    pub fn consume_samples(&mut self, block: &[f32]) {
        self.samples.push_block_front(block);
        let half_fft_size = 0.5 * self.fft_size as f32;

        if self.samples.len() >= self.fft_size {
            let contiguous_samples = self.samples.as_contiguous_latest(self.fft_size);
            for (i, sample) in contiguous_samples.iter().enumerate().take(self.fft_size) {
                self.fft_workspace[i] = Complex {
                    re: sample * self.window[i],
                    im: 0.0,
                };
            }

            // Execute FFT
            self.fft.process(&mut self.fft_workspace);

            // Store magnitudes for the VST UI
            for i in 0..(self.fft_size / 2) {
                self.cache[i] = self.fft_workspace[i].norm() / half_fft_size;
            }
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.samplerate
    }

    pub fn bins(&self) -> &[f32] {
        &self.cache
    }

    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
}

impl AudioConsumer for StftChannelConsumer {
    fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
        if channels.current == self.channel_target {
            self.consume_samples(block);
        }
    }
}
