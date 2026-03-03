use hug_accumulator::AudioConsumer;
use hug_shared::{BlockTime, ChannelsInfo};

pub use super::PeakBucket;

pub struct WindowBufferAvg {
    buckets: Vec<PeakBucket>,

    samplerate: f64,
    sample_per_bucket: usize,
    n_buckets: usize,
    seconds: f64,

    write_idx: usize,
    gen_id: usize,
    intermediate: Vec<f32>,
}

impl WindowBufferAvg {
    pub fn new(samplerate: f64, n_buckets: usize, seconds: f64) -> Self {
        // number of total sample that would be displayed
        let buckets = vec![PeakBucket::empty(); n_buckets];

        let mut buffer = Self {
            buckets,
            n_buckets,
            samplerate,
            sample_per_bucket: 0,
            write_idx: 0,
            gen_id: 0,
            seconds,
            intermediate: Vec::new(),
        };
        buffer.set_seconds(seconds);
        buffer
    }

    pub fn set_seconds(&mut self, seconds: f64) {
        let sample_count = self.samplerate * seconds;
        self.sample_per_bucket = (sample_count / self.n_buckets as f64).ceil() as usize;
        self.seconds = seconds;
    }

    pub fn push_point(&mut self, y: f32) {
        let bucket = &mut self.buckets[self.write_idx];
        bucket.add_sample(y);
        if bucket.count() == self.sample_per_bucket {
            self.write_idx = (self.write_idx + 1) % self.n_buckets;
            self.buckets[self.write_idx] = PeakBucket::empty();
        }
    }

    pub fn iter_peaks(&self) -> impl Iterator<Item = f32> {
        let start = self.write_idx;
        (start..self.n_buckets)
            .chain(0..start)
            .map(|idx| self.buckets[idx].peak())
    }

    pub fn num_points(&self) -> usize {
        self.n_buckets
    }
}

impl AudioConsumer for WindowBufferAvg {
    fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
        self.gen_id = self.gen_id.wrapping_add(1);
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
}
