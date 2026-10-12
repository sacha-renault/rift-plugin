use std::{cell::RefCell, sync::Arc};

use rift_plugin_accumulator::prelude::*;
use rift_plugin_types::transport::BlockTime;

use super::*;

use crate::dev_prelude::*;

pub struct AccumulatorProxy {
    consumers: RefCell<ConsumerDispatcher>,
}

impl AccumulatorProxy {
    pub fn new() -> Self {
        Self {
            consumers: RefCell::new(ConsumerDispatcher::new()),
        }
    }

    /// Consumer the proxy and start pumping the values on GUI side at every frame.
    pub fn pump_on_frame(self, cx: &mut Context, acc: Arc<AudioAccumulator>) {
        let frame_signal = FrameSignal::signal(cx);

        Binding::new(cx, *frame_signal, move |_| {
            self.pump(&acc);
        });
    }

    pub fn meter<'a>(&self, cx: &'a mut Context, channel: usize) -> Handle<'a, Meter> {
        let (level, peak) = (Signal::new(0.0), Signal::new(0.0));

        // a proxy that just forward the values of the pump inside
        // the UI meter
        struct PeakMeter {
            peak: AudioPeak,
            level: Signal<f32>,
            held: Signal<f32>,
        }

        impl MonoConsumer for PeakMeter {
            fn consume(&mut self, block: &[f32], time: BlockTime) {
                self.peak.consume(block, time);
                self.level.set_if_changed(self.peak.level());
                self.held.set_if_changed(self.peak.held());
            }
        }

        self.add_consumer_at_channel(
            PeakMeter {
                peak: AudioPeak::new(),
                level,
                held: peak,
            },
            channel,
        );

        Meter::with_peak(cx, level, peak)
    }

    fn pump(&self, acc: &AudioAccumulator) {
        acc.ui_thread_read(&mut self.consumers.borrow_mut());
    }

    pub fn add_consumer_all<C: MultiConsumer>(&self, consumer: C) {
        self.consumers
            .borrow_mut()
            .add_consumer_all(consumer.wraps_consumer());
    }

    pub fn add_consumer_at_channel<C: MonoConsumer>(&self, consumer: C, channel: usize) {
        self.consumers
            .borrow_mut()
            .add_consumer_at_channel(consumer.wraps_consumer(), channel);
    }

    pub fn add_consumer_averaged<C: MonoConsumer>(&self, consumer: C) {
        self.consumers
            .borrow_mut()
            .add_consumer_averaged(consumer.wraps_consumer());
    }
}
