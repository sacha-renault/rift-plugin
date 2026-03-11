use core::f32;
use std::sync::Arc;

use super::AudioConsumer;
use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};
use rift_plugin_shared::utils::dequeue_buffer::DequeBuffer;
use rift_plugin_shared::utils::spaces::Linspace;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

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
            samples: DequeBuffer::new(fft_size),
            cache: vec![0.0; fft_size / 2],
            fft,
            fft_size,
            window,
            samplerate,
            fft_workspace: vec![Complex { re: 0.0, im: 0.0 }; fft_size],
        }
    }

    pub fn consume_samples(&mut self, block: &[f32]) {
        self.samples.push_block(block);
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

fn hanning(fft_size: usize) -> Vec<f32> {
    Linspace::new(0.0, f32::consts::TAU, fft_size)
        .map(|w| 0.5 * (1.0 - w.cos()))
        .collect()
}
