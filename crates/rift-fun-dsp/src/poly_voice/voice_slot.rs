use fundsp::prelude32::*;

/// info avout the voice slot
#[derive(Debug, Clone, Copy, Default)]
pub struct VoiceSlotInfo {
    /// This represent the number of samples that was processed on this note
    /// since the note was pressed / `Held`. This is reseted on every trigger.
    pub processed_samples: u64,
    pub note: u8,
    pub velocity: u8,
}

impl VoiceSlotInfo {
    /// Get the velocity on a scale 0. to 1.
    pub fn velocity_scaled(&self) -> f32 {
        const MAX_VELOCITY: f32 = u8::MAX as f32;
        self.velocity as f32 / MAX_VELOCITY
    }
}

/// One slot of the voice pool: a reusable voice plus its envelope and gate.
pub(super) struct VoiceSlot<T> {
    pub(super) inner: T,
    pub(super) adsr: super::Adsr,
    pub(super) info: VoiceSlotInfo,
}

impl<T> VoiceSlot<T>
where
    T: super::VoiceSource,
{
    /// Builds an idle slot whose envelope is driven by the shared ADSR
    /// parameters.
    pub fn new(
        config: &T::Config,
        attack: &An<Var>,
        decay: &An<Var>,
        sustain: &An<Var>,
        release: &An<Var>,
        sample_rate: f32,
    ) -> Self {
        let mut adsr = super::Adsr::new(
            attack.clone(),
            decay.clone(),
            sustain.clone(),
            release.clone(),
        );
        adsr.set_sample_rate(sample_rate);

        let mut inner = T::new(config);
        inner.set_samplerate(sample_rate);

        Self {
            inner,
            adsr,
            info: VoiceSlotInfo::default(),
        }
    }

    /// Restarts the slot on `note` without allocating a new voice.
    pub fn trigger(&mut self, note: u8, velocity: u8) {
        self.inner.reset(note);
        self.info.note = note;
        self.info.velocity = velocity;
        self.info.processed_samples = 0;
        self.adsr.trigger();
    }

    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.inner.set_samplerate(samplerate);
        self.adsr.set_sample_rate(samplerate);
    }

    pub fn note(&self) -> u8 {
        self.info.note
    }
}
