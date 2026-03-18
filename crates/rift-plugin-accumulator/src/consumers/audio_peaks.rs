use std::cell::RefCell;
use std::rc::Rc;

use rift_plugin_shared::prelude::ConsumerCell;
use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};
use rift_plugin_shared::utils::interpo::lerp_n;

use crate::prelude::AudioConsumer;

/// Per-channel peak state tracked by [`AudioPeaks`].
struct ChannelAudioPeaks {
    /// The peak level, decay each block.
    true_peak: f32,
    /// An smoothed version of `true_peak`, used for display.
    smooth_peak: f32,
}

impl ChannelAudioPeaks {
    fn new() -> Self {
        Self {
            true_peak: 0.,
            smooth_peak: 0.,
        }
    }
}

/// An [`AudioConsumer`] that tracks true and smoothed peak levels per channel.
///
/// # Examples
///
/// ```rust
/// let mut peaks = AudioPeaks::new(2)
///     .lerp_factor(0.5)
///     .decay(|peak, block_size| peak * 0.999_f32.powi(block_size as i32));
///
/// // true_peak snaps immediately; smooth_peak trails behind
/// let true_peak  = peaks.true_peak(0);
/// let smooth_peak = peaks.peak(0);
/// ```
pub struct AudioPeaks {
    channel_peaks: Vec<ChannelAudioPeaks>,
    /// User-supplied decay function applied once per block to `true_peak`.
    /// Receives the current peak and the block length; returns the decayed peak.
    decay_fn: fn(f32, usize) -> f32,
    /// Pre-scaled lerp coefficient (user value × 1e-3) applied per sample via [`lerp_n`].
    lerp_factor: f32,
}

impl AudioPeaks {
    /// Creates a new `AudioPeaks` instance with `channels` independent peak trackers.
    ///
    /// Both `true_peak` and `smooth_peak` start at `0.0` for every channel.
    /// The default decay function is [`default_decay`] and the default lerp
    /// factor is `0.8e-3`.
    pub fn new(channels: usize) -> Self {
        let mut channel_peaks = Vec::new();
        channel_peaks.resize_with(channels, ChannelAudioPeaks::new);

        Self {
            channel_peaks,
            decay_fn: default_decay,
            lerp_factor: 0.8 * 1e-3,
        }
    }

    /// Sets the lerping factor for smooth audio level transitions.
    ///
    /// The provided value is scaled by 1e-3 internally. Since this function is called
    /// tons of times per second to update audio peaks, the raw lerp factor would be too large
    /// without this scaling.
    ///
    /// Higher values make `smooth_peak` follow `true_peak` more closely.
    pub fn lerp_factor(mut self, factor: f32) -> Self {
        self.lerp_factor = factor * 1e-3;
        self
    }

    /// Replaces the decay function applied to `true_peak` at the start of each block.
    ///
    /// The function receives `(current_peak, block_size)` and should return the
    /// decayed peak. The default is [`default_decay`], which multiplies by
    /// `0.9985 ^ block_size`.
    pub fn decay(mut self, func: fn(f32, usize) -> f32) -> Self {
        self.decay_fn = func;
        self
    }

    /// Returns the number of channels this instance was created with.
    pub fn num_channels(&self) -> usize {
        self.channel_peaks.len()
    }

    /// Returns the instantaneous (decayed) peak for `channel`, or `None` if
    /// `channel` is out of bounds.
    pub fn true_peak(&self, channel: usize) -> Option<f32> {
        self.channel_peaks.get(channel).map(|ch| ch.true_peak)
    }

    /// Returns the smoothed peak for `channel`, or `None` if `channel` is out
    /// of bounds.
    pub fn peak(&self, channel: usize) -> Option<f32> {
        self.channel_peaks.get(channel).map(|ch| ch.smooth_peak)
    }
}

impl AudioConsumer for AudioPeaks {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, _: BlockTime) {
        let Some(channel_peak) = self.channel_peaks.get(channel_info.current) else {
            return;
        };

        let &ChannelAudioPeaks {
            mut true_peak,
            mut smooth_peak,
        } = channel_peak;

        // Decay once for the whole block
        true_peak = (self.decay_fn)(true_peak, block.len());
        let block_peak = block
            .iter()
            .map(|v| v.abs())
            .fold(f32::NEG_INFINITY, f32::max);

        if block_peak > true_peak {
            true_peak = block_peak;
        }

        // Always lerp to true_peak
        smooth_peak = lerp_n(smooth_peak, true_peak, self.lerp_factor, block.len() as i32);
        self.channel_peaks[channel_info.current] = ChannelAudioPeaks {
            true_peak,
            smooth_peak,
        };
    }
}

fn default_decay(peak: f32, block_size: usize) -> f32 {
    peak * 0.9985_f32.powi(block_size as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_acc() -> AudioPeaks {
        AudioPeaks::new(1)
    }

    #[test]
    fn test_basic() {
        let mut acc = make_acc();
        acc.consume(
            &[1., 1., 1., 1.],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );

        assert!(acc.true_peak(0).unwrap() > acc.peak(0).unwrap());
        assert_eq!(acc.num_channels(), 1);
    }

    #[test]
    fn test_higher_lerp_factor() {
        let mut acc1 = make_acc();
        let mut acc2 = make_acc().lerp_factor(0.9);
        acc1.consume(
            &[1., 1., 1., 1.],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );
        acc2.consume(
            &[1., 1., 1., 1.],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );

        assert_eq!(acc1.true_peak(0).unwrap(), acc2.true_peak(0).unwrap());
        assert!(acc1.peak(0).unwrap() < acc2.peak(0).unwrap());
    }

    #[test]
    fn test_higher_lerp_decay() {
        let mut acc1 = make_acc();
        let mut acc2 = make_acc().decay(|peak, block_size| peak * 0.9f32.powi(block_size as i32));
        acc1.consume(
            &[0.5, 0.5],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );
        acc2.consume(
            &[0.5, 0.5],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );

        acc1.consume(
            &[0.0, 0.0],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );
        acc2.consume(
            &[0.0, 0.0],
            ChannelsInfo {
                current: 0,
                total_channels: 1,
            },
            BlockTime::none(),
        );

        assert!(acc1.true_peak(0).unwrap() > acc2.true_peak(0).unwrap());
    }

    #[test]
    fn test_out_of_bounds() {
        let mut acc = make_acc();
        acc.consume(
            &[0.0, 0.0],
            ChannelsInfo {
                current: 1,
                total_channels: 1,
            },
            BlockTime::none(),
        );

        assert_eq!(acc.peak(0), Some(0.))
    }
}
