use crate::scale::Scale;

/// Configuration of a float parameter: its plain bounds and normalization curve.
#[derive(Debug, Clone, Copy)]
pub struct FloatRange {
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) scale: Scale,
}

impl Default for FloatRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            scale: Scale::Linear,
        }
    }
}

impl FloatRange {
    /// A linear range spanning `min..=max`.
    pub fn linear(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            scale: Scale::Linear,
        }
    }

    /// A power-curve range (`factor > 0`).
    pub fn skew(min: f32, max: f32, factor: f32) -> Self {
        Self {
            min,
            max,
            scale: Scale::Skew(factor),
        }
    }

    /// An exponential-curve range (`factor > 0`, `factor != 1`).
    pub fn exp(min: f32, max: f32, factor: f32) -> Self {
        Self {
            min,
            max,
            scale: Scale::Exponential(factor),
        }
    }
}

/// Configuration of an integer parameter: its plain bounds. Always linear.
#[derive(Debug, Clone, Copy)]
pub struct IntRange {
    pub(crate) min: i32,
    pub(crate) max: i32,
}

impl Default for IntRange {
    fn default() -> Self {
        Self { min: 0, max: 1 }
    }
}

impl IntRange {
    /// A linear range spanning `min..=max`.
    pub fn linear(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}
