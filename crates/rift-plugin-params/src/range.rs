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

// The constructors are `const fn` with their own asserts, and the derive emits the
// call inside a `const` block, so a bad `range = ...` is a compile error.
impl FloatRange {
    /// A linear range spanning `min..=max`.
    pub const fn linear(min: f32, max: f32) -> Self {
        assert!(min != max, "range min and max must differ");
        Self {
            min,
            max,
            scale: Scale::Linear,
        }
    }

    /// A power-curve range; `factor` must be finite and greater than 0.
    ///
    /// ```compile_fail
    /// const _: rift_plugin_params::FloatRange =
    ///     rift_plugin_params::FloatRange::skew(20.0, 20_000.0, -1.0);
    /// ```
    pub const fn skew(min: f32, max: f32, factor: f32) -> Self {
        assert!(min != max, "range min and max must differ");
        assert!(
            factor.is_finite() && factor > 0.0,
            "skew factor must be finite and greater than 0"
        );
        Self {
            min,
            max,
            scale: Scale::Skew(factor),
        }
    }

    /// An exponential-curve range; `factor` must be finite, greater than 0 and
    /// different from 1.
    pub const fn exp(min: f32, max: f32, factor: f32) -> Self {
        assert!(min != max, "range min and max must differ");
        assert!(
            factor.is_finite() && factor > 0.0 && factor != 1.0,
            "exp factor must be finite, greater than 0 and different from 1"
        );
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
    pub const fn linear(min: i32, max: i32) -> Self {
        assert!(min != max, "range min and max must differ");
        Self { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_are_const_and_checked() {
        // These `const` items prove the constructors are `const fn`, the derive
        // emits them inside a `const` block so a bad `range = ...` fails the build.
        const LINEAR: FloatRange = FloatRange::linear(-1.0, 1.0);
        const SKEW: FloatRange = FloatRange::skew(20.0, 20_000.0, 3.0);
        const EXP: FloatRange = FloatRange::exp(-60.0, 6.0, 2.0);
        const INT: IntRange = IntRange::linear(-12, 12);

        assert_eq!((LINEAR.min, LINEAR.max), (-1.0, 1.0));
        assert!(matches!(SKEW.scale, Scale::Skew(3.0)));
        assert!(matches!(EXP.scale, Scale::Exponential(2.0)));
        assert_eq!((INT.min, INT.max), (-12, 12));
    }
}
