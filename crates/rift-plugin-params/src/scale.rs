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
        let t = match self {
            Self::Linear => normalized,
            Self::Skew(s) => normalized.powf(*s),
        };
        min + t * (max - min)
    }

    pub fn normalize(&self, value: f32, min: f32, max: f32) -> f32 {
        match self {
            Self::Linear => (value - min) / (max - min),
            Self::Skew(s) => ((value - min) / (max - min)).powf(1.0 / s),
        }
    }
}

#[cfg(test)]
mod tests {}
