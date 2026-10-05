//! Normalization curves: how a host/GUI position in `[0, 1]` maps to a plain
//! parameter value in `[min, max]`.
//!
//! [`Scale`] is only consulted on the normalized boundary ([`Param::normalize`] /
//! [`Param::denormalize`]); the DSP always sees plain values.
//!
//! # Adding a new curve
//!
//! Worked example: adding `Scale::Log(f32)`, reachable from the derive as
//! `range = log(min, max, base)`.
//!
//! 1. In this file (`scale.rs`), add the variant to [`Scale`] and handle it in
//!    [`Scale::denormalize`] and [`Scale::normalize`]. Keep them exact inverses
//!    (`normalize(denormalize(t)) == t`) and pin the endpoints
//!    (`denormalize(0.0) == min`, `denormalize(1.0) == max`). Cover it with
//!    `assert_scale_invariants` in the tests module, plus any shape specific check.
//! 2. In `rift-plugin-derive/src/params/parse.rs`, mirror the variant in
//!    `ScaleArg`, same shape but carrying the parsed argument as an `Expr`.
//! 3. In that same file, add one row to `RANGE_VARIANTS`: `name` (the DSL keyword,
//!    e.g. `"log"`), `extra_args` (each argument after the implicit `min, max`,
//!    e.g. `&["base"]`) and `build` (turns the extra args into the `ScaleArg`).
//! 4. In `rift-plugin-derive/src/params/expand.rs`, map the new `ScaleArg` to its
//!    `::rift_plugin::prelude::Scale::*` expression. Today this is an irrefutable
//!    `let` (a single variant); a second variant makes it refutable, so it becomes
//!    a `match`. The compiler will flag it.
//! 5. Add a derive test (`.../params/tests.rs`) that the form expands, plus one
//!    that a param which is not float rejects it.
//!
//! Nothing else needs to change: `validate_leaf` already rejects any curve on
//! params that are not float, and the "expected" error text rebuilds itself from
//! the table.
//!
//! If the curve constrains its argument (e.g. `base > 0`), validate it.

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
}

impl Scale {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let normalized = if normalized.is_nan() {
            0f32 // fallback in case the value is nan ...
        } else {
            normalized.clamp(0f32, 1f32)
        };

        let t = match self {
            Self::Linear => normalized,
            Self::Skew(s) => normalized.powf(*s),
            Self::Exponential(f) => (f.powf(normalized) - 1f32) / (f - 1f32),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        if value.is_nan() {
            return 0f32; // fallback ...
        }

        let t = (value - min) / (max - min);

        match self {
            Self::Linear => t,
            Self::Skew(s) => t.powf(1.0 / s),
            Self::Exponential(f) => (1f32 + t * (f - 1f32)).ln() / f.ln(),
        }
    }
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
}
