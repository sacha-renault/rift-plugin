use crate::scale::Scale;

/// Configuration of a float parameter: its plain bounds and normalization curve.
#[derive(Debug, Clone, Copy)]
pub struct FloatRange {
    min: f32,
    max: f32,
    scale: Scale,
}

impl FloatRange {
    const fn new(min: f32, max: f32, scale: Scale) -> Self {
        assert!(min != max, "range min and max must differ");
        assert!(
            min.is_finite() && max.is_finite(),
            "min & max must be a finite number"
        );

        Self { min, max, scale }
    }

    #[inline]
    pub(crate) fn min(&self) -> f32 {
        self.min
    }

    #[inline]
    pub(crate) fn max(&self) -> f32 {
        self.max
    }

    #[inline]
    pub(crate) fn normalize(&self, value: f32) -> f32 {
        self.scale.normalize(value, self.min, self.max)
    }

    #[inline]
    pub(crate) fn denormalize(&self, normalized: f32) -> f32 {
        self.scale.denormalize(normalized, self.min, self.max)
    }
}

impl Default for FloatRange {
    fn default() -> Self {
        Self::new(0., 1., Scale::Linear)
    }
}

// The constructors are `const fn` with their own asserts, and the derive emits the
// call inside a `const` block, so a bad `range = ...` is a compile error.
impl FloatRange {
    /// A linear range spanning `min..=max`.
    pub const fn linear(min: f32, max: f32) -> Self {
        Self::new(min, max, Scale::Linear)
    }

    /// A power-curve range; `factor` must be finite and greater than 0.
    ///
    /// ```compile_fail
    /// const _: rift_plugin_params::FloatRange =
    ///     rift_plugin_params::FloatRange::skew(20.0, 20_000.0, -1.0);
    /// ```
    pub const fn skew(min: f32, max: f32, factor: f32) -> Self {
        assert!(
            factor.is_finite() && factor > 0.0,
            "skew factor must be finite and greater than 0"
        );
        Self::new(min, max, Scale::Skew(factor))
    }

    /// An exponential-curve range; `factor` must be finite, greater than 0 and
    /// different from 1.
    pub const fn exp(min: f32, max: f32, factor: f32) -> Self {
        assert!(
            factor.is_finite() && factor > 0.0 && factor != 1.0,
            "exp factor must be finite, greater than 0 and different from 1"
        );
        Self::new(min, max, Scale::Exponential(factor))
    }

    pub const fn log(min: f32, max: f32) -> Self {
        assert!(
            min > 0f32 && max > 0f32,
            "log range requires strictly positive bounds"
        );

        Self::new(min, max, Scale::Log)
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
    pub const fn linear(min: i32, max: i32) -> Self {
        assert!(min != max, "range min and max must differ");
        Self { min, max }
    }
}
