use rift_plugin_core::utils::bounded_vec::BoundedVec;

use crate::filters::utils::*;

mod utils;

#[derive(Clone)]
pub struct BiquadCoefficient {
    /// b0 / a0
    b0_a0: f32,

    /// b1 / a0
    b1_a0: f32,

    /// b2 / a0
    b2_a0: f32,

    /// a1 / a0
    a1_a0: f32,

    /// a2 / a0
    a2_a0: f32,
}

impl BiquadCoefficient {
    pub fn new(samplerate: f32, mode: FilterMode, q: f32) -> Self {
        use FilterMode::*;

        match mode {
            LowPass { cutoff } => Self::lowpass(cutoff, q, samplerate),
        }
    }

    pub fn from_coeff(a0: f32, a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) -> Self {
        Self {
            b0_a0: b0 / a0,
            b1_a0: b1 / a0,
            b2_a0: b2 / a0,
            a1_a0: a1 / a0,
            a2_a0: a2 / a0,
        }
    }

    pub fn lowpass(cutoff: f32, q: f32, samplerate: f32) -> Self {
        let w0 = compute_w0(cutoff, samplerate);
        let alpha = compute_alpha(w0, q);

        let b0 = (1. - w0.cos()) / 2.;
        let b1 = 1. - w0.cos();
        let b2 = b0;
        let a0 = 1. + alpha;
        let a1 = -2. * w0.cos();
        let a2 = 1. - alpha;
        Self::from_coeff(a0, a1, a2, b0, b1, b2)
    }
}

#[derive(Clone, Default)]
pub struct BiquadStates {
    /// previous x: x_n-1
    xn_1: f32,

    /// previous previous x: x_n-2
    xn_2: f32,

    /// previous y: y_n-1
    yn_1: f32,

    /// rpevious previous y: y_n-2
    yn_2: f32,
}

impl BiquadStates {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterMode {
    /// Lowpass with cutoff frequency
    LowPass { cutoff: f32 },
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterOrder {
    Second,
}

impl FilterMode {
    pub fn lowpass(cutoff_frequency: f32) -> Self {
        Self::LowPass {
            cutoff: cutoff_frequency,
        }
    }
}

#[derive(Clone)]
pub struct BiquadFilter {
    states: BiquadStates,
    coefficients: BiquadCoefficient,
    is_active: bool,
}

impl BiquadFilter {
    pub fn new(samplerate: f32, mode: FilterMode, q: f32, is_active: bool) -> Self {
        Self {
            states: BiquadStates::default(),
            coefficients: BiquadCoefficient::new(samplerate, mode, q),
            is_active,
        }
    }

    #[inline]
    pub fn process(&mut self, xn: f32) -> f32 {
        let yn = self.coefficients.b0_a0 * xn
            + self.coefficients.b1_a0 * self.states.xn_1
            + self.coefficients.b2_a0 * self.states.xn_2
            - self.coefficients.a1_a0 * self.states.yn_1
            - self.coefficients.a2_a0 * self.states.yn_2;

        self.states.xn_2 = self.states.xn_1;
        self.states.xn_1 = xn;

        self.states.yn_2 = self.states.yn_1;
        self.states.yn_1 = yn;

        yn
    }
}

#[derive(Clone)]
pub struct BiquadCascade {
    stages: BoundedVec<BiquadFilter>,
    samplerate: f32,
    mode: FilterMode,
    order: FilterOrder,
}

impl BiquadCascade {
    pub fn new(samplerate: f32, mode: FilterMode, order: FilterOrder) -> Self {
        todo!("Not yet implemented");
    }

    #[inline]
    pub fn process(&mut self, mut xn: f32) -> f32 {
        for stage in self.stages.iter_mut().take_while(|stage| stage.is_active) {
            xn = stage.process(xn);
        }

        xn
    }
}
