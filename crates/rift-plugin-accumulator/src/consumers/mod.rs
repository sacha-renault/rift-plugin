use rift_plugin_shared::transport::{BlockTime, ChannelsInfo};

mod audio_peaks;
mod spectrogram;
mod windowed_peaks;

pub trait AudioConsumer: 'static {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}

pub use audio_peaks::AudioPeaks;
pub use spectrogram::StftChannelConsumer;
pub use windowed_peaks::{Bucket, WindowBuckets, WindowBufferMode};
