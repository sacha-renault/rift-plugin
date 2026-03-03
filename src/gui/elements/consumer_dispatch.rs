use hug_accumulator::{AudioAccumulator, AudioConsumer};
use hug_shared::RcCell;
use vizia::prelude::*;

pub struct NewData;

pub struct AudioConsumerDispatch<const N: usize> {
    consumers: Vec<RcCell<dyn AudioConsumer>>,
    accumulator: AudioAccumulator<N>,
}

impl<const N: usize> AudioConsumerDispatch<N> {
    pub fn new<L: Lens<Target = usize>>(
        cx: &mut Context,
        accumulator: AudioAccumulator<N>,
        new_data_lens: L,
    ) -> Handle<'_, Self> {
        Self {
            accumulator,
            consumers: Vec::new(),
        }
        .build(cx, move |cx| {
            // This will fire an event every time new data
            // is written in the accumulator
            Binding::new(cx, new_data_lens, move |cx, _| {
                cx.emit(NewData);
            });
        })
    }
}

impl<const N: usize> View for AudioConsumerDispatch<N> {
    fn element(&self) -> Option<&'static str> {
        Some("logic-node")
    }

    fn draw(&self, _: &mut DrawContext, _: &Canvas) {}

    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|_: &NewData, _| {
            self.accumulator.drain(|data_block, infos, time| {
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
}

impl<const N: usize> AudioConsumerDispatchExt for Handle<'_, AudioConsumerDispatch<N>> {
    fn add_consumer(self, consumer: RcCell<dyn AudioConsumer>) -> Self {
        self.modify(|acc_drain| acc_drain.consumers.push(consumer))
    }
}
