use rift_plugin_core::utils::bounded_vec::BoundedVec;

use super::biquad_args::*;
use super::biquad_filter::*;

#[derive(Clone)]
pub struct BiquadCascade {
    stages: BoundedVec<BiquadFilter>,
    samplerate: f32,
    mode: Option<FilterMode>,
    order: Option<FilterOrder>,
}

impl BiquadCascade {
    pub fn new(samplerate: f32) -> Self {
        let stages = BoundedVec::new(CASCADE_MAX_DEPTH);

        Self {
            samplerate,
            mode: None,
            order: None,
            stages,
        }
    }

    pub fn deactivate(&mut self) {
        self.mode = None;
        self.order = None;
        for filter in self.stages.iter_mut() {
            filter.is_active = false;
            filter.states.reset();
        }
    }

    pub fn set_mode(&mut self, mode: FilterMode, order: FilterOrder) {
        self.mode = Some(mode);
        self.order = Some(order);

        let num_stages = order.get_num_stages();
        for (idx, filter) in self.stages.iter_mut().enumerate() {
            if idx < num_stages {
                filter.is_active = true;
                filter.coefficients =
                    BiquadCoefficient::new(self.samplerate, mode, order.get_q(idx));
            } else {
                filter.is_active = false;
            }
        }

        // only if there is more things to allocate
        for idx in self.stages.len()..num_stages {
            let filter = BiquadFilter::new(self.samplerate, mode, order.get_q(idx), true);
            self.stages.push(filter);
        }
    }

    #[inline]
    pub fn process(&mut self, mut xn: f32) -> f32 {
        for stage in self.stages.iter_mut().take_while(|stage| stage.is_active) {
            xn = stage.process(xn);
        }

        xn
    }
}
