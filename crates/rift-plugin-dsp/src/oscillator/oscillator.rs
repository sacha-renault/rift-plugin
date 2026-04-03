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

    /// Number of currently active voices. Used as a fast path for
    /// [`is_active`](Self::is_active).
    active_count: usize,
}

impl Oscillator {
    /// Creates a new polyphonic oscillator with all 127 voices inactive.
    pub fn new(samplerate: f32) -> Self {
        let mut voices = BoundedVec::new(127);
        voices.resize_with(127, || OscillatorPosition::new(samplerate));
        Oscillator {
            voices,
            active_count: 0,
        }
    }

    /// Returns `true` if at least one voice is currently active.
    pub fn is_active(&self) -> bool {
        self.active_count != 0
    }

    /// Triggers the voice for `note`, converting it to a frequency via
    /// `midi_to_frequency` and resetting its phase to the value produced
    /// by `phase_generator`.
    ///
    /// The note is clamped to the range 0..=126 before indexing.
    pub fn trigger_with_phase<F>(&mut self, note: u8, phase_generator: F)
    where
        F: FnOnce() -> f32,
    {
        let frequency = midi_to_frequency(note);
        self.voices[note.clamp(0, 126) as usize].trigger_with_phase(frequency, phase_generator);
    }

    /// Deactivates the voice for `note`, stopping it from contributing
    /// to future output.
    ///
    /// The note is clamped to the range 0..=126 before indexing.
    pub fn deactivate(&mut self, note: u8) {
        self.voices[note.clamp(0, 126) as usize].deactivate();
    }

    /// Advances every active voice by one sample, maps each phase through
    /// `generator`, and returns the sum of all active voices.
    ///
    /// `generator` receives a phase value in [0, 1) and should return the
    /// corresponding waveform amplitude (for example, a sine lookup or
    /// wavetable read).
    pub fn get_next<F>(&mut self, generator: F) -> f32
    where
        F: Fn(f32) -> f32,
    {
        let mut value = 0.;

        for voice in self.voices.iter_mut() {
            if voice.is_active() {
                value += generator(voice.get_next())
            }
        }

        value
    }
}
