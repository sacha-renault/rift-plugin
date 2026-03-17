use crate::AudioConsumer;
use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};
use rift_plugin_shared::utils::interpo::lerp_n;

struct ChannelAudioPeaks {
    true_peak: f32,
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

pub struct AudioPeaks {
    channel_peaks: Vec<ChannelAudioPeaks>,
    decay_fn: fn(f32, usize) -> f32,
    lerp_factor: f32,
}

impl AudioPeaks {
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
    pub fn lerp_factor(mut self, factor: f32) -> Self {
        self.lerp_factor = factor * 1e-3;
        self
    }

    pub fn set_decay(mut self, func: fn(f32, usize) -> f32) -> Self {
        self.decay_fn = func;
        self
    }

    pub fn num_channels(&self) -> usize {
        self.channel_peaks.len()
    }

    pub fn true_peak(&self, channel: usize) -> Option<f32> {
        self.channel_peaks.get(channel).map(|ch| ch.true_peak)
    }

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
        let mut acc2 =
            make_acc().set_decay(|peak, block_size| peak * 0.9f32.powi(block_size as i32));
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
