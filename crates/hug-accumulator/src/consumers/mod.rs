use hug_shared::{BlockTime, ChannelsInfo};

mod spectrogram;
mod windowed_peaks;

pub trait AudioConsumer: 'static {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}

pub use spectrogram::StftChannelConsumer;
pub use windowed_peaks::{WindowBuffer, WindowBufferMode};
