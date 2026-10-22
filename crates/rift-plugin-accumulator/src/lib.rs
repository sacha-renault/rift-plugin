mod accumulator;
mod consumers;

pub mod prelude {
    use super::*;

    pub use rift_plugin_utils::ConsumerCell;

    pub use accumulator::AudioAccumulator;
    pub use consumers::*;
}
