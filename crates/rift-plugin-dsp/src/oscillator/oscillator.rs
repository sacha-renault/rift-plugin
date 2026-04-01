use rift_plugin_core::utils::{bounded_vec::BoundedVec, notes::midi_to_frequency};

use super::OscillatorPosition;

pub struct Oscillator {
    voices: BoundedVec<OscillatorPosition>,
    active_count: usize,
}

impl Oscillator {
    pub fn new(samplerate: f32) -> Self {
        let mut voices = BoundedVec::new(127);
        voices.resize_with(127, || OscillatorPosition::new(samplerate));
        Oscillator {
            voices,
            active_count: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_count != 0
    }

    pub fn trigger_with_phase<F>(&mut self, note: u8, phase_generator: F)
    where
        F: FnOnce() -> f32,
    {
        let frequency = midi_to_frequency(note);
        self.voices[note.clamp(0, 126) as usize].trigger_with_phase(frequency, phase_generator);
    }

    pub fn deactivate(&mut self, note: u8) {
        self.voices[note.clamp(0, 126) as usize].deactivate();
    }

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
