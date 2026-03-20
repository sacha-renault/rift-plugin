use std::sync::Arc;

use rift_plugin_core::prelude::*;

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

    /// Copies each slice from `slices` into the corresponding channel's block ring.
    ///
    /// Channels and slices are paired by position: the first slice goes into
    /// channel 0, the second into channel 1, and so on. Extra slices beyond
    /// the number of allocated channels are silently ignored; channels with no
    /// corresponding slice receive nothing.
    ///
    /// `block_info_opt` is forwarded to every block. Pass `Some` when transport
    /// timing information is available, `None` otherwise.
    ///
    /// The `num_writes` counter is incremented **once** per call regardless of
    /// how many channels were actually written, so consumers can use it as a
    /// per-render-cycle dirty flag.
    ///
    /// # Real-time safety
    ///
    /// This function is lock-free and allocation-free and may be called from
    /// the audio thread.
    fn push_slices<'a>(
        &self,
        slices: &mut dyn Iterator<Item = &'a [f32]>,
        block_info_opt: Option<BlockInfo>,
    );

    /// Returns the current write counter.
    ///
    /// it is intended as a lightweight change-detection signal for the UI thread
    fn num_writes(&self) -> u64;

    /// Drains all pending blocks from every channel and dispatches them to each consumer.
    ///
    /// Blocks are consumed one full round at a time: one block is popped from
    /// each channel in index order before moving to the next round. If any
    /// channel runs out of blocks mid-round, [`Self::clear`] is called to flush
    /// the remaining channels and the drain loop exits, keeping all channels
    /// in sync.
    ///
    /// Each block is delivered to every registered consumer via
    /// [`AudioConsumer::consume`], tagged with a [`ChannelsInfo`] describing the
    /// channel's index and the total channel count. Consumers that cannot be
    /// borrowed (because they are already mutably borrowed elsewhere) are
    /// silently skipped for that block.
    ///
    /// # Thread safety
    ///
    /// Intended for the UI thread only.
    fn drain(&self, consumers: &[ConsumerCell<dyn AudioConsumer>]);

    /// Discards all buffered blocks across every channel.
    ///
    /// Called automatically by [`Self::drain`] when channels fall out of sync, but
    /// can also be called directly to reset the accumulator to a clean state
    /// (e.g. on transport stop or plugin deactivation).
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
/// ```ignore
/// // Create an accumulator with up to 2 channels and a queue depth of 8 blocks.
/// // N=512 sets the internal frame-block size at compile time.
/// let accumulator = AudioAccumulator::new::<512>(2, 8);
///
/// // The handle is cheap to clone - both point to the same queue.
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
    /// * `N` - the compile-time frame-block size used internally by the accumulator.
    ///   This value is erased after construction; choose it to match your audio engine's
    ///   maximum block size.
    ///
    /// # Arguments
    ///
    /// * `max_channels` - number of audio channels to allocate buffers for.
    /// * `max_block_in_queue` - maximum number of audio blocks that can be buffered
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
    fn drain(&self, consumers: &[ConsumerCell<dyn AudioConsumer>]) {
        self.inner.drain(consumers);
    }
}
