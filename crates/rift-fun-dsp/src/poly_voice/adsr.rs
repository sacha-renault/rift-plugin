//! Per-voice Attack/Decay/Sustain/Release envelope.

use fundsp::prelude32::{An, Var};
use rift_plugin_utils::DEFAULT_SAMPLERATE;

/// The segment of the envelope currently being played.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Silent and waiting for a note.
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// A single-voice ADSR envelope driven by 4 shared variables.
pub struct Adsr {
    attack: An<Var>,
    decay: An<Var>,
    sustain: An<Var>,
    release: An<Var>,

    sample_rate: f32,
    stage: Stage,
    /// Current envelope gain in `0..=1`.
    value: f32,
    /// Gain captured when the release began, so the ramp always reaches zero
    /// after `release` seconds no matter which stage the key was released in.
    release_level: f32,
}

impl Adsr {
    /// Creates an idle envelope driven by `attack`, `decay`, `sustain`,
    /// `release`.
    pub fn new(attack: An<Var>, decay: An<Var>, sustain: An<Var>, release: An<Var>) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            sample_rate: DEFAULT_SAMPLERATE as f32,
            stage: Stage::Idle,
            value: 0.0,
            release_level: 0.0,
        }
    }

    /// Sets the sample rate used to turn the envelope times into per-sample
    /// increments.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Begins a note: restarts the attack from silence so the envelope can be
    /// retriggered.
    pub fn trigger(&mut self) {
        self.stage = Stage::Attack;
        self.value = 0.0;
        self.release_level = 0.0;
    }

    /// Marks the note as released: the key is let go and the envelope ramps down
    /// to silence from its current level.
    pub fn release(&mut self) {
        if matches!(self.stage, Stage::Attack | Stage::Decay | Stage::Sustain) {
            self.release_level = self.value;
            self.stage = Stage::Release;
        }
    }

    /// Returns true while the note is sounding, either held or still ringing
    /// through its release tail.
    pub fn is_sounding(&self) -> bool {
        !matches!(self.stage, Stage::Idle)
    }

    /// Advances the envelope by one sample and returns its gain in `0..=1`.
    pub fn tick(&mut self) -> f32 {
        let dt = self.sample_rate.recip();

        match self.stage {
            Stage::Idle => self.value = 0.0,
            Stage::Attack => {
                let attack = self.attack.value().max(0.0);
                self.value += if attack > 0.0 { dt / attack } else { 1.0 };
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                let sustain = self.sustain.value().clamp(0.0, 1.0);
                let decay = self.decay.value().max(0.0);
                if decay > 0.0 {
                    self.value -= (1.0 - sustain) * dt / decay;
                }
                if decay <= 0.0 || self.value <= sustain {
                    self.value = sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.value = self.sustain.value().clamp(0.0, 1.0),
            Stage::Release => {
                let release = self.release.value().max(0.0);
                if release > 0.0 {
                    self.value -= self.release_level * dt / release;
                } else {
                    self.value = 0.0;
                }
                if self.value <= 0.0 {
                    // The envelope is silent: hand the voice back to `PolyVoice`.
                    self.value = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }

        self.value
    }
}

#[cfg(test)]
mod tests {
    use fundsp::prelude32::{shared, var};

    use super::Adsr;

    /// Builds a held envelope running at 1 ms per sample (1 kHz) so that a time
    /// in seconds is also a count of samples.
    fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Adsr {
        let mut adsr = Adsr::new(
            var(&shared(attack)),
            var(&shared(decay)),
            var(&shared(sustain)),
            var(&shared(release)),
        );
        adsr.set_sample_rate(1000.0);
        adsr.trigger();
        adsr
    }

    #[test]
    fn attacks_then_decays_to_sustain() {
        // 20 samples attack, 20 samples decay down to 0.5.
        let mut adsr = adsr(0.02, 0.02, 0.5, 0.5);

        for _ in 0..20 {
            adsr.tick();
        }
        assert!((adsr.value - 1.0).abs() < 1e-3, "peak = {}", adsr.value);

        for _ in 0..20 {
            adsr.tick();
        }
        assert!((adsr.value - 0.5).abs() < 1e-3, "sustain = {}", adsr.value);
    }

    #[test]
    fn release_fades_and_marks_idle() {
        // Instant attack/decay into a full sustain, 10-sample release.
        let mut adsr = adsr(0.0, 0.0, 1.0, 0.01);

        for _ in 0..5 {
            adsr.tick();
        }
        assert_eq!(adsr.value, 1.0);
        assert!(adsr.is_sounding());

        adsr.release();
        for _ in 0..10 {
            adsr.tick();
        }

        assert_eq!(adsr.value, 0.0);
        assert!(!adsr.is_sounding(), "voice should be released back");
    }
}
