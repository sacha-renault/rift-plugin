use hug_accumulator::{AudioAccumulator, AudioConsumer};
use hug_shared::RcCell;
use vizia::prelude::*;

pub struct NewData;

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
        Some("logic-node")
    }

    fn draw(&self, _: &mut DrawContext, _: &Canvas) {}

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|_: &NewData, _| {
            let acc = self.accumulator.get(cx);
            acc.drain(|data_block, infos, time| {
                for consumer_cell in self.consumers.iter() {
                    match consumer_cell.try_borrow_mut() {
                        Ok(mut consumer) => consumer.consume(data_block, infos, time),
                        Err(err) => log::error!("Couldn't add data in consumer {err}"),
                    }
                }
            });
        });
    }
}

pub trait AudioConsumerDispatchExt {
    fn add_consumer(self, consumer: RcCell<dyn AudioConsumer>) -> Self;
    fn redraw_lens(self) -> impl Lens<Target = u64>;
}

impl<const N: usize, L> AudioConsumerDispatchExt for Handle<'_, AudioConsumerDispatch<N, L>>
where
    L: Lens<Target = AudioAccumulator<N>>,
{
    fn add_consumer(self, consumer: RcCell<dyn AudioConsumer>) -> Self {
        self.modify(|acc_drain| acc_drain.consumers.push(consumer))
    }

    fn redraw_lens(self) -> impl Lens<Target = u64> {
        let lens = self
            .data::<AudioConsumerDispatch<N, L>>()
            .unwrap()
            .accumulator
            .map(|acc| acc.num_writes());
        lens
    }
}
