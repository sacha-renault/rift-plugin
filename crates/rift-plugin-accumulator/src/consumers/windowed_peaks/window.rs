use rift_plugin_core::transport::{BlockTime, ChannelsInfo};

pub use super::bucket::Bucket;
use crate::prelude::AudioConsumer;

/// A circular buffer that accumulates audio samples over a time window and exposes [`Bucket::value`].
///
/// This struct is designed for visualizing audio waveforms (e.g., in a VU meter or oscilloscope)
/// where maintaining full sample precision isn't required, but tracking peaks over time is essential.
/// It implements [`AudioConsumer`] to accept audio blocks from the accumulator chain.
///
/// Channel routing (averaging, single-channel capture) is handled upstream by
/// [`ConsumerDispatcher`]; this struct only processes the mono block it receives.
pub struct WindowBuckets<B: Bucket> {
    /// The circular list of buckets storing peak data for each visual segment.
    buckets: Vec<B>,

    samplerate: f64,
    sample_per_bucket: usize,
    n_buckets: usize,
    seconds: f64,

    /// The index in the buckets array where the next sample should be written.
    write_idx: usize,
}

impl<B: Bucket> WindowBuckets<B> {
    pub fn new(samplerate: f64, seconds: f64) -> Self {
        let n_buckets = 1;
        let buckets = vec![B::empty(); n_buckets];

        let mut buffer = Self {
            buckets,
            n_buckets,
            samplerate,
            sample_per_bucket: 0,
            write_idx: 0,
            seconds,
        };
        buffer.set_seconds(seconds);
        buffer
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
        num_buckets = num_buckets.max(1);
        if num_buckets != self.n_buckets {
            self.n_buckets = num_buckets;
            self.recalculate_buckets();
            self.buckets = vec![B::empty(); self.n_buckets];
            self.write_idx = 0;
        }
    }

    /// Returns an iterator over the values of all buckets.
    ///
    /// The iterator starts from the `write_idx` and wraps around to show the full history.
    pub fn iter_values(&self) -> impl Iterator<Item = f32> {
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
            self.buckets[self.write_idx] = B::empty();
        }
    }
}

impl<B: Bucket> AudioConsumer for WindowBuckets<B> {
    fn consume(&mut self, block: &[f32], _: ChannelsInfo, _: BlockTime) {
        for &value in block.iter() {
            self.push_point(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_core::transport::{BlockTime, ChannelsInfo};

    use crate::consumers::windowed_peaks::peaks::PeakBucket;

    use super::*;

    fn make_buffer(n_buckets: usize, seconds: f64) -> WindowBuckets<PeakBucket> {
        let mut buf = WindowBuckets::new(44100.0, seconds);
        buf.set_num_buckets(n_buckets);
        buf
    }

    fn dummy_channels() -> ChannelsInfo {
        ChannelsInfo {
            current: 0,
            total_channels: 1,
        }
    }

    fn feed_block(buffer: &mut WindowBuckets<PeakBucket>, block: &[f32]) {
        buffer.consume(block, dummy_channels(), BlockTime::none());
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
    fn test_produces_nonzero_peaks() {
        let mut b = make_buffer(64, 1.0);
        let block = vec![1.0_f32; 512];
        feed_block(&mut b, &block);
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

        feed_block(&mut b, &vec![1.0_f32; 2048]);
        let first: Vec<f32> = b.iter_values().collect();

        feed_block(&mut b, &vec![0.0_f32; 44100]);
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
