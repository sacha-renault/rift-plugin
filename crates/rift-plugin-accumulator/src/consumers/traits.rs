use rift_plugin_core::prelude::{BlockTime, ChannelsInfo};

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
}

impl<T> MonoConsumer for T
where
    T: MultiConsumer,
{
    fn consume(&mut self, block: &[f32], time: BlockTime) {
        self.consume(block, ChannelsInfo::mono(), time);
    }
}
