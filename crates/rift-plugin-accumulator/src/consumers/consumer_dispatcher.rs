use rift_plugin_core::prelude::{BlockTime, ChannelsInfo, ConsumerCell};

use crate::prelude::AudioConsumer;

/// Determines how audio channels are reduced before reaching a consumer.
pub enum ChannelMode {
    /// Average all channels into a single mono signal.
    ///
    /// The consumer receives one block per audio callback, delivered after the
    /// last channel has been accumulated. Each sample is the arithmetic mean
    /// of the corresponding samples across all channels.
    Averaged,

    /// Forward every channel independently.
    ///
    /// The consumer is called once per channel per audio callback, receiving
    /// the raw block along with the full [`ChannelsInfo`] so it can distinguish
    /// which channel it is processing. This is useful for consumers that track
    /// per-channel state, such as peak meters.
    All,

    /// Forward a single channel unchanged.
    ///
    /// The consumer receives the raw block for the specified channel index
    /// and is never called for other channels.
    Channel(usize),
}

/// Pairs a consumer with its channel routing mode.
struct ConsumerWithMode {
    consumer: ConsumerCell<dyn AudioConsumer>,
    mode: ChannelMode,
}

/// Routes multi-channel audio blocks to registered [`AudioConsumer`]s according
/// to each consumer's [`ChannelMode`].
///
/// This struct centralises the channel-reduction logic that would otherwise be
/// duplicated inside every consumer. Consumers registered here only need to
/// handle a plain `&[f32]` block. They never deal with channel bookkeeping.
///
/// # Example
///
/// ```ignore
/// let mut dispatcher = ConsumerDispatcher::new();
/// dispatcher.add_consumer_averaged(my_vu_meter.clone());
/// dispatcher.add_consumer_at_channel(my_stft.clone(), 0);
///
/// // In the drain loop (called from the UI thread):
/// for ch in 0..total_channels {
///     dispatcher.dispatch(block, channels_info, time);
/// }
/// ```
pub struct ConsumerDispatcher {
    /// Scratch buffer used to accumulate the channel average.
    /// Only allocated / used when at least one consumer is in [`ChannelMode::Averaged`].
    intermediate: Vec<f32>,

    /// All registered consumers with their routing mode.
    consumers: Vec<ConsumerWithMode>,

    /// Fast-path flag: when `false`, the averaging work in [`dispatch`](Self::dispatch) is
    /// skipped entirely. Set to `true` once by [`add_consumer_averaged`](Self::add_consumer_averaged).
    /// Since consumers cannot be removed, this flag never goes stale.
    has_average_consumer: bool,
}

impl ConsumerDispatcher {
    /// Creates an empty dispatcher with no consumers registered.
    pub fn new() -> Self {
        Self {
            intermediate: Vec::new(),
            consumers: Vec::new(),
            has_average_consumer: false,
        }
    }

    /// Registers a consumer that will receive a mono-averaged block once per
    /// audio callback, after all channels have been dispatched.
    pub fn add_consumer_averaged(&mut self, consumer: ConsumerCell<dyn AudioConsumer>) {
        let consumer_with_mode = ConsumerWithMode {
            consumer,
            mode: ChannelMode::Averaged,
        };

        self.has_average_consumer = true;
        self.consumers.push(consumer_with_mode);
    }

    /// Registers a consumer that will receive a every channel independently.
    pub fn add_consumer_all(&mut self, consumer: ConsumerCell<dyn AudioConsumer>) {
        let consumer_with_mode = ConsumerWithMode {
            consumer,
            mode: ChannelMode::All,
        };

        self.consumers.push(consumer_with_mode);
    }

    /// Registers a consumer that will receive the raw block for `channel` only.
    /// Blocks for all other channels are silently ignored.
    pub fn add_consumer_at_channel(
        &mut self,
        consumer: ConsumerCell<dyn AudioConsumer>,
        channel: usize,
    ) {
        let consumer_with_mode = ConsumerWithMode {
            consumer,
            mode: ChannelMode::Channel(channel),
        };
        self.consumers.push(consumer_with_mode);
    }

    /// Returns the total number of registered consumers (both averaged and
    /// channel-specific).
    pub fn consumer_count(&self) -> usize {
        self.consumers.len()
    }

    /// Routes `block` to the appropriate consumers based on the current channel.
    ///
    /// Must be called once per channel per audio callback, in ascending channel
    /// order. Consumers whose [`ConsumerCell`] is already borrowed are silently
    /// skipped to avoid panics in single-threaded `RefCell` scenarios.
    pub fn dispatch(&mut self, block: &[f32], channels: ChannelsInfo, time: BlockTime) {
        let total_channel = channels.total_channels as f32;

        // At channel 0, we ensure our intermediate buffer is large enough
        if self.has_average_consumer {
            if channels.current == 0 {
                self.intermediate.clear();
                self.intermediate.resize(block.len(), 0.);
            }

            for (s, &v) in self.intermediate.iter_mut().zip(block.iter()) {
                *s += v / total_channel;
            }
        }

        for ConsumerWithMode { consumer, mode } in self.consumers.iter() {
            let Ok(mut consumer) = consumer.try_borrow_mut() else {
                #[cfg(debug_assertions)]
                panic!("Can't borrow consumer");

                #[cfg(not(debug_assertions))]
                continue;
            };

            match mode {
                ChannelMode::Averaged if channels.is_last_channel() => {
                    consumer.consume(self.intermediate.as_slice(), channels, time);
                }
                ChannelMode::Channel(c) if channels.current == *c => {
                    consumer.consume(block, channels, time);
                }
                ChannelMode::All => consumer.consume(block, channels, time),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use rift_plugin_core::prelude::{BlockTime, ChannelsInfo, ConsumerCell};

    use crate::prelude::AudioConsumer;

    use super::*;

    struct ConsumerMock {
        calls: Vec<MockCall>,
    }

    struct MockCall {
        data: Vec<f32>,
        channel: usize,
        total_channels: usize,
    }

    impl ConsumerMock {
        fn new() -> ConsumerCell<Self> {
            Rc::new(RefCell::new(ConsumerMock { calls: Vec::new() }))
        }

        fn n_calls(&self) -> usize {
            self.calls.len()
        }
    }

    impl AudioConsumer for ConsumerMock {
        fn consume(&mut self, block: &[f32], channels: ChannelsInfo, _: BlockTime) {
            self.calls.push(MockCall {
                data: block.to_vec(),
                channel: channels.current,
                total_channels: channels.total_channels,
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
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_at_channel(consumer.clone(), 1);

        let left = vec![0.0_f32; 4];
        let right = vec![1.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 1);
        assert_eq!(c.calls[0].data, right);
    }

    #[test]
    fn channel_mode_ignores_wrong_channel() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_at_channel(consumer.clone(), 1);

        // Only dispatch channel 0
        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert_eq!(consumer.borrow().n_calls(), 0);
    }

    #[test]
    fn averaged_mode_produces_mean_of_channels() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_averaged(consumer.clone());

        let left = vec![1.0_f32; 4];
        let right = vec![0.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 1);
        assert_eq!(c.calls[0].data, vec![0.5_f32; 4]);
    }

    #[test]
    fn averaged_mode_not_called_until_last_channel() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_averaged(consumer.clone());

        // Only dispatch channel 0 out of 2
        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert_eq!(consumer.borrow().n_calls(), 0);
    }

    #[test]
    fn averaged_mode_mono_passes_through() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_averaged(consumer.clone());

        let block = vec![0.7_f32; 4];
        d.dispatch(&block, ch(0, 1), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 1);
        assert_eq!(c.calls[0].data, block);
    }

    #[test]
    fn all_mode_receives_every_channel() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_all(consumer.clone());

        let left = vec![1.0_f32; 4];
        let right = vec![2.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 2);
        assert_eq!(c.calls[0].channel, 0);
        assert_eq!(c.calls[0].data, left);
        assert_eq!(c.calls[1].channel, 1);
        assert_eq!(c.calls[1].data, right);
    }

    #[test]
    fn all_mode_forwards_channel_info() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_all(consumer.clone());

        d.dispatch(&[0.0; 4], ch(0, 3), BlockTime::none());
        d.dispatch(&[0.0; 4], ch(1, 3), BlockTime::none());
        d.dispatch(&[0.0; 4], ch(2, 3), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 3);
        for (i, call) in c.calls.iter().enumerate() {
            assert_eq!(call.channel, i);
            assert_eq!(call.total_channels, 3);
        }
    }

    #[test]
    fn mixed_modes_all_receive_correct_data() {
        let avg_consumer = ConsumerMock::new();
        let ch0_consumer = ConsumerMock::new();
        let all_consumer = ConsumerMock::new();

        let mut d = ConsumerDispatcher::new();
        d.add_consumer_averaged(avg_consumer.clone());
        d.add_consumer_at_channel(ch0_consumer.clone(), 0);
        d.add_consumer_all(all_consumer.clone());

        let left = vec![1.0_f32; 4];
        let right = vec![0.0_f32; 4];
        dispatch_stereo(&mut d, &left, &right);

        // Averaged: called once with mean
        let avg = avg_consumer.borrow();
        assert_eq!(avg.n_calls(), 1);
        assert_eq!(avg.calls[0].data, vec![0.5_f32; 4]);

        // Channel(0): called once with left
        let ch0 = ch0_consumer.borrow();
        assert_eq!(ch0.n_calls(), 1);
        assert_eq!(ch0.calls[0].data, left);

        // All: called twice with both channels
        let all = all_consumer.borrow();
        assert_eq!(all.n_calls(), 2);
        assert_eq!(all.calls[0].data, left);
        assert_eq!(all.calls[1].data, right);
    }

    #[test]
    fn no_intermediate_work_without_averaged_consumers() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_at_channel(consumer.clone(), 0);

        assert!(!d.has_average_consumer);
        assert!(d.intermediate.is_empty());
        d.dispatch(&[1.0; 4], ch(0, 2), BlockTime::none());

        assert!(d.intermediate.is_empty());
    }

    #[test]
    fn intermediate_resets_between_callbacks() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_averaged(consumer.clone());

        dispatch_stereo(&mut d, &[1.0; 4], &[1.0; 4]);
        dispatch_stereo(&mut d, &[0.0; 4], &[0.0; 4]);

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 2);
        assert_eq!(c.calls[1].data, vec![0.0_f32; 4]);
    }

    #[test]
    fn consumer_count_reflects_all_modes() {
        let mut d = ConsumerDispatcher::new();
        assert_eq!(d.consumer_count(), 0);

        d.add_consumer_averaged(ConsumerMock::new());
        d.add_consumer_at_channel(ConsumerMock::new(), 0);
        d.add_consumer_all(ConsumerMock::new());

        assert_eq!(d.consumer_count(), 3);
    }

    #[test]
    fn empty_dispatcher_does_not_panic() {
        let mut d = ConsumerDispatcher::new();
        d.dispatch(&[1.0; 64], ch(0, 1), BlockTime::none());
    }

    #[test]
    fn empty_block_dispatches_empty_slice() {
        let consumer = ConsumerMock::new();
        let mut d = ConsumerDispatcher::new();
        d.add_consumer_at_channel(consumer.clone(), 0);

        d.dispatch(&[], ch(0, 1), BlockTime::none());

        let c = consumer.borrow();
        assert_eq!(c.n_calls(), 1);
        assert!(c.calls[0].data.is_empty());
    }
}
