/// Tracks the phase position of an oscillator, advancing it each sample
/// based on the current frequency and sample rate.
///
/// The position is maintained as a value in the range [0, 1), representing
/// one full cycle of the oscillator. When no frequency is set, the oscillator
/// is considered inactive and its position will not advance.
pub struct OscillatorPosition {
    /// position inside of the oscillator
    position: f32,

    /// 1. / samplerate
    sr_recip: f32,

    /// The current frequency in Hz, or `None` if the oscillator is inactive.
    frequency: Option<f32>,
}

impl OscillatorPosition {
    /// Creates a new inactive oscillator at phase 0 for the given sample rate.
    pub fn new(samplerate: f32) -> Self {
        Self {
            position: 0.,
            sr_recip: samplerate.recip(),
            frequency: None,
        }
    }

    /// Activates the oscillator at the given frequency and resets the phase
    /// to a value produced by `phase_generator`.
    ///
    /// This is useful when you want a voice to start at a random or
    /// otherwise non-zero phase to avoid phase alignment artifacts.
    pub fn trigger_with_phase<F>(&mut self, frequency: f32, phase_generator: F)
    where
        F: FnOnce() -> f32,
    {
        self.frequency = Some(frequency);
        self.position = phase_generator();
    }

    /// Activates the oscillator at the given frequency and resets the phase
    /// to zero.
    pub fn trigger(&mut self, frequency: f32) {
        self.frequency = Some(frequency);
        self.position = 0.;
    }

    /// Updates the frequency without resetting the phase.
    ///
    /// todo!() how to make it work only when the oscillator is active ?
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = Some(frequency);
    }

    /// Deactivates the oscillator. While inactive, calls to [`advance_by`]
    /// will not change the position.
    pub fn deactivate(&mut self) {
        self.frequency = None;
    }

    /// Returns the current phase position in the range [0, 1).
    #[inline]
    pub fn get_current(&self) -> f32 {
        self.position
    }

    /// Advances the phase by one sample and returns the new position.
    #[inline]
    pub fn get_next(&mut self) -> f32 {
        self.advance_by(1);
        self.get_current()
    }

    /// Advances the phase by `sample_count` samples.
    ///
    /// The resulting position is wrapped into [0, 1) via `rem_euclid`.
    /// If the oscillator is inactive, this is a no-op.
    pub fn advance_by(&mut self, sample_count: usize) {
        if let Some(frequency) = self.frequency {
            let pos = self.position + self.sr_recip * frequency * (sample_count as f32);

            // todo!()
            // this might drift because of f32 precision
            // a way to avoid that is calculating position from absolute position
            // this would requires knowing frame it was turned on and the actual frame
            // Note: this would be annoying if transport stop playing, or if changing frequency etc ...
            // maybe this is "good enough" actually. Needs to be checked anyway ...
            self.position = pos.rem_euclid(1.);
        }
    }

    /// Returns `true` if the oscillator has an active frequency.
    pub fn is_active(&self) -> bool {
        self.frequency.is_some()
    }
}
