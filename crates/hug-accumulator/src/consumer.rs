use hug_shared::{BlockTime, ChannelsInfo};

pub trait AudioConsumer: 'static {
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}
