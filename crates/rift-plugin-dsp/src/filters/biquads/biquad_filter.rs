use crate::filters::biquads::biquad_args::BiquadConfig;

use super::biquad_args::FilterMode;
use super::utils::*;

/// Pre-normalized biquad coefficients (all divided by a0).
///
/// See cookbook at:
/// `https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html`
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
    pub fn precompute_coeffs(a0: f32, a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) -> Self {
        Self {
            b0_a0: b0 / a0,
            b1_a0: b1 / a0,
            b2_a0: b2 / a0,
            a1_a0: a1 / a0,
            a2_a0: a2 / a0,
        }
    }
    
    #[rustfmt::skip]
    pub fn new(samplerate: f32, args: BiquadConfig) -> Self {
        use FilterMode::*;

        match args.mode {
            LowPass { cutoff, .. } => Self::lowpass(cutoff, args.get_q(), samplerate),
            HighPass { cutoff, .. } => Self::highpass(cutoff, args.get_q(), samplerate),
            Peaking { frequency, gain, ..} => {
                Self::peaking(frequency, args.get_q(), samplerate, gain)
            }
            LowShelf { frequency, gain, .. } => {
                Self::low_shelf(frequency, args.get_q(), samplerate, gain)
            }
            HighShelf { frequency, gain, ..} => {
                Self::high_shelf(frequency, args.get_q(), samplerate, gain)
            }
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
        Self::precompute_coeffs(a0, a1, a2, b0, b1, b2)
    }

    pub fn highpass(cutoff: f32, q: f32, samplerate: f32) -> Self {
        let w0 = compute_w0(cutoff, samplerate);
        let alpha = compute_alpha(w0, q);

        let b0 = (1. + w0.cos()) / 2.;
        let b1 = -(1. + w0.cos());
        let b2 = b0;
        let a0 = 1. + alpha;
        let a1 = -2. * w0.cos();
        let a2 = 1. - alpha;
        Self::precompute_coeffs(a0, a1, a2, b0, b1, b2)
    }

    pub fn peaking(frequency: f32, q: f32, samplerate: f32, gain: f32) -> Self {
        let a = 10f32.powf(gain / 40f32);
        let w0 = compute_w0(frequency, samplerate);
        let alpha = compute_alpha(w0, q);

        let b0 = 1. + alpha * a;
        let b1 = -2. * w0.cos();
        let b2 = 1. - alpha * a;
        let a0 = 1. + alpha / a;
        let a1 = -2. * w0.cos();
        let a2 = 1. - alpha / a;

        Self::precompute_coeffs(a0, a1, a2, b0, b1, b2)
    }

    pub fn low_shelf(frequency: f32, q: f32, samplerate: f32, gain: f32) -> Self {
        let a = 10f32.powf(gain / 40.0);
        let w0 = compute_w0(frequency, samplerate);
        let alpha = compute_alpha(w0, q);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * w0.cos() + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * w0.cos());
        let b2 = a * ((a + 1.0) - (a - 1.0) * w0.cos() - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * w0.cos() + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * w0.cos());
        let a2 = (a + 1.0) + (a - 1.0) * w0.cos() - two_sqrt_a_alpha;

        Self::precompute_coeffs(a0, a1, a2, b0, b1, b2)
    }

    pub fn high_shelf(frequency: f32, q: f32, samplerate: f32, gain: f32) -> Self {
        let a = 10f32.powf(gain / 40.0);
        let w0 = compute_w0(frequency, samplerate);
        let alpha = compute_alpha(w0, q);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * w0.cos() + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * w0.cos());
        let b2 = a * ((a + 1.0) + (a - 1.0) * w0.cos() - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * w0.cos() + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * w0.cos());
        let a2 = (a + 1.0) - (a - 1.0) * w0.cos() - two_sqrt_a_alpha;

        Self::precompute_coeffs(a0, a1, a2, b0, b1, b2)
    }
}

/// Per-sample state for a single biquad (the two delay lines).
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

/// A single second-order IIR filter section.
///
/// Inactive stages pass the signal through unchanged.
#[derive(Clone)]
pub struct BiquadFilter {
    pub(crate) states: BiquadStates,
    pub(crate) coefficients: BiquadCoefficient,
    pub(crate) is_active: bool,
}

impl BiquadFilter {
    pub fn new(samplerate: f32, args: BiquadConfig, is_active: bool) -> Self {
        Self {
            states: BiquadStates::default(),
            coefficients: BiquadCoefficient::new(samplerate, args),
            is_active,
        }
    }

    #[inline]
    pub fn process(&mut self, xn: f32) -> f32 {
        if !self.is_active {
            return xn;
        }

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
