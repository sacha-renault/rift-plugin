//! Normalization curves: how a host/GUI position in `[0, 1]` maps to a plain
//! parameter value in `[min, max]`.
//!
//! [`Scale`] is only consulted on the normalized boundary ([`Param::normalize`] /
//! [`Param::denormalize`]); the DSP always sees plain values.
//!
//! # Adding a new curve
//!
//! Worked example: adding `Scale::Log(f32)`, reachable from the derive as
//! `range = log(min, max)`.
//!
//! 1. In this file (`scale.rs`), add the variant to [`Scale`] and handle it in
//!    [`Scale::denormalize`] and [`Scale::normalize`]. Keep them exact inverses
//!    (`normalize(denormalize(t)) == t`) and pin the endpoints
//!    (`denormalize(0.0) == min`, `denormalize(1.0) == max`). Cover it with
//!    `assert_scale_invariants` in the tests module, plus any shape specific check.
//! 2. In `range.rs`, add a matching `const fn` on `crate::FloatRange` (e.g.
//!    `FloatRange::log(min, max)`) with its own argument asserts. That is the
//!    whole DSL surface: the derive forwards `range = log(..)` to
//!    `<FloatRange|IntRange>::log(..)` inside a `const` block, so the asserts run
//!    at compile time.
//!
//! Nothing else needs to change: `expand_range` builds the call straight from the
//! DSL keyword, and `validate_leaf` only rejects `range` on params that have no
//! range type (bool, enum).

#[derive(Debug, Default, Clone, Copy)]
pub enum Scale {
    #[default]
    Linear,

    /// Power curve: more resolution at bottom (skew > 1) or top (skew < 1)
    Skew(f32),

    /// Exponential curve: `t = (factor^normalized - 1) / (factor - 1)` for
    /// `factor > 0` and `factor != 1`. More resolution at bottom (factor > 1) or
    /// top (factor < 1), and it accepts any range, including one that crosses
    /// zero (unlike a plain log curve).
    Exponential(f32),

    /// Log curve
    Log,
}

impl Scale {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let normalized = if normalized.is_nan() {
            0f32 // fallback in case the value is nan ...
        } else {
            normalized.clamp(0f32, 1f32)
        };

        match self {
            Self::Linear => linedenorm(normalized, min, max),
            Self::Skew(s) => linedenorm(normalized.powf(*s), min, max),
            Self::Exponential(f) => {
                let t = (f.powf(normalized) - 1f32) / (f - 1f32);
                linedenorm(t, min, max)
            }
            Self::Log => min * (max / min).powf(normalized),
        }
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        let value = if value.is_nan() {
            return 0f32; // fallback ...
        } else {
            value.clamp(min, max)
        };

        match self {
            Self::Linear => lineblend(value, min, max),
            Self::Skew(s) => lineblend(value, min, max).powf(1.0 / s),
            Self::Exponential(f) => (1f32 + lineblend(value, min, max) * (f - 1f32)).ln() / f.ln(),
            Self::Log => (value.ln() - min.ln()) / (max.ln() - min.ln()),
        }
    }
}

#[inline]
fn linedenorm(t: f32, min: f32, max: f32) -> f32 {
    min + t * (max - min)
}

#[inline]
fn lineblend(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

#[cfg(test)]
mod tests {
    use crate::assert_approx_eq;

    use super::*;

    fn assert_scale_invariants(scale: Scale, min: f32, max: f32) {
        let endpoint_tolerance = (max - min).abs() * 1e-6;
        let round_trip_tolerance = 1e-4;

        assert_approx_eq!(scale.denormalize(0.0, min, max), min, endpoint_tolerance);
        assert_approx_eq!(scale.denormalize(1.0, min, max), max, endpoint_tolerance);

        let mut previous = f32::NEG_INFINITY;
        for &normalized in &[0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
            let value = scale.denormalize(normalized, min, max);
            assert!(
                value >= previous,
                "curve is not increasing at {normalized}: {value} < {previous}"
            );
            previous = value;

            assert_approx_eq!(
                scale.normalize(value, min, max),
                normalized,
                round_trip_tolerance
            );
        }
    }

    #[test]
    fn linear_invariants() {
        assert_scale_invariants(Scale::Linear, 0.0, 100.0);
        assert_scale_invariants(Scale::Linear, -1.0, 1.0);

        // A linear scale is a straight line.
        assert_approx_eq!(Scale::Linear.denormalize(0.5, 0.0, 100.0), 50.0);
        assert_approx_eq!(Scale::Linear.normalize(25.0, 0.0, 100.0), 0.25);
    }

    #[test]
    fn skew_invariants() {
        for &factor in &[1.0 / 3.0, 1.0 / 2.0, 1.0, 2.0, 3.0] {
            assert_scale_invariants(Scale::Skew(factor), 20.0, 20000.0);
        }

        // A factor above 1 pushes the midpoint below the linear midpoint.
        assert!(Scale::Skew(3.0).denormalize(0.5, 0.0, 1000.0) < 500.0);
        // A factor below 1 pushes it above.
        assert!(Scale::Skew(0.3).denormalize(0.5, 0.0, 1000.0) > 500.0);
    }

    #[test]
    fn exponential_invariants() {
        for &factor in &[1.0 / 3.0, 1.0 / 2.0, 2.0, 3.0] {
            assert_scale_invariants(Scale::Exponential(factor), 20.0, 20000.0);
            // Unlike a plain log curve, it also works for a range that crosses zero.
            assert_scale_invariants(Scale::Exponential(factor), -60.0, 6.0);
        }

        // A factor above 1 pushes the midpoint below the linear midpoint.
        assert!(Scale::Exponential(3.0).denormalize(0.5, 0.0, 1000.0) < 500.0);
        // A factor below 1 pushes it above.
        assert!(Scale::Exponential(0.3).denormalize(0.5, 0.0, 1000.0) > 500.0);
    }

    #[test]
    fn log_invariants() {
        assert_scale_invariants(Scale::Log, 20.0, 20000.0);
        assert_approx_eq!(Scale::Log.denormalize(0.5, 20.0, 20000.0), 632.4555, 1e-2);
    }
}
