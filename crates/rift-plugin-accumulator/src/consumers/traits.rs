use std::{cell::RefCell, rc::Rc};

use rift_plugin_core::prelude::{BlockTime, ChannelsInfo, ConsumerCell, MultiChannel};

/// A consumer that receives a single-channel audio block.
///
/// Used by [`ConsumerDispatcher`](crate::prelude::ConsumerDispatcher) for
/// averaged and single-channel routing modes, where the consumer doesn't
/// need to know about channel layout.
///
/// Every [`MultiConsumer`] automatically implements this trait via a blanket
/// impl, receiving [`ChannelsInfo::mono()`] as channel context.
pub trait MonoConsumer: 'static {
    /// Processes one block of PCM samples.
    ///
    /// - `block` - f32 samples for a single channel.
    ///   Length may be less than `N` for the final chunk of a render cycle.
    /// - `time` - transport position at the start of this block, or
    ///   [`BlockTime::none`] if timing information was unavailable.
    fn consume(&mut self, block: &[f32], time: BlockTime);

    fn wraps_consumer(self) -> ConsumerCell<Self>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
    }
}

/// A consumer that receives per-channel audio blocks with channel context.
///
/// Used by [`ConsumerDispatcher`](crate::prelude::ConsumerDispatcher) for
/// the all-channels routing mode, where the consumer needs to distinguish
/// between channels (e.g. per-channel peak meters).
///
/// Implementors automatically get a [`MonoConsumer`] impl for free, so a
/// `MultiConsumer` can be registered in any consumer slot.
pub trait MultiConsumer: 'static {
    /// Processes one block of PCM samples for a specific channel.
    ///
    /// - `block` - f32 samples for the current channel.
    ///   Length may be less than `N` for the final chunk of a render cycle.
    /// - `channel_info` - identifies which channel this block belongs to
    ///   and how many channels are in the bus in total.
    /// - `time` - transport position at the start of this block, or
    ///   [`BlockTime::none`] if timing information was unavailable.
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);

    fn wraps_consumer(self) -> ConsumerCell<Self>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
    }
}

impl<T> MultiConsumer for MultiChannel<T>
where
    T: MonoConsumer,
{
    fn consume(&mut self, block: &[f32], info: ChannelsInfo, time: BlockTime) {
        let channel_idx = info.current;

        #[cfg(debug_assertions)]
        self.with_channel_mut(channel_idx, |consumer| consumer.consume(block, time));

        #[cfg(not(debug_assertions))]
        self.try_with_channel_mut(channel_idx, |consumer| consumer.consume(block, time));
    }
}
