use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};

pub use super::bucket::Bucket;
use crate::AudioConsumer;

/// Represents the operating mode of a [`WindowBuckets`].
pub enum WindowBucketsMode {
    /// Average all channels into a single one
    Averaged,

    /// Capture a single channel (start at idx 0)
    Channel(usize),
}

/// A circular buffer that accumulates audio samples over a time window and exposes [`Bucket::value`].
///
/// This struct is designed for visualizing audio waveforms (e.g., in a VU meter or oscilloscope)
/// where maintaining full sample precision isn't required, but tracking peaks over time is essential.
/// It implements [`AudioConsumer`] to accept audio blocks from the accumulator chain.
pub struct WindowBuckets<B: Bucket> {
    /// The circular list of buckets storing peak data for each visual segment.
    buckets: Vec<B>,

    /// The current operating mode (Averaged all channels vs. specific channel capture).
    mode: WindowBucketsMode,

    samplerate: f64,
    sample_per_bucket: usize,
    n_buckets: usize,
    seconds: f64,

    /// The index in the buckets array where the next sample should be written.
    write_idx: usize,

    /// Temporary buffer for accumulating samples when operating in [`WindowBucketsMode::Averaged`] mode.
    intermediate: Vec<f32>,
}

impl<B: Bucket> WindowBuckets<B> {
    pub fn new(samplerate: f64, seconds: f64) -> Self {
        // number of total sample that would be displayed
        let n_buckets = 1;
        let buckets = vec![B::empty(); n_buckets];

        let mut buffer = Self {
            buckets,
            mode: WindowBucketsMode::Averaged,
            n_buckets,
            samplerate,
            sample_per_bucket: 0,
            write_idx: 0,
            seconds,
            intermediate: Vec::new(),
        };
        buffer.set_seconds(seconds);
        buffer
    }

    pub fn set_mode(&mut self, mode: WindowBucketsMode) {
        self.mode = mode;

        // if it's channel specific, we can deallocate
        // entirely the intermediate buffer. We don't use
        // it to sum up over channels
        if matches!(self.mode, WindowBucketsMode::Channel(_)) {
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

    /// Updates the number of buckets (visual segments) then recalculate the number of sample required
    /// per bucket and resize `buckets`.
    ///
    /// # Note:
    /// - This function will allocate.
    /// - if `num_buckets` is 0, it will be clamped to the minimal value: 1.
    pub fn set_num_buckets(&mut self, mut num_buckets: usize) {
        // Number of bucket should never be 0. Otherwise, we might crash later
        // So even if request of resizing at 0, we don't.
        num_buckets = num_buckets.min(1);
        if num_buckets != self.n_buckets {
            self.n_buckets = num_buckets;
            self.recalculate_buckets();
            self.buckets = vec![B::empty(); self.n_buckets];
        }
    }

    /// Returns an iterator over the values of all buckets.
    ///
    /// The iterator starts from the `write_idx` and wraps around to show the full history.
    pub fn iter_values(&self) -> impl Iterator<Item = f32> {
        // FIX: write_idx is the one we currently write on e.g. the most recent
        // it has to be at the very end so start must be +1
        let start = (self.write_idx + 1).rem_euclid(self.n_buckets);
        (start..self.n_buckets)
            .chain(0..start)
            .map(|idx| self.buckets[idx].value())
    }

    /// Get value at `idx`.
    ///
    /// 0 means oldest while len() - 1 is most recent
    pub fn get_value(&self, idx: usize) -> Option<f32> {
        if idx >= self.n_buckets {
            None
        } else {
            Some(self.buckets[(self.write_idx + 1 + idx).rem_euclid(self.n_buckets)].value())
        }
    }

    /// Returns the total number of [`Bucket`]s currently stored.
    pub fn num_points(&self) -> usize {
        self.n_buckets
    }

    // Private helper for calculate the number of sample per bucket based on samplerate and seconds
    fn recalculate_buckets(&mut self) {
        let sample_count = self.samplerate * self.seconds;
        self.sample_per_bucket = (sample_count / self.n_buckets as f64).ceil() as usize;
    }

    #[inline]
    fn push_point(&mut self, y: f32) {
        let bucket = &mut self.buckets[self.write_idx];
        bucket.add_sample(y);
        if bucket.count() == self.sample_per_bucket {
            self.write_idx = (self.write_idx + 1) % self.n_buckets;

            // When we fill current bucket, we need to "remove" the next one
            // because the data becomes outdated and will be use for nexts writes
            self.buckets[self.write_idx] = B::empty();
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

impl<B: Bucket> AudioConsumer for WindowBuckets<B> {
    fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
        match self.mode {
            WindowBucketsMode::Averaged => self.consume_avg(block, channels),
            WindowBucketsMode::Channel(captured) if channels.current == captured => {
                self.consume_channel(block)
            }
            WindowBucketsMode::Channel(_) => {} // Might not be the right channel
        }
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_shared::assert_approx_eq;
    use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};

    use crate::consumers::windowed_peaks::peaks::PeakBucket;

    use super::*;

    fn make_buffer(n_buckets: usize, seconds: f64) -> WindowBuckets<PeakBucket> {
        let mut buf = WindowBuckets::new(44100.0, seconds);
        buf.set_num_buckets(n_buckets);
        buf
    }

    fn make_channels(current: usize, total: usize) -> ChannelsInfo {
        ChannelsInfo {
            current,
            total_channels: total,
        }
    }

    fn feed_block(buffer: &mut WindowBuckets<PeakBucket>, block: &[f32], total_channels: usize) {
        for ch in 0..total_channels {
            buffer.consume(block, make_channels(ch, total_channels), BlockTime::none());
        }
    }

    #[test]
    fn test_initial_peaks_are_zero() {
        let b = make_buffer(64, 1.0);
        assert!(b.iter_values().all(|p| p == 0.0));
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
        assert!(b.iter_values().any(|p| p > 0.0));
    }

    #[test]
    fn test_averaged_mode_averages_channels() {
        let mut b = make_buffer(64, 1.0);

        // ch0 = 1.0, ch1 = 0.0 → average should be 0.5
        let ones = vec![1.0_f32; 512];
        let zeros = vec![0.0_f32; 512];
        b.consume(&ones, make_channels(0, 2), BlockTime::none());
        b.consume(&zeros, make_channels(1, 2), BlockTime::none());

        let max_peak = b.iter_values().fold(0.0_f32, f32::max);
        assert_approx_eq!(max_peak, 0.5, 1e-4);
    }

    #[test]
    fn test_averaged_ignores_incomplete_channel_pass() {
        // Only feeding channel 0 out of 2 — intermediate should not be pushed yet
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 512];
        b.consume(&block, make_channels(0, 2), BlockTime::none());

        // Peaks should still be zero since channel 1 hasn't come in
        assert!(b.iter_values().all(|p| p == 0.0));
    }

    #[test]
    fn test_channel_mode_captures_correct_channel() {
        let mut b = make_buffer(64, 1.0);
        b.set_mode(WindowBucketsMode::Channel(1));

        let silence = vec![0.0_f32; 512];
        let signal = vec![1.0_f32; 512];

        // ch0 = silence, ch1 = signal
        b.consume(&silence, make_channels(0, 2), BlockTime::none());
        b.consume(&signal, make_channels(1, 2), BlockTime::none());

        assert!(b.iter_values().any(|p| p > 0.0));
    }

    #[test]
    fn test_channel_mode_ignores_wrong_channel() {
        let mut b = make_buffer(64, 1.0);
        b.set_mode(WindowBucketsMode::Channel(1));

        let signal = vec![1.0_f32; 512];
        b.consume(&signal, make_channels(0, 2), BlockTime::none());

        assert!(b.iter_values().all(|p| p == 0.0));
    }

    #[test]
    fn test_set_mode_to_channel_deallocates_intermediate() {
        let mut b = make_buffer(64, 1.0);

        // Trigger intermediate allocation via averaged mode
        let block = vec![1.0_f32; 512];
        feed_block(&mut b, &block, 2);

        b.set_mode(WindowBucketsMode::Channel(0));

        // Channel mode should work fine after switching
        b.consume(&block, make_channels(0, 1), BlockTime::none());
        assert!(b.iter_values().any(|p| p > 0.0));
    }

    #[test]
    fn test_iter_peaks_length_is_always_n_buckets() {
        let b = make_buffer(32, 1.0);
        assert_eq!(b.iter_values().count(), 32);
    }

    #[test]
    fn test_peaks_update_after_new_data() {
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 2048];

        feed_block(&mut b, &block, 1);
        let first: Vec<f32> = b.iter_values().collect();

        let silence = vec![0.0_f32; 44100];
        feed_block(&mut b, &silence, 1);
        let second: Vec<f32> = b.iter_values().collect();

        assert_ne!(first, second, "peaks should change after new data");
    }

    #[test]
    fn test_set_num_buckets_updates_num_points() {
        let mut b = make_buffer(64, 1.0);
        b.set_num_buckets(128);
        assert_eq!(b.num_points(), 128);
    }
}
