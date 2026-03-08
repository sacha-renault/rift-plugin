use hug_shared::{BlockTime, ChannelsInfo};

mod spectrogram;

pub trait AudioConsumer: 'static {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}

pub use spectrogram::StftChannelConsumer;
