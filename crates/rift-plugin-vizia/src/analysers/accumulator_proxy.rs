use std::{cell::RefCell, rc::Rc, sync::Arc};

use rift_plugin_accumulator::prelude::*;
use rift_plugin_types::transport::BlockTime;

use super::*;

use crate::dev_prelude::*;

pub struct AccumulatorProxy {
    consumers: Rc<RefCell<ConsumerDispatcher>>,
}

impl AccumulatorProxy {
    /// Starts pumping `accumulator` every frame for as long as the GUI lives.
    ///
    /// Call from a static layout only (root is always fine). You risk otherwise to register
    /// multiple time the consumer on every view rebuild.
    pub fn new(cx: &mut Context, accumulator: Arc<AudioAccumulator>) -> Self {
        let consumers = Rc::new(RefCell::new(ConsumerDispatcher::new()));
        let frame_signal = FrameSignal::signal(cx);

        let pumped = consumers.clone();
        Binding::new(cx, *frame_signal, move |_| {
            accumulator.ui_thread_read(&mut pumped.borrow_mut());
        });

        Self { consumers }
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
