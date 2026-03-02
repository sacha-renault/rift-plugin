use crate::{BlockTime, accumulator::ChannelsInfo};

pub trait AudioConsumer: Send + Sync + 'static {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}
