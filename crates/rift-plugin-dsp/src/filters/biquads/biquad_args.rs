use super::biquad_qs::*;

/// Maximum number of biquad stages in a cascade. Corresponds to order 20.
pub const CASCADE_MAX_DEPTH: usize = 10;

/// Filter order for a [`super::BiquadCascade`], from 2nd to 20th order (even only).
/// Each order adds one biquad stage, with Q values tuned for Butterworth response.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterOrder {
    Two,
    Four,
    Six,
    Eight,
    Ten,
    Twelve,
    Fourteen,
    Sixteen,
    Eighteen,
    Twenty,
}

impl FilterOrder {
    /// Returns the number of biquad stages for this order.
    pub fn num_stages(&self) -> usize {
        match self {
            FilterOrder::Two => 1,
            FilterOrder::Four => 2,
            FilterOrder::Six => 3,
            FilterOrder::Eight => 4,
            FilterOrder::Ten => 5,
            FilterOrder::Twelve => 6,
            FilterOrder::Fourteen => 7,
            FilterOrder::Sixteen => 8,
            FilterOrder::Eighteen => 9,
            FilterOrder::Twenty => 10,
        }
    }
}

/// Filter topology for a [`super::BiquadCascade`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterMode {
    /// Cutoff frequency in Hz.
    LowPass {
        cutoff: f32,
        order: FilterOrder,
    },

    /// Cutoff frequency in Hz.
    HighPass {
        cutoff: f32,
        order: FilterOrder,
    },

    Peaking {
        frequency: f32,
        gain: f32,
        q: f32,
    },
}

impl FilterMode {
    /// Returns the Butterworth Q for the given stage index (0-based).
    pub fn get_q(&self, cascade_depth: usize) -> f32 {
        match self {
            FilterMode::LowPass { order, .. } | FilterMode::HighPass { order, .. } => {
                let num_stages = order.num_stages();
                PASS_Q_ORDER[num_stages - 1][cascade_depth]
            }
            FilterMode::Peaking { q, .. } => *q,
        }
    }

    pub fn num_stages(&self) -> usize {
        match self {
            FilterMode::LowPass { order, .. } | FilterMode::HighPass { order, .. } => {
                order.num_stages()
            }
            FilterMode::Peaking { .. } => 1,
        }
    }
}

/// Bundles a filter mode, order, and cascade stage index into a single
/// argument for constructing [`BiquadCoefficient`](super::biquad_filter::BiquadCoefficient).
///
/// The `depth` field identifies which stage in the cascade this biquad
/// represents, which determines the Butterworth Q value used.
pub struct BiquadConfig {
    pub mode: FilterMode,
    pub depth: usize,
}

impl BiquadConfig {
    pub fn new(mode: FilterMode, depth: usize) -> Self {
        Self { mode, depth }
    }

    pub fn get_q(&self) -> f32 {
        self.mode.get_q(self.depth)
    }
}
