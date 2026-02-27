pub trait AudioConsumer: Send + Sync + 'static {
    fn consume(&mut self, block: &[f32], channel_idx: usize, total_channels: usize);
}
