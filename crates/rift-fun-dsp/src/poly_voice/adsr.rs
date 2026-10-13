//! Per-voice Attack/Decay/Sustain/Release envelope.

use fundsp::prelude32::{An, Var};
use rift_plugin_utils::DEFAULT_SAMPLERATE;

pub const INACTIVE: f32 = 0.0;
pub const ACTIVE: f32 = 1.0;
pub const RELEASED: f32 = 2.0;

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

/// A single-voice ADSR envelope driven by five shared variables.
pub struct Adsr {
    attack: An<Var>,
    decay: An<Var>,
    sustain: An<Var>,
    release: An<Var>,
    active: An<Var>,

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
    /// `release` and `active`.
    pub fn new(
        attack: An<Var>,
        decay: An<Var>,
        sustain: An<Var>,
        release: An<Var>,
        active: An<Var>,
    ) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            active,
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

    /// Returns the envelope to its idle state so it can be retriggered.
    pub fn reset(&mut self) {
        self.stage = Stage::Idle;
        self.value = 0.0;
        self.release_level = 0.0;
    }

    /// Advances the envelope by one sample and returns its gain in `0..=1`.
    pub fn tick(&mut self) -> f32 {
        let dt = self.sample_rate.recip();
        let gate = self.active.value();

        if gate == ACTIVE {
            // A fresh note, or a retrigger while the tail is still ringing,
            // restarts the attack from the current level.
            if matches!(self.stage, Stage::Idle | Stage::Release) {
                self.stage = Stage::Attack;
            }
        } else if matches!(self.stage, Stage::Attack | Stage::Decay | Stage::Sustain) {
            // The key was let go: ramp down from wherever we are now.
            self.stage = Stage::Release;
            self.release_level = self.value;
        }

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
                    self.value = 0.0;
                    self.stage = Stage::Idle;
                    // The envelope is silent: hand the voice back to `PolyVoice`.
                    self.active.set_value(INACTIVE);
                }
            }
        }

        self.value
    }
}

#[cfg(test)]
mod tests {
    use fundsp::prelude32::{Shared, shared, var};

    use super::Adsr;
    use super::{ACTIVE, INACTIVE, RELEASED};

    /// Builds an envelope running at 1 ms per sample (1 kHz) so that a time in
    /// seconds is also a count of samples.
    fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> (Adsr, Shared) {
        let active = Shared::new(ACTIVE);
        let mut adsr = Adsr::new(
            var(&shared(attack)),
            var(&shared(decay)),
            var(&shared(sustain)),
            var(&shared(release)),
            var(&active),
        );
        adsr.set_sample_rate(1000.0);
        (adsr, active)
    }

    #[test]
    fn attacks_then_decays_to_sustain() {
        // 20 samples attack, 20 samples decay down to 0.5.
        let (mut adsr, _) = adsr(0.02, 0.02, 0.5, 0.5);

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
    fn release_fades_and_marks_inactive() {
        // Instant attack/decay into a full sustain, 10-sample release.
        let (mut adsr, active) = adsr(0.0, 0.0, 1.0, 0.01);

        for _ in 0..5 {
            adsr.tick();
        }
        assert_eq!(adsr.value, 1.0);
        assert_eq!(active.value(), ACTIVE);

        active.set(RELEASED);
        for _ in 0..10 {
            adsr.tick();
        }

        assert_eq!(adsr.value, 0.0);
        assert_eq!(active.value(), INACTIVE, "voice should be released back");
    }
}
