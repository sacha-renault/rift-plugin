use rift_plugin_types::transport::BlockTime;
use rift_plugin_utils::conversion::linear_to_db;

use crate::prelude::MonoConsumer;

const FALLBACK_SAMPLE_RATE: f64 = 48_000.0;
const SILENCE: f64 = 1e-8;

/// Peak meter for a single channel.
pub struct AudioPeak {
    // f64: the per-sample release coefficient is within ~1e-5 of 1.0, which
    // f32 can't represent precisely enough to keep the release rate accurate.
    level: f64,
    held: f64,
    clipped: bool,

    release_db_per_sec: f32,
    hold_ms: f32,
    clip_threshold: f32,

    /// Derived from the settings above and `sample_rate`.
    sample_rate: f64,
    release_coef: f64,
    hold_samples: u64,
    hold_left: u64,

    /// Expected `sample_pos` of the next block, to detect dropped samples.
    next_pos: Option<u64>,
}

impl Default for AudioPeak {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPeak {
    /// Creates a meter with a 20 dB/s release, a 1 s hold and a clip
    /// threshold of 0 dBFS.
    pub fn new() -> Self {
        let mut meter = Self {
            level: 0.0,
            held: 0.0,
            clipped: false,
            release_db_per_sec: 20.0,
            hold_ms: 1000.0,
            clip_threshold: 1.0,
            sample_rate: FALLBACK_SAMPLE_RATE,
            release_coef: 1.0,
            hold_samples: 0,
            hold_left: 0,
            next_pos: None,
        };
        meter.update_coefs();
        meter
    }

    /// How fast `level` and `held` fall, in dB per second. Clamped to >= 0.
    pub fn release_db_per_sec(mut self, db_per_sec: f32) -> Self {
        self.release_db_per_sec = db_per_sec.max(0.0);
        self.update_coefs();
        self
    }

    /// How long `held` stays at a peak before falling, in milliseconds.
    pub fn hold_ms(mut self, ms: f32) -> Self {
        self.hold_ms = ms.max(0.0);
        self.update_coefs();
        self
    }

    /// Linear amplitude at or above which [`Self::clipped`] is set.
    /// Defaults to `1.0` (0 dBFS).
    pub fn clip_threshold(mut self, threshold: f32) -> Self {
        self.clip_threshold = threshold;
        self
    }

    /// Current level, linear amplitude.
    pub fn level(&self) -> f32 {
        self.level as f32
    }

    /// Current level in dBFS, `-inf` for silence.
    pub fn level_db(&self) -> f32 {
        linear_to_db(self.level())
    }

    /// Held peak, linear amplitude. Always >= [`Self::level`].
    pub fn held(&self) -> f32 {
        self.held as f32
    }

    /// Held peak in dBFS, `-inf` for silence.
    pub fn held_db(&self) -> f32 {
        linear_to_db(self.held())
    }

    /// Whether a sample reached the clip threshold since the last
    /// [`Self::reset_clip`].
    pub fn clipped(&self) -> bool {
        self.clipped
    }

    pub fn reset_clip(&mut self) {
        self.clipped = false;
    }

    /// Resets the meter to silence and clears the clip flag.
    pub fn reset(&mut self) {
        self.level = 0.0;
        self.held = 0.0;
        self.hold_left = 0;
        self.clipped = false;
    }

    fn update_coefs(&mut self) {
        // Falling by `r` dB per second is multiplying by 10^(-r / 20) per second.
        let db_per_sample = self.release_db_per_sec as f64 / self.sample_rate;
        self.release_coef = 10f64.powf(-db_per_sample / 20.0);
        self.hold_samples = (self.hold_ms as f64 * 1e-3 * self.sample_rate).round() as u64;
        self.hold_left = self.hold_left.min(self.hold_samples);
    }

    /// Advances `n` samples of silence.
    fn skip(&mut self, n: u64) {
        let held_frozen = n.min(self.hold_left);
        self.hold_left -= held_frozen;

        let decay = |exp: u64| self.release_coef.powf(exp as f64);
        self.level *= decay(n);
        self.held = (self.held * decay(n - held_frozen)).max(self.level);
    }

    #[inline]
    fn push(&mut self, sample: f32) {
        let a = sample.abs() as f64;

        self.level = (self.level * self.release_coef).max(a);

        if a >= self.held {
            self.held = a;
            self.hold_left = self.hold_samples;
        } else if self.hold_left > 0 {
            self.hold_left -= 1;
        } else {
            self.held = (self.held * self.release_coef).max(self.level);
        }

        if a >= self.clip_threshold as f64 {
            self.clipped = true;
        }
    }
}

impl MonoConsumer for AudioPeak {
    fn consume(&mut self, block: &[f32], time: BlockTime) {
        if time.sample_rate > 0.0 && time.sample_rate != self.sample_rate {
            self.sample_rate = time.sample_rate;
            self.update_coefs();
        }

        if let Some(expected) = self.next_pos
            && time.sample_pos > expected
        {
            self.skip(time.sample_pos - expected);
        }
        self.next_pos = Some(time.sample_pos + block.len() as u64);

        for &sample in block {
            self.push(sample);
        }

        if self.level < SILENCE {
            self.level = 0.0;
        }
        if self.held < SILENCE {
            self.held = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_utils::assert_approx_eq;

    use super::*;

    const SR: f64 = 48_000.0;

    fn at(sample_pos: u64) -> BlockTime {
        BlockTime {
            sample_pos,
            sample_rate: SR,
            transport: None,
        }
    }

    /// Feeds `len` samples of `value` as one block, continuing from `pos`.
    fn feed(meter: &mut AudioPeak, pos: &mut u64, value: f32, len: usize) {
        meter.consume(&vec![value; len], at(*pos));
        *pos += len as u64;
    }

    #[test]
    fn starts_silent() {
        let m = AudioPeak::new();
        assert_eq!(m.level(), 0.0);
        assert_eq!(m.held(), 0.0);
        assert_eq!(m.level_db(), f32::NEG_INFINITY);
        assert!(!m.clipped());
    }

    #[test]
    fn instant_attack() {
        let mut m = AudioPeak::new();
        m.consume(&[0.0, -0.5, 0.25], at(0));
        assert_eq!(m.held(), 0.5);
        assert!(m.level() >= 0.49);
    }

    #[test]
    fn releases_at_db_per_sec() {
        let mut m = AudioPeak::new().release_db_per_sec(20.0);
        let mut pos = 0;

        feed(&mut m, &mut pos, 1.0, 1);
        feed(&mut m, &mut pos, 0.0, SR as usize);

        // 1 s at 20 dB/s from 0 dBFS
        assert_approx_eq!(m.level_db(), -20.0, 0.01);
    }

    #[test]
    fn release_is_block_size_independent() {
        let mut a = AudioPeak::new();
        let mut b = AudioPeak::new();
        let (mut pa, mut pb) = (0, 0);

        feed(&mut a, &mut pa, 1.0, 1);
        feed(&mut b, &mut pb, 1.0, 1);

        feed(&mut a, &mut pa, 0.0, 4800);
        for _ in 0..75 {
            feed(&mut b, &mut pb, 0.0, 64);
        }

        assert_approx_eq!(a.level(), b.level(), 1e-5);
        assert_approx_eq!(a.held(), b.held(), 1e-5);
    }

    #[test]
    fn release_is_sample_rate_independent() {
        let mut a = AudioPeak::new();
        let mut b = AudioPeak::new();

        let time = |sample_pos, sample_rate| BlockTime {
            sample_pos,
            sample_rate,
            transport: None,
        };

        a.consume(&[1.0], time(0, 48_000.));
        a.consume(&vec![0.0; 24_000], time(1, 48_000.));

        b.consume(&[1.0], time(0, 96_000.));
        b.consume(&vec![0.0; 48_000], time(1, 96_000.));

        assert_approx_eq!(a.level_db(), b.level_db(), 0.01);
    }

    #[test]
    fn held_stays_during_hold_then_falls() {
        let mut m = AudioPeak::new().hold_ms(100.0).release_db_per_sec(20.0);
        let mut pos = 0;

        feed(&mut m, &mut pos, 0.8, 1);
        feed(&mut m, &mut pos, 0.0, 4000); // < 100 ms
        assert_eq!(m.held(), 0.8);
        assert!(m.level() < 0.8);

        feed(&mut m, &mut pos, 0.0, 24_000); // way past the hold
        assert!(m.held() < 0.8);
        assert!(m.held() >= m.level());
    }

    #[test]
    fn held_restarts_on_higher_peak() {
        let mut m = AudioPeak::new().hold_ms(100.0);
        let mut pos = 0;

        feed(&mut m, &mut pos, 0.5, 1);
        feed(&mut m, &mut pos, 0.0, 4000);
        feed(&mut m, &mut pos, 0.9, 1);
        feed(&mut m, &mut pos, 0.0, 4000);

        assert_eq!(m.held(), 0.9);
    }

    #[test]
    fn clip_is_sticky_until_reset() {
        let mut m = AudioPeak::new();
        let mut pos = 0;

        feed(&mut m, &mut pos, 0.99, 10);
        assert!(!m.clipped());

        feed(&mut m, &mut pos, -1.0, 1);
        feed(&mut m, &mut pos, 0.0, 48_000);
        assert!(m.clipped());

        m.reset_clip();
        assert!(!m.clipped());
    }

    #[test]
    fn custom_clip_threshold() {
        let mut m = AudioPeak::new().clip_threshold(0.5);
        m.consume(&[0.6], at(0));
        assert!(m.clipped());
    }

    #[test]
    fn dropped_samples_decay_like_silence() {
        let mut gap = AudioPeak::new();
        let mut silence = AudioPeak::new();

        gap.consume(&[1.0], at(0));
        gap.consume(&[0.0], at(48_001)); // 48000 samples dropped

        silence.consume(&[1.0], at(0));
        silence.consume(&vec![0.0; 48_001], at(1));

        assert_approx_eq!(gap.level_db(), silence.level_db(), 0.01);
        assert_approx_eq!(gap.held_db(), silence.held_db(), 0.01);
    }

    #[test]
    fn decays_to_exact_zero() {
        let mut m = AudioPeak::new().hold_ms(0.0);
        let mut pos = 0;

        feed(&mut m, &mut pos, 1.0, 1);
        feed(&mut m, &mut pos, 0.0, 48_000 * 10); // 200 dB below

        assert_eq!(m.level(), 0.0);
        assert_eq!(m.held(), 0.0);
    }

    #[test]
    fn reset_clears_everything() {
        let mut m = AudioPeak::new();
        m.consume(&[1.0], at(0));
        m.reset();

        assert_eq!(m.level(), 0.0);
        assert_eq!(m.held(), 0.0);
        assert!(!m.clipped());
    }

    #[test]
    fn works_without_timing() {
        let mut m = AudioPeak::new();
        m.consume(&[0.5], BlockTime::none());
        m.consume(&[0.0; 100], BlockTime::none());
        assert!(m.level() > 0.0 && m.level() < 0.5);
    }
}
