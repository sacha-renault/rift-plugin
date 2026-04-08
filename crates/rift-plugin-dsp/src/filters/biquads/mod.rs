//! Butterworth biquad filters with cascading support.
//!
//! Entry points: [`BiquadCascade`] for filtering, [`FilterMode`] and [`FilterOrder`] for configuration.
//! `https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html`

mod biquad_args;
mod biquad_cascade;
mod biquad_filter;
mod biquad_qs;
mod utils;

pub use biquad_args::{FilterMode, FilterOrder};
pub use biquad_cascade::BiquadCascade;
