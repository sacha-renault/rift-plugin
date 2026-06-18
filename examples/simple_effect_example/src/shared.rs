use rift_plugin::prelude::*;
use rift_plugin_accumulator::prelude::AudioAccumulator;

pub const BLOCK_SIZE: usize = 128;

pub struct Shared {
    pub post_acc: AudioAccumulator,
    pub lfo_position: AtomicF32,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            post_acc: AudioAccumulator::new::<BLOCK_SIZE>(2, 500),
            lfo_position: AtomicF32::new(0.),
        }
    }
}
