use std::marker::PhantomData;

use crate::dev_prelude::*;
use rift_plugin_accumulator::prelude::*;
use rift_plugin_types::transport::BlockTime;
use rift_plugin_utils::DEFAULT_SAMPLERATE;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Padding {
    sample_amount: usize,
    value: f32,
}

pub struct StaticWaveform<B: Bucket> {
    wave: Signal<Vec<f32>>,
    left_padding: Signal<Padding>,
    right_padding: Signal<Padding>,
    _p: PhantomData<fn() -> B>,
}

impl<B: Bucket> View for StaticWaveform<B> {
    fn element(&self) -> Option<&'static str> {
        Some("wave")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let left = self.left_padding.get_untracked();
        let right = self.right_padding.get_untracked();
        let wave = self.wave.get_untracked();

        let total_samples = left.sample_amount + right.sample_amount + wave.len();
        let BoundingBox { w, h, .. } = cx.bounds();
        if w < 1.0 || h < 1.0 {
            return;
        }

        // TODO, use actual sr. It works ok`ish` for now because
        // RollingWindow is initialized with DEFAULT_SAMPLERATE
        // as well. ... Careful if using somehting else later
        // the `BlockTime has to use same samplerate
        let seconds = total_samples as f64 / DEFAULT_SAMPLERATE;

        // Create a accumulator to display buckets
        let mut consumer = RollingWindow::<B>::new(seconds);
        consumer.set_columns(w.floor() as usize);

        let visible = consumer.visible_columns();
        if !visible.is_finite() || visible <= 0.0 {
            return;
        }

        // Build the buckets
        let time = BlockTime::empty_with_sr(DEFAULT_SAMPLERATE);

        if left.sample_amount > 0 {
            consumer.consume(&vec![left.value; left.sample_amount], time);
        }

        consumer.consume(&wave, time);

        if right.sample_amount > 0 {
            consumer.consume(&vec![right.value; right.sample_amount], time);
        }

        // Final draw
        draw_rms_buckets(cx, canvas, consumer.phase().round(), consumer.iter());
    }
}

impl<B: Bucket> StaticWaveform<B> {
    pub fn new<D>(cx: &mut Context, wave: D) -> Handle<'_, Self>
    where
        D: Res<Vec<f32>> + 'static,
    {
        let signal = wave.to_signal(cx);
        let left_padding = Signal::default();
        let right_padding = Signal::default();

        Self {
            wave: signal,
            left_padding,
            right_padding,
            _p: PhantomData,
        }
        .build(cx, |_| {})
        .bind(signal, |mut handle| handle.needs_redraw())
        .role(Role::Canvas)
    }
}

#[modifiers(for Handle<'_, StaticWaveform<B>> where B: Bucket)]
pub trait StaticWaveformModifiers {
    #[concrete]
    fn left_padding(self, sample_amount: usize, value: f32) -> Self {
        self.modify(|wave| {
            wave.left_padding.set_if_changed(Padding {
                sample_amount,
                value,
            })
        })
    }

    #[concrete]
    fn right_padding(self, sample_amount: usize, value: f32) -> Self {
        self.modify(|wave| {
            wave.right_padding.set_if_changed(Padding {
                sample_amount,
                value,
            })
        })
    }
}
