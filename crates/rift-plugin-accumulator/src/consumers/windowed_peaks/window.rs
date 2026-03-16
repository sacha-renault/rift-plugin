use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};

pub use super::peaks::PeakBucket;
use crate::AudioConsumer;

/// Represents the operating mode of a [`WindowBuffer`].
pub enum WindowBufferMode {
    /// Average all channels into a single one
    Averaged,

    /// Capture a single channel (start at idx 0)
    Channel(usize),
}

/// A circular buffer that accumulates audio samples over a time window and exposes peak values.
///
/// This struct is designed for visualizing audio waveforms (e.g., in a VU meter or oscilloscope)
/// where maintaining full sample precision isn't required, but tracking peaks over time is essential.
/// It implements [`AudioConsumer`] to accept audio blocks from the accumulator chain.
pub struct WindowBuffer {
    /// The circular list of buckets storing peak data for each visual segment.
    buckets: Vec<PeakBucket>,

    /// The current operating mode (Averaged all channels vs. specific channel capture).
    mode: WindowBufferMode,

    samplerate: f64,
    sample_per_bucket: usize,
    n_buckets: usize,
    seconds: f64,

    /// The index in the buckets array where the next sample should be written.
    write_idx: usize,

    /// Temporary buffer for accumulating samples when operating in [`WindowBufferMode::Averaged`] mode.
    intermediate: Vec<f32>,
}

impl WindowBuffer {
    pub fn new(samplerate: f64, seconds: f64) -> Self {
        // number of total sample that would be displayed
        let buckets = vec![PeakBucket::empty(); 10];

        let mut buffer = Self {
            buckets,
            mode: WindowBufferMode::Averaged,
            n_buckets: 10,
            samplerate,
            sample_per_bucket: 0,
            write_idx: 0,
            seconds,
            intermediate: Vec::new(),
        };
        buffer.set_seconds(seconds);
        buffer
    }

    pub fn set_mode(&mut self, mode: WindowBufferMode) {
        self.mode = mode;

        // if it's channel specific, we can deallocate
        // entirely the intermediate buffer. We don't use
        // it to sum up over channels
        if matches!(self.mode, WindowBufferMode::Channel(_)) {
            self.intermediate = Vec::new();
        }
    }

    /// Sets the duration of audio history the buffer represents.
    ///
    /// Adjusting seconds automatically recalculates `sample_per_bucket` based on the current `n_buckets`.
    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = seconds;
        self.recalculate_buckets();
    }

    /// Updates the number of buckets (visual segments) then  recalculate the number of sample required per bucket.
    pub fn set_num_buckets(&mut self, num_buckets: usize) {
        if num_buckets != self.n_buckets {
            self.n_buckets = num_buckets;
            self.recalculate_buckets();
            self.buckets = vec![PeakBucket::empty(); self.n_buckets];
        }
    }

    /// Returns an iterator over the peak values of all buckets.
    ///
    /// The iterator starts from the `write_idx` and wraps around to show the full history.
    pub fn iter_peaks(&self) -> impl Iterator<Item = f32> {
        // FIX: write_idx is the one we currently write on e.g. the most recent
        // it has to be at the very end so start must be +1
        let start = (self.write_idx + 1).rem_euclid(self.n_buckets);
        (start..self.n_buckets)
            .chain(0..start)
            .map(|idx| self.buckets[idx].peak())
    }

    /// Get peak at `idx`.
    ///
    /// 0 means oldest while len() - 1 is most recent
    pub fn get_peak(&self, idx: usize) -> Option<f32> {
        if idx >= self.n_buckets {
            None
        } else {
            Some(self.buckets[(self.write_idx + 1 + idx).rem_euclid(self.n_buckets)].peak())
        }
    }

    pub fn aligned_start(&self, num_points: usize) -> usize {
        let group_size = self.n_buckets / num_points;
        let oldest = self.write_idx + 1;
        let offset = oldest % group_size;
        if offset == 0 { 0 } else { group_size - offset }
    }

    /// Returns the total number of peaks (buckets) currently stored.
    pub fn num_points(&self) -> usize {
        self.n_buckets
    }

    // Private helper for calculate the number of sample per bucket based on samplerate and seconds
    fn recalculate_buckets(&mut self) {
        let sample_count = self.samplerate * self.seconds;
        self.sample_per_bucket = (sample_count / self.n_buckets as f64).ceil() as usize;
    }

    fn push_point(&mut self, y: f32) {
        let bucket = &mut self.buckets[self.write_idx];
        bucket.add_sample(y);
        if bucket.count() == self.sample_per_bucket {
            self.write_idx = (self.write_idx + 1) % self.n_buckets;

            // When we fill current bucket, we need to "remove" the next one
            // because the data becomes outdated and will be use for nexts writes
            self.buckets[self.write_idx] = PeakBucket::empty();
        }
    }

    /// Private helper for averaging mode logic
    fn consume_avg(&mut self, block: &[f32], channels: ChannelsInfo) {
        let total_channel = channels.total_channels as f32;

        // At channel 0, we ensure our intermediate buffer is large enough
        if channels.current == 0 {
            self.intermediate.clear();
            self.intermediate.resize(block.len(), 0.);
        }

        for (s, &v) in self.intermediate.iter_mut().zip(block.iter()) {
            *s += v / total_channel;
        }

        if channels.is_last_channel() {
            for i in 0..self.intermediate.len() {
                let v = self.intermediate[i];
                self.push_point(v);
            }
        }
    }

    /// Private helper for single-channel mode logic
    ///
    /// must be called only on the correct channel
    fn consume_channel(&mut self, block: &[f32]) {
        for &value in block.iter() {
            self.push_point(value);
        }
    }
}

impl AudioConsumer for WindowBuffer {
    fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
        match self.mode {
            WindowBufferMode::Averaged => self.consume_avg(block, channels),
            WindowBufferMode::Channel(captured) if channels.current == captured => {
                self.consume_channel(block)
            }
            WindowBufferMode::Channel(_) => {} // Might not be the right channel
        }
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_shared::assert_approx_eq;
    use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};

    use super::*;

    fn make_buffer(n_buckets: usize, seconds: f64) -> WindowBuffer {
        let mut buf = WindowBuffer::new(44100.0, seconds);
        buf.set_num_buckets(n_buckets);
        buf
    }

    fn make_channels(current: usize, total: usize) -> ChannelsInfo {
        ChannelsInfo {
            current,
            total_channels: total,
        }
    }

    fn feed_block(buffer: &mut WindowBuffer, block: &[f32], total_channels: usize) {
        for ch in 0..total_channels {
            buffer.consume(block, make_channels(ch, total_channels), BlockTime::none());
        }
    }

    #[test]
    fn test_initial_peaks_are_zero() {
        let b = make_buffer(64, 1.0);
        assert!(b.iter_peaks().all(|p| p == 0.0));
    }

    #[test]
    fn test_num_points_matches_n_buckets() {
        for &n in &[16_usize, 64, 256] {
            assert_eq!(make_buffer(n, 1.0).num_points(), n);
        }
    }

    #[test]
    fn test_averaged_mode_produces_nonzero_peaks() {
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 512];
        feed_block(&mut b, &block, 2);
        assert!(b.iter_peaks().any(|p| p > 0.0));
    }

    #[test]
    fn test_averaged_mode_averages_channels() {
        let mut b = make_buffer(64, 1.0);

        // ch0 = 1.0, ch1 = 0.0 → average should be 0.5
        let ones = vec![1.0_f32; 512];
        let zeros = vec![0.0_f32; 512];
        b.consume(&ones, make_channels(0, 2), BlockTime::none());
        b.consume(&zeros, make_channels(1, 2), BlockTime::none());

        let max_peak = b.iter_peaks().fold(0.0_f32, f32::max);
        assert_approx_eq!(max_peak, 0.5, 1e-4);
    }

    #[test]
    fn test_averaged_ignores_incomplete_channel_pass() {
        // Only feeding channel 0 out of 2 — intermediate should not be pushed yet
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 512];
        b.consume(&block, make_channels(0, 2), BlockTime::none());

        // Peaks should still be zero since channel 1 hasn't come in
        assert!(b.iter_peaks().all(|p| p == 0.0));
    }

    #[test]
    fn test_channel_mode_captures_correct_channel() {
        let mut b = make_buffer(64, 1.0);
        b.set_mode(WindowBufferMode::Channel(1));

        let silence = vec![0.0_f32; 512];
        let signal = vec![1.0_f32; 512];

        // ch0 = silence, ch1 = signal
        b.consume(&silence, make_channels(0, 2), BlockTime::none());
        b.consume(&signal, make_channels(1, 2), BlockTime::none());

        assert!(b.iter_peaks().any(|p| p > 0.0));
    }

    #[test]
    fn test_channel_mode_ignores_wrong_channel() {
        let mut b = make_buffer(64, 1.0);
        b.set_mode(WindowBufferMode::Channel(1));

        let signal = vec![1.0_f32; 512];
        b.consume(&signal, make_channels(0, 2), BlockTime::none());

        assert!(b.iter_peaks().all(|p| p == 0.0));
    }

    #[test]
    fn test_set_mode_to_channel_deallocates_intermediate() {
        let mut b = make_buffer(64, 1.0);

        // Trigger intermediate allocation via averaged mode
        let block = vec![1.0_f32; 512];
        feed_block(&mut b, &block, 2);

        b.set_mode(WindowBufferMode::Channel(0));

        // Channel mode should work fine after switching
        b.consume(&block, make_channels(0, 1), BlockTime::none());
        assert!(b.iter_peaks().any(|p| p > 0.0));
    }

    #[test]
    fn test_iter_peaks_length_is_always_n_buckets() {
        let b = make_buffer(32, 1.0);
        assert_eq!(b.iter_peaks().count(), 32);
    }

    #[test]
    fn test_peaks_update_after_new_data() {
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 2048];

        feed_block(&mut b, &block, 1);
        let first: Vec<f32> = b.iter_peaks().collect();

        let silence = vec![0.0_f32; 44100];
        feed_block(&mut b, &silence, 1);
        let second: Vec<f32> = b.iter_peaks().collect();

        assert_ne!(first, second, "peaks should change after new data");
    }

    #[test]
    fn test_set_num_buckets_updates_num_points() {
        let mut b = make_buffer(64, 1.0);
        b.set_num_buckets(128);
        assert_eq!(b.num_points(), 128);
    }
}
