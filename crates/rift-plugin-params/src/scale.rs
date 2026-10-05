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
//!    (`denormalize(0.0) == min`, `denormalize(1.0) == max`). Add a math test
//!    (round trip plus endpoints).
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
}

impl Scale {
    pub fn denormalize(&self, normalized: f32, min: f32, max: f32) -> f32 {
        let value = if normalized.is_nan() {
            0f32 // fallback in case the value is nan ...
        } else {
            normalized.clamp(0f32, 1f32)
        };

        let t = match self {
            Self::Linear => value,
            Self::Skew(s) => value.powf(*s),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        if value.is_nan() {
            return 0f32; // fallback ...
        }

        match self {
            Self::Linear => (value - min) / (max - min),
            Self::Skew(s) => ((value - min) / (max - min)).powf(1.0 / s),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_approx_eq;

    use super::*;

    #[test]
    fn skew_range_mapping() {
        // Linear sanity check
        let linear = Scale::Linear;
        assert_approx_eq!(linear.denormalize(0.0, 0.0, 100.0), 0.0);
        assert_approx_eq!(linear.denormalize(0.5, 0.0, 100.0), 50.0);
        assert_approx_eq!(linear.denormalize(1.0, 0.0, 100.0), 100.0);

        // Skew: endpoints should always map exactly
        let skew = Scale::Skew(3.0);
        assert_approx_eq!(skew.denormalize(0.0, 20.0, 20000.0), 20.0);
        assert_approx_eq!(skew.denormalize(1.0, 20.0, 20000.0), 20000.0);

        // Skew > 1: midpoint should map below the linear midpoint
        let mid = skew.denormalize(0.5, 0.0, 1000.0);
        assert!(mid < 500.0);

        // Skew < 1: midpoint should map above the linear midpoint
        let skew_inv = Scale::Skew(0.3);
        let mid_inv = skew_inv.denormalize(0.5, 0.0, 1000.0);
        assert!(mid_inv > 500.0);

        // Roundtrip: normalize(denormalize(x)) == x
        for &s in &[0.3_f32, 1.0, 2.0, 3.0] {
            let mapping = Scale::Skew(s);
            for &n in &[0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
                let value = mapping.denormalize(n, 20.0, 20000.0);
                let back = mapping.normalize(value, 20.0, 20000.0);
                assert_approx_eq!(back, n, 1e-5);
            }
        }
    }
}
