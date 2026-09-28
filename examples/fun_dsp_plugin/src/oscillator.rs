use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::utils::bounded_vec::BoundedVec;
use rift_plugin_dsp::oscillator::OscillatorPosition;

pub struct Oscillator {
    phases: BoundedVec<(u8, OscillatorPosition)>,
    wavetables: BoundedVec<An<PhaseSynth>>,
    samplerate: f64,
}

impl Oscillator {
    pub fn new(samplerate: f64, phases_cap: usize, wt_cap: usize) -> Self {
        Self {
            samplerate,
            phases: BoundedVec::new(phases_cap),
            wavetables: BoundedVec::new(wt_cap),
        }
    }

    pub fn set_wavetables(&mut self, wt: Vec<Wavetable>) {
        self.wavetables.clear();

        for table in wt {
            let mut s = An(PhaseSynth::new(Arc::new(table)));
            AudioUnit::set_sample_rate(&mut s, self.samplerate);
            self.wavetables.push(s);
        }
    }

    pub fn trigger(
        &mut self,
        note: u8,
        frequency: f32,
        phase_generator: impl FnOnce() -> f32,
    ) -> bool {
        let mut pos = OscillatorPosition::new(self.samplerate as f32);
        pos.trigger_with_phase(frequency, phase_generator);

        self.phases.try_push((note, pos)).is_ok()
    }

    pub fn deactivate(&mut self, note: u8) {
        self.phases.retain(|(n, _)| *n != note);
    }

    pub fn tick(&mut self, mut wt_pos: usize) -> [f32; 2] {
        let mut l = 0f32;
        let mut r = 0f32;

        wt_pos = std::cmp::min(wt_pos, self.wavetables.len());

        for (_, phase) in self.phases.iter_mut() {
            let value = self.wavetables[wt_pos].filter_mono(phase.get_next_phase());

            l += value;
            r += value;
        }

        [l, r]
    }
}
