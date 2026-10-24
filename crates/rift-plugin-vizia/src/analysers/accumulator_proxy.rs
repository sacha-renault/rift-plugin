use std::{cell::RefCell, rc::Rc, sync::Arc};

use rift_plugin_accumulator::prelude::*;
use rift_plugin_types::transport::BlockTime;

use super::*;

use crate::{analysers::rolling_window::RollingWindowUI, dev_prelude::*};

pub struct AccumulatorProxy {
    consumers: Rc<RefCell<ConsumerDispatcher>>,
}

impl AccumulatorProxy {
    /// Starts pumping `accumulator` every frame for as long as the GUI lives.
    ///
    /// Call from a static layout only (root is always fine). You risk otherwise to register
    /// multiple time the consumer on every view rebuild.
    pub fn install(cx: &mut Context, accumulator: Arc<AudioAccumulator>) -> Self {
        let consumers = Rc::new(RefCell::new(ConsumerDispatcher::new()));
        let frame_signal = FrameSignal::signal(cx);

        let pumped = consumers.clone();
        Binding::new(cx, *frame_signal, move |_| {
            accumulator.ui_thread_read(&mut pumped.borrow_mut());
        });

        Self { consumers }
    }

    pub fn meter<'a>(&self, cx: &'a mut Context, channel: usize) -> Handle<'a, MeterUI> {
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
                peak: AudioPeak::default(),
                level,
                held: peak,
            },
            channel,
        );

        MeterUI::with_peak(cx, level, peak)
    }

    pub fn rolling_window<'a, B: Bucket>(
        &self,
        cx: &'a mut Context,
        kind: MonoConsumerKind,
        seconds: f64,
    ) -> Handle<'a, RollingWindowUI<B>> {
        let consumer = self
            .consumers
            .borrow_mut()
            .add_mono_consumer(RollingWindow::new(seconds), kind);
        RollingWindowUI::new(cx, consumer)
    }

    /// Registers `consumer` for every channel independently.
    ///
    /// Returns the cell the dispatcher writes into (keep it to read the results).
    pub fn add_consumer_all<C: MultiConsumer>(&self, consumer: C) -> ConsumerCell<C> {
        self.consumers.borrow_mut().add_consumer_all(consumer)
    }

    /// Registers `consumer` for a single `channel` only.
    ///
    /// Returns the cell the dispatcher writes into (keep it to read the results).
    pub fn add_consumer_at_channel<C: MonoConsumer>(
        &self,
        consumer: C,
        channel: usize,
    ) -> ConsumerCell<C> {
        self.consumers
            .borrow_mut()
            .add_consumer_at_channel(consumer, channel)
    }

    /// Registers `consumer` for the mono-averaged mix of all channels.
    ///
    /// Returns the cell the dispatcher writes into (keep it to read the results).
    pub fn add_consumer_averaged<C: MonoConsumer>(&self, consumer: C) -> ConsumerCell<C> {
        self.consumers.borrow_mut().add_consumer_averaged(consumer)
    }
}
