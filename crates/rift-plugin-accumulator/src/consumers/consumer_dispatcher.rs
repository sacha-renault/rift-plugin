use rift_plugin_types::transport::{BlockTime, ChannelsInfo};
use rift_plugin_utils::ConsumerCell;

use crate::prelude::{MonoConsumer, MultiConsumer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonoConsumerKind {
    Averaged,
    Channel(usize),
}

/// A consumer that only receives one channel.
struct ChannelConsumer {
    consumer: ConsumerCell<dyn MonoConsumer>,
    channel: usize,
}

/// Routes multi-channel audio blocks to registered consumers.
///
/// This struct centralises the channel-reduction logic that would otherwise be
/// duplicated inside every consumer. There are three routing modes, one per
/// `add_consumer_*` method:
///
/// - **averaged** - a [`MonoConsumer`] receives the mean of all channels, once
///   the last channel of a block has been dispatched.
/// - **all** - a [`MultiConsumer`] receives every channel with its [`ChannelsInfo`].
/// - **at channel** - a [`MonoConsumer`] receives a single channel unchanged.
///
/// Each `add_consumer_*` takes the consumer **by value**, wraps it, and returns
/// the [`ConsumerCell`] it registered, so the caller (typically the UI) keeps a
/// clone to read the results.
///
/// # Example
///
/// ```ignore
/// let mut dispatcher = ConsumerDispatcher::new();
/// let vu = dispatcher.add_consumer_averaged(my_vu_meter); // keep `vu` to read levels
/// dispatcher.add_consumer_at_channel(my_stft, 0);
///
/// // In the drain loop (called from the UI thread):
/// for ch in 0..total_channels {
///     dispatcher.dispatch(block, channels_info, time);
/// }
/// ```
pub struct ConsumerDispatcher {
    /// Scratch buffer used to accumulate the channel average.
    /// Only used when at least one averaged consumer is registered.
    intermediate: Vec<f32>,

    /// Registered consumers
    all_consumers: Vec<ConsumerCell<dyn MultiConsumer>>,
    avg_consumers: Vec<ConsumerCell<dyn MonoConsumer>>,
    channel_consumers: Vec<ChannelConsumer>,
}

impl Default for ConsumerDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumerDispatcher {
    /// Creates an empty dispatcher with no consumers registered.
    pub fn new() -> Self {
        Self {
            intermediate: Vec::new(),
            avg_consumers: Vec::new(),
            all_consumers: Vec::new(),
            channel_consumers: Vec::new(),
        }
    }

    /// Registers a consumer that will receive a mono-averaged block once per
    /// audio callback, after all channels have been dispatched.
    ///
    /// Returns the [`ConsumerCell`] the dispatcher writes into.
    pub fn add_consumer_averaged<C: MonoConsumer>(&mut self, consumer: C) -> ConsumerCell<C> {
        let cell = consumer.wraps_consumer();
        self.avg_consumers.push(cell.clone());
        cell
    }

    /// Registers a consumer that will receive a every channel independently.
    ///
    /// Returns the [`ConsumerCell`] the dispatcher writes into.
    pub fn add_consumer_all<C: MultiConsumer>(&mut self, consumer: C) -> ConsumerCell<C> {
        let cell = consumer.wraps_consumer();
        self.all_consumers.push(cell.clone());
        cell
    }

    /// Registers a consumer that will receive the raw block for `channel` only.
    /// Blocks for all other channels are silently ignored.
    ///
    /// Returns the [`ConsumerCell`] the dispatcher writes into.
    pub fn add_consumer_at_channel<C: MonoConsumer>(
        &mut self,
        consumer: C,
        channel: usize,
    ) -> ConsumerCell<C> {
        let cell = consumer.wraps_consumer();
        self.channel_consumers.push(ChannelConsumer {
            consumer: cell.clone(),
            channel,
        });
        cell
    }

    /// Add a mono consumer, either `Averaged` or at a specific `Channel`
    pub fn add_mono_consumer<C: MonoConsumer>(
        &mut self,
        consumer: C,
        kind: MonoConsumerKind,
    ) -> ConsumerCell<C> {
        match kind {
            MonoConsumerKind::Averaged => self.add_consumer_averaged(consumer),
            MonoConsumerKind::Channel(channel) => self.add_consumer_at_channel(consumer, channel),
        }
    }

    /// Returns the total number of registered consumers (both averaged and
    /// channel-specific).
    pub fn consumer_count(&self) -> usize {
        self.all_consumers.len() + self.avg_consumers.len() + self.channel_consumers.len()
    }

    /// Routes `block` to the appropriate consumers based on the current channel.
    ///
    /// Must be called once per channel per block, in ascending channel order,
    /// with the same block length for every channel.
    ///
    /// A consumer whose [`ConsumerCell`] is already borrowed (e.g. the UI holds
    /// a `borrow()` across the dispatch) panics in debug builds and is skipped
    /// in release builds.
    pub fn dispatch(&mut self, block: &[f32], channels: ChannelsInfo, time: BlockTime) {
        let total_channel = channels.total_channels as f32;

        // At channel 0, we ensure our intermediate buffer is large enough
        if !self.avg_consumers.is_empty() {
            if channels.current == 0 {
                self.intermediate.clear();
                self.intermediate.resize(block.len(), 0.);
            }

            for (s, &v) in self.intermediate.iter_mut().zip(block.iter()) {
                *s += v / total_channel;
            }
        }

        for consumer in &self.all_consumers {
            with_consumer(consumer, |c| c.consume(block, channels, time));
        }

        for ChannelConsumer { consumer, channel } in &self.channel_consumers {
            if channels.current == *channel {
                with_consumer(consumer, |c| c.consume(block, time));
            }
        }

        if channels.is_last_channel() {
            for consumer in &self.avg_consumers {
                with_consumer(consumer, |c| c.consume(&self.intermediate, time));
            }
        }
    }
}

/// Runs `f` on the consumer. Panics on debug builds and
/// just ignore on release builds.
fn with_consumer<T: ?Sized>(consumer: &ConsumerCell<T>, f: impl FnOnce(&mut T)) {
    match consumer.try_borrow_mut() {
        Ok(mut c) => f(&mut c),
        Err(_) => debug_assert!(false, "consumer is already borrowed during dispatch"),
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_types::transport::{BlockTime, ChannelsInfo};

    use crate::prelude::{MonoConsumer, MultiConsumer};

    use super::*;

    struct MonoMock {
        calls: Vec<Vec<f32>>,
    }

    impl MonoMock {
        fn new() -> Self {
            MonoMock { calls: Vec::new() }
        }
    }

    impl MonoConsumer for MonoMock {
        fn consume(&mut self, block: &[f32], _time: BlockTime) {
            self.calls.push(block.to_vec());
        }
    }

    struct MultiMockCall {
        data: Vec<f32>,
        channel: usize,
        total_channels: usize,
    }

    struct MultiMock {
        calls: Vec<MultiMockCall>,
    }

    impl MultiMock {
        fn new() -> Self {
            MultiMock { calls: Vec::new() }
        }
    }

    impl MultiConsumer for MultiMock {
        fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, _time: BlockTime) {
            self.calls.push(MultiMockCall {
                data: block.to_vec(),
                channel: channel_info.current,
                total_channels: channel_info.total_channels,
            });
        }
    }

    fn ch(current: usize, total: usize) -> ChannelsInfo {
        ChannelsInfo {
            current,
            total_channels: total,
        }
    }

    fn dispatch_stereo(dispatcher: &mut ConsumerDispatcher, left: &[f32], right: &[f32]) {
        dispatcher.dispatch(left, ch(0, 2), BlockTime::none());
        dispatcher.dispatch(right, ch(1, 2), BlockTime::none());
    }

    #[test]
    fn channel_mode_receives_correct_channel() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_at_channel(MonoMock::new(), 1);

        let left = vec![0.0_f32; 4];
        let right = vec![1.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 1);
        assert_eq!(c.calls[0], right);
    }

    #[test]
    fn channel_mode_ignores_wrong_channel() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_at_channel(MonoMock::new(), 1);

        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert_eq!(consumer.borrow().calls.len(), 0);
    }

    #[test]
    fn averaged_mode_produces_mean_of_channels() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_averaged(MonoMock::new());

        let left = vec![1.0_f32; 4];
        let right = vec![0.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 1);
        assert_eq!(c.calls[0], vec![0.5_f32; 4]);
    }

    #[test]
    fn averaged_mode_not_called_until_last_channel() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_averaged(MonoMock::new());

        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert_eq!(consumer.borrow().calls.len(), 0);
    }

    #[test]
    fn averaged_mode_mono_passes_through() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_averaged(MonoMock::new());

        let block = vec![0.7_f32; 4];
        d.dispatch(&block, ch(0, 1), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 1);
        assert_eq!(c.calls[0], block);
    }

    #[test]
    fn all_mode_receives_every_channel() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_all(MultiMock::new());

        let left = vec![1.0_f32; 4];
        let right = vec![2.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 2);
        assert_eq!(c.calls[0].channel, 0);
        assert_eq!(c.calls[0].data, left);
        assert_eq!(c.calls[1].channel, 1);
        assert_eq!(c.calls[1].data, right);
    }

    #[test]
    fn all_mode_forwards_channel_info() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_all(MultiMock::new());

        d.dispatch(&[0.0; 4], ch(0, 3), BlockTime::none());
        d.dispatch(&[0.0; 4], ch(1, 3), BlockTime::none());
        d.dispatch(&[0.0; 4], ch(2, 3), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 3);
        for (i, call) in c.calls.iter().enumerate() {
            assert_eq!(call.channel, i);
            assert_eq!(call.total_channels, 3);
        }
    }

    #[test]
    fn mixed_modes_all_receive_correct_data() {
        let mut d = ConsumerDispatcher::new();
        let avg_consumer = d.add_consumer_averaged(MonoMock::new());
        let ch0_consumer = d.add_consumer_at_channel(MonoMock::new(), 0);
        let all_consumer = d.add_consumer_all(MultiMock::new());

        let left = vec![1.0_f32; 4];
        let right = vec![0.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let avg = avg_consumer.borrow();
        assert_eq!(avg.calls.len(), 1);
        assert_eq!(avg.calls[0], vec![0.5_f32; 4]);

        let ch0 = ch0_consumer.borrow();
        assert_eq!(ch0.calls.len(), 1);
        assert_eq!(ch0.calls[0], left);

        let all = all_consumer.borrow();
        assert_eq!(all.calls.len(), 2);
        assert_eq!(all.calls[0].data, left);
        assert_eq!(all.calls[1].data, right);
    }

    #[test]
    fn no_intermediate_work_without_averaged_consumers() {
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_at_channel(MonoMock::new(), 0);

        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert!(d.intermediate.is_empty());
    }

    #[test]
    fn intermediate_resets_between_callbacks() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_averaged(MonoMock::new());

        dispatch_stereo(&mut d, &[1.0; 4], &[1.0; 4]);
        dispatch_stereo(&mut d, &[0.0; 4], &[0.0; 4]);

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 2);
        assert_eq!(c.calls[1], vec![0.0_f32; 4]);
    }

    #[test]
    fn consumer_count_reflects_all_modes() {
        let mut d = ConsumerDispatcher::new();
        assert_eq!(d.consumer_count(), 0);

        d.add_consumer_averaged(MonoMock::new());
        d.add_consumer_at_channel(MonoMock::new(), 0);
        d.add_consumer_all(MultiMock::new());

        assert_eq!(d.consumer_count(), 3);
    }

    #[test]
    fn empty_dispatcher_does_not_panic() {
        let mut d = ConsumerDispatcher::new();
        d.dispatch(&[1.0; 64], ch(0, 1), BlockTime::none());
    }

    #[test]
    fn empty_block_dispatches_empty_slice() {
        let mut d = ConsumerDispatcher::new();
        let consumer = d.add_consumer_at_channel(MonoMock::new(), 0);

        d.dispatch(&[], ch(0, 1), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.calls.len(), 1);
        assert!(c.calls[0].is_empty());
    }
}
