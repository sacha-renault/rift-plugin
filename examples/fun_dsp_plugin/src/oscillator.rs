use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::utils::bounded_vec::BoundedVec;
use rift_plugin_dsp::oscillator::OscillatorVoice;

const VOICES_CAPACITY: usize = 256;
const WAVETABLES_CAPACITY: usize = 256;

pub struct Oscillator {
    voices: BoundedVec<OscillatorVoice, VOICES_CAPACITY>,
    wavetables: BoundedVec<Arc<Wavetable>, WAVETABLES_CAPACITY>,
    samplerate: f64,
}

impl Oscillator {
    pub fn new(samplerate: f64) -> Self {
        Self {
            samplerate,
            voices: BoundedVec::new(),
            wavetables: BoundedVec::new(),
        }
    }

    pub fn set_wavetables(&mut self, wt: Vec<Arc<Wavetable>>) {
        self.wavetables.clear();

        for table in wt {
            self.wavetables.push(table);
        }
    }

    pub fn trigger(&mut self, note: u8, phase_generator: impl FnOnce() -> f32) -> bool {
        let position = OscillatorVoice::new_with_phase_generator(
            self.samplerate as f32,
            note,
            phase_generator,
        );

        self.voices.try_push(position).is_ok()
    }

    pub fn deactivate(&mut self, note: u8) {
        self.voices.retain(|v| v.note() != note);
    }

    pub fn tick(&mut self, wt_pos: f32) -> f32 {
        let wt = ((self.wavetables.len() - 1) as f32 * wt_pos.clamp(0.0, 1.0)) as usize;
        let table = &self.wavetables[wt];

        let mut v = 0f32;
        for voice in self.voices.iter_mut() {
            // fundsp's own API: frequency is known, so pass it directly
            let (value, hint) =
                table.read(voice.table_hint(), voice.frequency(), voice.next_phase());
            voice.set_table_hint(hint);
            v += value;
        }
        v
    }
}
