use rift_plugin_core::transport::BlockTime;
use rift_plugin_core::utils::interpo::lerp_n;

use crate::prelude::MonoConsumer;

/// Tracks true and smoothed peak levels for a single audio channel.
///
/// `true_peak` captures the highest absolute sample value and decays each
/// block via a configurable decay function. `smooth_peak` trails behind
/// using per-sample linear interpolation, producing a value suitable for
/// display (e.g. a meter).
///
/// For multi-channel use, wrap in [`MultiChannel<AudioPeak>`](rift_plugin_core::prelude::MultiChannel).
///
/// # Examples
///
/// ```ignore
/// let mut peaks = AudioPeaks::new()
///     .lerp_factor(0.5)
///     .decay(|peak, block_size| peak * 0.999_f32.powi(block_size as i32));
///
/// let true_peak  = peaks.true_peak();
/// let smooth_peak = peaks.peak();
/// ```
pub struct AudioPeak {
    /// The instantaneous peak level, decayed once per block.
    true_peak: f32,
    /// A smoothed version of `true_peak`, suitable for display.
    smooth_peak: f32,
    /// User-supplied decay function applied once per block to `true_peak`.
    /// Receives the current peak and the block length; returns the decayed peak.
    decay_fn: fn(f32, usize) -> f32,
    /// Pre-scaled lerp coefficient (user value * 1e-3) applied per sample
    /// via [`lerp_n`].
    lerp_factor: f32,
}

impl Default for AudioPeak {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPeak {
    /// Creates a new `AudioPeaks` with both peaks starting at `0.0`.
    ///
    /// The default decay function is [`default_decay`] (`0.9985 ^ block_size`)
    /// and the default lerp factor is `0.8e-3`.
    pub fn new() -> Self {
        Self {
            true_peak: 0.,
            smooth_peak: 0.,
            decay_fn: default_decay,
            lerp_factor: 0.8 * 1e-3,
        }
    }

    /// Sets the lerp factor for smooth peak transitions.
    ///
    /// The provided value is scaled by 1e-3 internally. Higher values make
    /// `smooth_peak` follow `true_peak` more closely.
    pub fn lerp_factor(mut self, factor: f32) -> Self {
        self.lerp_factor = factor * 1e-3;
        self
    }

    /// Replaces the decay function applied to `true_peak` at the start of
    /// each block.
    ///
    /// The function receives `(current_peak, block_size)` and should return
    /// the decayed peak. The default multiplies by `0.9985 ^ block_size`.
    pub fn decay(mut self, func: fn(f32, usize) -> f32) -> Self {
        self.decay_fn = func;
        self
    }

    /// Returns the instantaneous (decayed) peak.
    pub fn true_peak(&self) -> f32 {
        self.true_peak
    }

    /// Returns the smoothed peak, suitable for driving a level meter.
    pub fn peak(&self) -> f32 {
        self.smooth_peak
    }
}

impl MonoConsumer for AudioPeak {
    fn consume(&mut self, block: &[f32], _: BlockTime) {
        self.true_peak = (self.decay_fn)(self.true_peak, block.len());

        let block_peak = block
            .iter()
            .map(|v| v.abs())
            .fold(f32::NEG_INFINITY, f32::max);

        if block_peak > self.true_peak {
            self.true_peak = block_peak;
        }

        self.smooth_peak = lerp_n(
            self.smooth_peak,
            self.true_peak,
            self.lerp_factor,
            block.len() as i32,
        );
    }
}

fn default_decay(peak: f32, block_size: usize) -> f32 {
    peak * 0.9985_f32.powi(block_size as i32)
}
