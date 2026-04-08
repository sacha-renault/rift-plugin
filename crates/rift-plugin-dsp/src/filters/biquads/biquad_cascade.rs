use rift_plugin_core::utils::bounded_vec::BoundedVec;

use super::biquad_args::*;
use super::biquad_filter::*;

/// A cascade of up to [`CASCADE_MAX_DEPTH`] biquad stages, forming a higher-order filter.
///
/// Stages are allocated lazily up to the max depth, no reallocation occurs after that.
/// When inactive, all samples pass through unchanged.
///
/// # Usage
/// ```ignore
/// let mut filter = BiquadCascade::new(44100.0);
/// filter.set_mode(FilterMode::LowPass { cutoff: 500.0 }, FilterOrder::Four);
///
/// let output = filter.process(input_sample);
///
/// filter.deactivate();
/// ```
#[derive(Clone)]
pub struct BiquadCascade {
    stages: BoundedVec<BiquadFilter>,
    samplerate: f32,
    mode: Option<FilterMode>,
    order: Option<FilterOrder>,
}

impl BiquadCascade {
    /// Creates an inactive cascade. Call [`Self::set_mode`] to activate.
    pub fn new(samplerate: f32) -> Self {
        let stages = BoundedVec::new(CASCADE_MAX_DEPTH);

        Self {
            samplerate,
            mode: None,
            order: None,
            stages,
        }
    }

    /// Deactivates the cascade and resets all stage states.
    /// The internal allocation is preserved, [`Self::set_mode`] can be called again without reallocating.
    pub fn deactivate(&mut self) {
        self.mode = None;
        self.order = None;
        for filter in self.stages.iter_mut() {
            filter.is_active = false;
            filter.states.reset();
        }
    }

    /// Sets the filter mode and order, activating the cascade.
    /// Coefficients are updated immediately, existing stage states are preserved.
    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = Some(mode);

        let num_stages = mode.num_stages();
        for (idx, filter) in self.stages.iter_mut().enumerate() {
            if idx < num_stages {
                filter.is_active = true;
                let args = BiquadConfig::new(mode, idx);
                filter.coefficients = BiquadCoefficient::new(self.samplerate, args);
            } else {
                filter.is_active = false;
            }
        }

        // only if there is more things to allocate
        for idx in self.stages.len()..num_stages {
            let args = BiquadConfig::new(mode, idx);
            let filter = BiquadFilter::new(self.samplerate, args, true);
            self.stages.push(filter);
        }
    }

    /// Processes a single sample through all active stages.
    #[inline]
    pub fn process(&mut self, mut xn: f32) -> f32 {
        for stage in self.stages.iter_mut().take_while(|stage| stage.is_active) {
            xn = stage.process(xn);
        }

        xn
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_core::utils::spaces::Linspace;

    use super::*;

    const SAMPLERATE: f32 = 48000.;

    fn make_saw_wave() -> Vec<f32> {
        let mut wave = Vec::with_capacity(SAMPLERATE as usize);

        // This will be a 40Hz saw wave at 48kHz
        for x in Linspace::new(0., 40., SAMPLERATE as usize) {
            let y = (x.rem_euclid(1.) - 0.5) * 2.;
            wave.push(y);
        }

        wave
    }

    #[test]
    fn activate() {
        let mut cascade = BiquadCascade::new(SAMPLERATE);
        let wave = make_saw_wave();

        let mode = FilterMode::HighPass {
            cutoff: 200.,
            order: FilterOrder::Four,
        };
        cascade.set_mode(mode);
        let y0 = cascade.process(wave[0]);
        let y1 = cascade.process(wave[1]);
        let y2 = cascade.process(wave[2]);

        assert_ne!(y0, wave[0]);
        assert_ne!(y1, wave[1]);
        assert_ne!(y2, wave[2]);
    }

    #[test]
    fn reactivate_with_new_copy() {
        let mut cascade = BiquadCascade::new(SAMPLERATE);

        let mode_order_4 = FilterMode::HighPass {
            cutoff: 200.,
            order: FilterOrder::Four,
        };
        let mode_order_6 = FilterMode::HighPass {
            cutoff: 200.,
            order: FilterOrder::Six,
        };

        cascade.set_mode(mode_order_4);
        let current_n_stage = cascade.stages.len();
        cascade.set_mode(mode_order_6);

        assert_ne!(current_n_stage, cascade.stages.len());
    }

    #[test]
    fn reactivate_with_no_copy() {
        let mut cascade = BiquadCascade::new(SAMPLERATE);

        let mode_order_4 = FilterMode::HighPass {
            cutoff: 200.,
            order: FilterOrder::Four,
        };
        let mode_order_6 = FilterMode::HighPass {
            cutoff: 200.,
            order: FilterOrder::Six,
        };

        cascade.set_mode(mode_order_6);
        let current_n_stage = cascade.stages.len();
        cascade.set_mode(mode_order_4);

        assert_eq!(current_n_stage, cascade.stages.len());
    }

    // todo!()
    // more test ? But idk how to do.
}
