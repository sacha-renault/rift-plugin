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

/// A cheaply cloneable handle to a lock-free audio accumulation queue.
///
/// `AudioAccumulator` wraps an [`Arc`]-backed, type-erased [`AudioAccumulatorErased`]
/// implementation, allowing it to be shared freely across threads without additional
/// synchronization. Cloning produces a new handle to the **same** underlying accumulator.
///
/// The const parameter `N` controls the internal capacity at construction time but is
/// erased after [`AudioAccumulator::new`] returns, so all handles share a uniform type
/// regardless of capacity.
///
/// # Thread safety
///
/// The push path ([`AudioAccumulatorErased::push_slices`]) is entirely lock-free and
/// allocation-free, making it safe to call from a real-time audio thread. The drain
/// path ([`AudioAccumulatorErased::drain`]) may take locks and is intended for the UI
/// thread only.
///
/// # Examples
///
/// ```rust
/// // Create an accumulator with up to 2 channels and a queue depth of 8 blocks.
/// // N=512 sets the internal frame-block size at compile time.
/// let accumulator = AudioAccumulator::new::<512>(2, 8);
///
/// // The handle is cheap to clone — both point to the same queue.
/// let accumulator_ui = accumulator.clone();
/// ```
#[derive(Clone)]
pub struct AudioAccumulator {
    inner: Arc<dyn AudioAccumulatorErased>,
}

impl AudioAccumulator {
    /// Creates a new `AudioAccumulator` backed by an [`InnerAudioAccumulator<N>`].
    ///
    /// # Type parameters
    ///
    /// * `N` — the compile-time frame-block size used internally by the accumulator.
    ///   This value is erased after construction; choose it to match your audio engine's
    ///   maximum block size.
    ///
    /// # Arguments
    ///
    /// * `max_channels` — number of audio channels to allocate buffers for.
    /// * `max_block_in_queue` — maximum number of audio blocks that can be buffered
    ///   before the audio thread drop data.
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
