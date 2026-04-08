use rift_plugin_core::utils::{bounded_vec::BoundedVec, notes::midi_to_frequency};

use super::OscillatorPosition;

/// A polyphonic oscillator that maps one voice per MIDI note (0 through 126).
///
/// Each voice is an [`OscillatorPosition`] indexed directly by note number,
/// so triggering the same note twice will retrigger rather than stack.
/// The `generator` function passed to [`get_next`](Self::get_next) defines the
/// waveform shape by mapping a phase in [0, 1) to an amplitude.
pub struct Oscillator {
    /// Fixed-size pool of 127 voices, one per MIDI note.
    voices: BoundedVec<OscillatorPosition>,

    /// cached value to easy fetch who is active and who isn't
    active_voices: Vec<usize>,
}

impl Oscillator {
    /// Creates a new polyphonic oscillator with all 127 voices inactive.
    pub fn new(samplerate: f32) -> Self {
        let mut voices = BoundedVec::new(127);
        voices.resize_with(127, || OscillatorPosition::new(samplerate));
        let active_voices = Vec::with_capacity(127);

        Oscillator {
            voices,
            active_voices,
        }
    }

    /// Returns `true` if at least one voice is currently active.
    pub fn is_active(&self) -> bool {
        !self.active_voices.is_empty()
    }

    /// Triggers the voice for `note`, converting it to a frequency via
    /// `midi_to_frequency` and resetting its phase to the value produced
    /// by `phase_generator`.
    ///
    /// The note is clamped to the range 0..=126 before indexing.
    pub fn trigger<F>(&mut self, note: u8, phase_generator: F)
    where
        F: FnOnce() -> f32,
    {
        let frequency = midi_to_frequency(note);
        let idx = note.clamp(0, 126) as usize;
        if !self.voices[idx].is_active() {
            self.active_voices.push(idx);
        }

        self.voices[idx].trigger_with_phase(frequency, phase_generator);
    }

    /// Deactivates the voice for `note`, stopping it from contributing
    /// to future output.
    ///
    /// The note is clamped to the range 0..=126 before indexing.
    pub fn deactivate(&mut self, note: u8) {
        let idx = note.clamp(0, 126) as usize;
        self.voices[idx].deactivate();

        if let Some(remove_idx) = self.active_voices.iter().position(|i| *i == idx) {
            self.active_voices.swap_remove(remove_idx);
        }
    }

    /// Advances every active voice by one sample, maps each phase through
    /// `generator`, and returns the sum of all active voices.
    ///
    /// `generator` receives a phase value in [0, 1) and should return the
    /// corresponding waveform amplitude (for example, a sine lookup or
    /// wavetable read).
    pub fn get_next<F>(&mut self, mut generator: F) -> f32
    where
        F: FnMut(f32) -> f32,
    {
        let mut value = 0.;

        for &voice_idx in self.active_voices.iter() {
            value += generator(self.voices[voice_idx].get_next_phase());
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLERATE: f32 = 48e3;

    fn assert_num_voice_correct(osc: &Oscillator, active_voices: usize) {
        assert_eq!(osc.active_voices.len(), active_voices);
        assert_eq!(
            osc.voices.iter().filter(|voice| voice.is_active()).count(),
            active_voices
        );

        for (idx, voice) in osc.voices.iter().enumerate() {
            if voice.is_active() {
                assert!(osc.active_voices.contains(&idx))
            } else {
                assert!(!osc.active_voices.contains(&idx))
            }
        }
    }

    #[test]
    fn inactive_returns_zero() {
        let mut osc = Oscillator::new(SAMPLERATE);
        let y = osc.get_next(|x| x);

        assert_eq!(y, 0.);
        assert_num_voice_correct(&osc, 0);
    }

    #[test]
    fn single_voice() {
        let mut osc = Oscillator::new(SAMPLERATE);
        osc.trigger(60, || 0.);

        let y = osc.get_next(|x| x.cos());

        assert_num_voice_correct(&osc, 1);
        assert_eq!(y, 1.); // cos(0) = 1
    }

    #[test]
    fn activation_deactivation() {
        let mut osc = Oscillator::new(SAMPLERATE);

        assert!(!osc.is_active());
        osc.trigger(60, || 0.);
        assert_num_voice_correct(&osc, 1);

        // trigger an other voice
        osc.trigger(72, || 0.);
        assert!(osc.is_active());
        assert_num_voice_correct(&osc, 2);

        // retrig same voice, so no extra activation, just phase reset
        osc.trigger(72, || 0.);
        assert_num_voice_correct(&osc, 2);

        osc.deactivate(72);
        assert_num_voice_correct(&osc, 1);

        // trigger an other voice
        osc.trigger(84, || 0.);
        assert_num_voice_correct(&osc, 2);

        osc.deactivate(84);
        assert_num_voice_correct(&osc, 1);

        osc.deactivate(60);
        assert_num_voice_correct(&osc, 0);
        assert!(!osc.is_active());
    }
}
