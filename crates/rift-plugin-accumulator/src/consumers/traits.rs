use rift_plugin_types::transport::{BlockTime, ChannelsInfo};
use rift_plugin_utils::MultiChannel;

/// A consumer that receives a single-channel audio block.
///
/// Used by [`ConsumerDispatcher`](crate::prelude::ConsumerDispatcher) for
/// averaged and single-channel routing modes, where the consumer doesn't
/// need to know about channel layout.
///
/// For one independent instance per channel, wrap it in
/// [`MultiChannel<T>`](rift_plugin_utils::MultiChannel), which implements
/// [`MultiConsumer`].
pub trait MonoConsumer: 'static {
    /// Processes one block of PCM samples.
    ///
    /// - `block` - f32 samples for a single channel. Any length, possibly empty.
    /// - `time` - timing of `block[0]`. Use [`BlockTime::advanced_by`] for
    ///   later samples. `time.transport` is `None` if the host provided none.
    ///   A gap in `time.sample_pos` between two calls means samples were
    ///   dropped (e.g. the GUI was closed).
    fn consume(&mut self, block: &[f32], time: BlockTime);
}

/// A consumer that receives per-channel audio blocks with channel context.
///
/// Used by [`ConsumerDispatcher`](crate::prelude::ConsumerDispatcher) for
/// the all-channels routing mode, where the consumer needs to distinguish
/// between channels (e.g. per-channel peak meters).
///
/// Channels of the same block are delivered in ascending order, all with the
/// same `block` length and `time`.
pub trait MultiConsumer: 'static {
    /// Processes one block of PCM samples for a specific channel.
    ///
    /// - `block` - f32 samples for the current channel. Any length, possibly empty.
    /// - `channel_info` - identifies which channel this block belongs to
    ///   and how many channels are in the bus in total.
    /// - `time` - timing of `block[0]`, see [`MonoConsumer::consume`].
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
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
