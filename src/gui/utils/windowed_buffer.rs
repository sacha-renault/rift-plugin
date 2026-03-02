use hug_accumulator::AudioConsumer;
use hug_shared::{BlockTime, ChannelsInfo};

pub use super::PeakBucket;

pub struct WindowedBuffer {
    buckets: Vec<PeakBucket>,

    samplerate: f64,
    sample_per_bucket: usize,
    n_buckets: usize,
    tempo: f64,
    beats: f64,

    write_idx: usize,
    gen_id: usize,
}

impl WindowedBuffer {
    pub fn new(samplerate: f64, n_buckets: usize, beats: f64, tempo: f64) -> Self {
        // number of total sample that would be displayed
        let buckets = vec![PeakBucket::empty(); n_buckets];

        let mut buffer = Self {
            buckets,
            n_buckets,
            samplerate,
            sample_per_bucket: 0,
            write_idx: 0,
            gen_id: 0,
            tempo,
            beats,
        };
        buffer.set_tempo(tempo);
        buffer
    }

    pub fn set_tempo(&mut self, tempo: f64) {
        self.tempo = tempo;
        self.set_beats(self.beats);
    }

    pub fn set_beats(&mut self, beats: f64) {
        let seconds_per_beat = 60.0 / self.tempo;
        let total_seconds = seconds_per_beat * self.beats;
        let sample_count = self.samplerate * total_seconds;
        self.sample_per_bucket = (sample_count / self.n_buckets as f64).ceil() as usize;
        self.beats = beats;
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

impl AudioConsumer for WindowedBuffer {
    fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
        self.gen_id = self.gen_id.wrapping_add(1);
        let ChannelsInfo { current, .. } = channels;

        // This is a POC, we rn just process channel 0
        if current != 0 {
            return;
        }

        for &v in block.iter() {
            self.push_point(v);
        }
    }
}
