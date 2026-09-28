use rift_plugin_utils::notes::midi_to_frequency;

/// Tracks the phase position of an oscillator, advancing it each sample
/// based on the current frequency and sample rate.
pub struct OscillatorVoice {
    /// Note that was trigger
    note: u8,

    /// position inside of the oscillator
    position: f32,

    /// 1. / samplerate
    sr_recip: f32,

    /// The current frequency in Hz, or `None` if the oscillator is inactive.
    frequency: f32,

    /// Table hint:
    /// can be usefull in the case there is a wavetable choice to do
    /// (for ex. in fundsp)
    table_hint: usize,
}

impl OscillatorVoice {
    /// Creates a new inactive oscillator at phase 0 for the given sample rate.
    pub fn new(samplerate: f32, note: u8) -> Self {
        Self::new_with_phase(samplerate, note, 0f32)
    }

    pub fn new_with_phase_generator(
        samplerate: f32,
        note: u8,
        phase: impl FnOnce() -> f32,
    ) -> Self {
        Self::new_with_phase(samplerate, note, phase())
    }

    pub fn new_with_phase(samplerate: f32, note: u8, phase: f32) -> Self {
        Self {
            position: phase.clamp(0f32, 1f32),
            note,
            sr_recip: samplerate.recip(),
            frequency: midi_to_frequency(note),
            table_hint: 0,
        }
    }

    /// Returns the current phase position in the range [0, 1).
    #[inline]
    pub fn current(&self) -> f32 {
        self.position
    }

    /// First get current value then advance by once
    ///
    /// **FIX**: Now get the value directly to avoid skipping value on the first call.
    #[inline]
    pub fn next_phase(&mut self) -> f32 {
        let value = self.current();
        self.advance_by(1);
        value
    }

    /// Advances the phase by `sample_count` samples.
    ///
    /// The resulting position is wrapped into [0, 1) via `rem_euclid`.
    /// If the oscillator is inactive, this is a no-op.
    pub fn advance_by(&mut self, sample_count: usize) {
        let pos = self.position + self.sr_recip * self.frequency * (sample_count as f32);

        // todo!()
        // this might drift because of f32 precision
        // a way to avoid that is calculating position from absolute position
        // this would requires knowing frame it was turned on and the actual frame
        // Note: this would be annoying if transport stop playing, or if changing frequency etc ...
        // maybe this is "good enough" actually. Needs to be checked anyway ...
        self.position = pos.rem_euclid(1.);
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn note(&self) -> u8 {
        self.note
    }

    pub fn table_hint(&self) -> usize {
        self.table_hint
    }

    pub fn set_table_hint(&mut self, hint: usize) {
        self.table_hint = hint;
    }
}
