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
    pub fn new(samplerate: f64, n_buckets: usize, seconds: f64) -> Self {
        // number of total sample that would be displayed
        let buckets = vec![PeakBucket::empty(); n_buckets];

        let mut buffer = Self {
            buckets,
            mode: WindowBufferMode::Averaged,
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
        self.recalculate_sample_per_bucket();
    }

    /// Updates the number of buckets (visual segments) then  recalculate the number of sample required per bucket.
    pub fn set_num_buckets(&mut self, num_buckets: usize) {
        self.n_buckets = num_buckets;
        self.recalculate_sample_per_bucket();
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

    /// Returns the total number of peaks (buckets) currently stored.
    pub fn num_points(&self) -> usize {
        self.n_buckets
    }

    // Private helper for calculate the number of sample per bucket based on samplerate and seconds
    fn recalculate_sample_per_bucket(&mut self) {
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

    // Private helper for averaging mode logic
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

    // Private helper for single-channel mode logic
    //
    // must be called only on the correct channel
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
