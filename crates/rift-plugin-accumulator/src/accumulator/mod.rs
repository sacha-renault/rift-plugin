use std::sync::Arc;

use rift_plugin_shared::{RcCell, transport::BlockInfo};

use crate::prelude::*;

mod channel;
mod inner;

#[cfg(test)]
mod tests;

mod private {
    pub trait Sealed {}
}

/// A trait to erase the const N parameter of [`inner::InnerAudioAccumulator`]
pub trait AudioAccumulatorErased: private::Sealed + Send + Sync + 'static {
    /// Number of channel allocated for this [`AudioAccumulator`]
    fn channels(&self) -> usize;

    /// This function is totally lock free, audio thread
    /// can push slices here with no lock, mutex guard or alloc
    fn push_slices<'a>(
        &self,
        slices: &mut dyn Iterator<Item = &'a [f32]>,
        block_info_opt: Option<BlockInfo>,
    );

    /// Return the number of writes that occures on the audio accumulator since
    /// the beginning. Used to check when there is new data coming in the accumulator.
    fn num_writes(&self) -> u64;

    /// This function is meant to be called on the UI thread
    /// locks are fine here
    fn drain(&self, consumers: &[RcCell<dyn AudioConsumer>]);

    /// Clear all buffers in the accumulator
    fn clear(&self);
}

#[derive(Clone)]
pub struct AudioAccumulator {
    inner: Arc<dyn AudioAccumulatorErased>,
}

impl AudioAccumulator {
    pub fn new<const N: usize>(max_channels: usize, max_block_in_queue: usize) -> Self {
        Self {
            inner: Arc::new(inner::InnerAudioAccumulator::<N>::new(
                max_channels,
                max_block_in_queue,
            )),
        }
    }
}

impl private::Sealed for AudioAccumulator {}

impl AudioAccumulatorErased for AudioAccumulator {
    #[inline]
    fn channels(&self) -> usize {
        self.inner.channels()
    }

    #[inline]
    fn clear(&self) {
        self.inner.clear()
    }

    #[inline]
    fn num_writes(&self) -> u64 {
        self.inner.num_writes()
    }

    #[inline]
    fn push_slices<'a>(
        &self,
        slices: &mut dyn Iterator<Item = &'a [f32]>,
        block_info_opt: Option<BlockInfo>,
    ) {
        self.inner.push_slices(slices, block_info_opt);
    }

    #[inline]
    fn drain(&self, consumers: &[RcCell<dyn AudioConsumer>]) {
        self.inner.drain(consumers);
    }
}
