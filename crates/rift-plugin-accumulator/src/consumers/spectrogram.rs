use core::f32;
use std::sync::Arc;

use super::MonoConsumer;
use rift_plugin_core::transport::BlockTime;
use rift_plugin_core::utils::dequeue_buffer::DequeBuffer;
use rift_plugin_core::utils::spaces::Linspace;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

/// Single-channel STFT consumer. Accumulates incoming samples into a rolling
/// buffer and recomputes the magnitude spectrum whenever a full `fft_size`
/// window is available. Only processes blocks tagged with `channel_target`;
/// all others are ignored.
pub struct StftConsumer {
    samples: DequeBuffer,
    cache: Vec<f32>,
    window: Vec<f32>,

    fft_size: usize,
    samplerate: f32,

    fft: Arc<dyn Fft<f32>>,
    fft_workspace: Vec<Complex<f32>>,
}

impl StftConsumer {
    /// Creates a consumer for `channel` using a forward FFT of size `fft_size`
    /// at the given `samplerate`. The FFT plan is computed once here.
    pub fn new(fft_size: usize, samplerate: f32) -> Self {
        let fft = FftPlanner::<f32>::new().plan_fft_forward(fft_size);
        let window = hanning(fft_size);

        Self {
            samples: DequeBuffer::new(fft_size),
            cache: vec![0.0; fft_size / 2],
            fft,
            fft_size,
            window,
            samplerate,
            fft_workspace: vec![Complex { re: 0.0, im: 0.0 }; fft_size],
        }
    }

    /// Pushes `block` into the rolling buffer. If the buffer now holds at
    /// least `fft_size` samples, runs the windowed FFT and updates [`Self::bins`].
    /// Does nothing until enough samples have accumulated.
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

    /// Normalized magnitude bins for the positive-frequency half of the spectrum.
    /// All zeros until the first full window has been processed.
    pub fn bins(&self) -> &[f32] {
        &self.cache
    }

    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
}

impl MonoConsumer for StftConsumer {
    fn consume(&mut self, block: &[f32], _: BlockTime) {
        self.consume_samples(block);
    }
}

fn hanning(fft_size: usize) -> Vec<f32> {
    Linspace::new(0.0, f32::consts::TAU, fft_size)
        .map(|w| 0.5 * (1.0 - w.cos()))
        .collect()
}

#[cfg(test)]
mod tests {
    use rift_plugin_core::assert_approx_eq;

    use super::*;
    use std::f32::consts::PI;

    fn make_consumer(fft_size: usize) -> StftConsumer {
        StftConsumer::new(fft_size, 44100.0)
    }

    fn feed_sine(consumer: &mut StftConsumer, freq_hz: f32) {
        let n = consumer.fft_size();
        let sr = consumer.sample_rate();
        let block: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * freq_hz * i as f32 / sr).sin())
            .collect();
        consumer.consume_samples(&block);
    }

    #[test]
    fn test_hanning_window() {
        // Length
        for &n in &[64_usize, 256, 1024] {
            assert_eq!(hanning(n).len(), n);
        }

        let w = hanning(1024);

        // Endpoints near zero
        assert_approx_eq!(w[0], 0.0, 1e-4);
        assert_approx_eq!(w[1023], 0.0, 1e-4);

        // Peak is 1
        let max = w.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert_approx_eq!(max, 1.0, 1e-4);
    }

    #[test]
    fn test_bins_start_at_zero() {
        let c = make_consumer(256);
        assert!(c.bins().iter().all(|&b| b == 0.0));
    }

    #[test]
    fn test_bins_len_is_half_fft_size() {
        let c = make_consumer(512);
        assert_eq!(c.bins().len(), 256);
    }

    #[test]
    fn test_bins_unchanged_before_full_block() {
        let mut c = make_consumer(256);

        // only half a block
        c.consume_samples(&vec![1.0_f32; 128]);
        assert!(c.bins().iter().all(|&b| b == 0.0));
    }

    #[test]
    fn dc_signal_energy_in_bin_zero() {
        let fft_size = 1024;
        let mut c = make_consumer(fft_size);
        c.consume_samples(&vec![1.0_f32; fft_size]);

        let bins = c.bins();
        let dc = bins[0];

        // Hanning window leaks some bin 0 into bin 1
        assert!(
            dc > bins[1],
            "DC bin should dominate: dc={dc}, max_ac={} (idx=1)",
            bins[1]
        );

        for (idx, bin) in bins[2..].iter().cloned().enumerate() {
            // Other should be MUCH smaller
            assert!(
                dc > bin * 10.,
                "DC bin should dominate: dc={dc}, max_ac={bin} (idx={})",
                idx + 2
            );
        }
    }

    #[test]
    fn test_sine_energy_at_correct_bin() {
        let fft_size = 4096;
        let sr = 44100.0_f32;
        let freq = 1000.0_f32;

        let mut c = StftConsumer::new(fft_size, sr);
        feed_sine(&mut c, freq);

        let bins = c.bins();
        let bin_hz = sr / fft_size as f32;
        let expected_bin = (freq / bin_hz).round() as usize;

        // Find the peak bin (ignore DC)
        let peak_bin = bins[1..]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i + 1)
            .unwrap();

        assert_eq!(
            peak_bin,
            expected_bin,
            "peak at bin {peak_bin} ({} Hz), expected {expected_bin} ({freq} Hz)",
            peak_bin as f32 * bin_hz
        );
    }

    #[test]
    fn test_consume_processes_correct_channel() {
        let mut c = make_consumer(256);
        let block = vec![1.0_f32; 256];

        c.consume(&block, BlockTime::none());
        // DC bin must be non-zero after a full block of 1.0
        assert!(c.bins()[0] > 0.0);
    }

    #[test]
    fn test_magnitudes_are_non_negative() {
        let fft_size = 512;
        let mut c = make_consumer(fft_size);
        feed_sine(&mut c, 440.0);
        assert!(c.bins().iter().all(|&b| b >= 0.0));
    }

    #[test]
    fn test_second_block_updates_bins() {
        let fft_size = 512;
        let mut c = make_consumer(fft_size);

        feed_sine(&mut c, 440.0);
        let first: Vec<f32> = c.bins().to_vec();

        feed_sine(&mut c, 2000.0);
        let second: Vec<f32> = c.bins().to_vec();

        assert_ne!(first, second, "bins should change after new input");
    }
}
