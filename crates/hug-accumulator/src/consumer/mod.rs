pub trait AudioConsumer: Send + Sync + 'static {
    fn consume(&mut self, block: &[f32], channel_idx: usize, total_channels: usize);
}

impl AudioConsumer for () {
    fn consume(&mut self, _: &[f32], _: usize, _: usize) {}
}
