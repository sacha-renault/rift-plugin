use crate::AudioConsumer;
use hug_shared::{BlockTime, ChannelsInfo, utils::interpo::lerp_n};

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
            lerp_factor: 0.8,
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
