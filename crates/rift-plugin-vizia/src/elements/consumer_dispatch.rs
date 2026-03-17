use rift_plugin_accumulator::{AudioAccumulator, AudioConsumer};
use rift_plugin_shared::RcCell;
use vizia::prelude::*;

/// Simple struct that is send when the binding over the redraw lens change
/// This will be processed in [`View::event`].
pub struct NewData;

/// Dispatches audio data from an accumulator to registered consumers.
///
/// This component listens for write events in its accumulator, drains the data block,
/// and distributes it to all attached [`AudioConsumer`] instances. It allow a many reader
/// on a single queue.
///
/// # Examples:
/// ```compile_fail
/// let dispatcher = AudioConsumerDispatch::new(cx, AppData::accumulator))
///     .add_consumer(wave_consumer.clone())
///     .add_consumer(audio_peaks_consumer.clone());
///
/// // The redraw lens allow any component to redraw only when there is fresh data
/// let redraw_lens = dispatcher.redraw_lens();
/// ```
pub struct AudioConsumerDispatch<const N: usize, L>
where
    L: Lens<Target = AudioAccumulator<N>>,
{
    consumers: Vec<RcCell<dyn AudioConsumer>>,
    accumulator: L,
}

impl<const N: usize, L> AudioConsumerDispatch<N, L>
where
    L: Lens<Target = AudioAccumulator<N>>,
{
    pub fn new(cx: &mut Context, accumulator: L) -> Handle<'_, Self> {
        Self {
            accumulator,
            consumers: Vec::new(),
        }
        .build(cx, move |cx| {
            // This will fire an event every time new data
            // is written in the accumulator
            Binding::new(cx, accumulator.map(|acc| acc.num_writes()), move |cx, _| {
                cx.emit(NewData);
            });
        })
        .width(Pixels(0.0))
        .height(Pixels(0.0))
    }
}

impl<const N: usize, L> View for AudioConsumerDispatch<N, L>
where
    L: Lens<Target = AudioAccumulator<N>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("audio-dispatcher")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|_: &NewData, _| {
            let acc = self.accumulator.get(cx);
            acc.drain(&self.consumers);
        });
    }
}

pub trait AudioConsumerDispatchExt {
    /// Adds a new consumer that will receive audio data drained from the accumulator.
    ///
    /// The added [`AudioConsumer`] is registered with the dispatcher immediately.
    /// When a write event occurs, all registered consumers are notified in sequence.
    fn add_consumer(self, consumer: RcCell<dyn AudioConsumer>) -> Self;

    /// Generates a redraw lens that fires whenever new data arrives in the accumulator.
    ///
    /// This lens maps the accumulator's internal write counter (`num_writes`) to a signal
    /// indicating whether fresh audio blocks have been processed since the last frame.
    fn redraw_lens(&self) -> impl Lens<Target = u64>;
}

impl<const N: usize, L> AudioConsumerDispatchExt for Handle<'_, AudioConsumerDispatch<N, L>>
where
    L: Lens<Target = AudioAccumulator<N>>,
{
    fn add_consumer(self, consumer: RcCell<dyn AudioConsumer>) -> Self {
        self.modify(|acc_drain| acc_drain.consumers.push(consumer))
    }

    fn redraw_lens(&self) -> impl Lens<Target = u64> {
        self.data::<AudioConsumerDispatch<N, L>>()
            .expect("Handle<'_, Self> doesn't contain Self ?")
            .accumulator
            .map(|acc| acc.num_writes())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    const N_TEST: usize = 10;

    #[derive(Lens)]
    struct AccData {
        acc: AudioAccumulator<N_TEST>,
    }

    impl Model for AccData {}

    struct MockConsumer {
        count: usize,
    }

    impl AudioConsumer for MockConsumer {
        fn consume(
            &mut self,
            _: &[f32],
            _: rift_plugin_shared::transport::ChannelsInfo,
            _: rift_plugin_shared::transport::BlockTime,
        ) {
            self.count += 1;
        }
    }

    fn push_audio(acc: AudioAccumulator<N_TEST>) {
        let audio: Vec<f32> = vec![0., 0.25, 0.5, 0.4, 0.7];
        acc.push_slices([audio.as_slice()].into_iter(), None);
    }

    #[test]
    fn test_new() {
        let mut ocx = Context::new();
        let cx = &mut ocx;

        AccData {
            acc: AudioAccumulator::<N_TEST>::new(1, 3),
        }
        .build(cx);

        let consumer = Rc::new(RefCell::new(MockConsumer { count: 0 }));
        let acd = AudioConsumerDispatch::new(cx, AccData::acc).add_consumer(consumer.clone());
        let redraw = acd.redraw_lens();

        let mut consumer_count = 0;
        acd.modify(|view| consumer_count = view.consumers.len());

        let num_writes_before = redraw.get(cx);
        push_audio(AccData::acc.get(cx));
        let num_writes_after = redraw.get(cx);

        assert_eq!(num_writes_before + 1, num_writes_after);
        assert_eq!(consumer_count, 1);
    }

    #[test]
    fn test_dispatch() {
        let mut ocx = Context::new();
        let cx = &mut ocx;

        let acc = AudioAccumulator::<N_TEST>::new(1, 3);
        AccData { acc: acc.clone() }.build(cx);

        let consumer = Rc::new(RefCell::new(MockConsumer { count: 0 }));

        let mut acd = AudioConsumerDispatch {
            accumulator: AccData::acc,
            consumers: vec![consumer.clone()],
        };

        push_audio(acc.clone());
        push_audio(acc.clone());

        let mut event = Event::new(NewData);
        let mut evt_cx = EventContext::new(cx);
        acd.event(&mut evt_cx, &mut event);

        assert_eq!(consumer.borrow().count, 2);
    }
}
