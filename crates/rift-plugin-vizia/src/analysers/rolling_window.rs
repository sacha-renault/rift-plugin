use crate::dev_prelude::*;
use rift_plugin_accumulator::prelude::{Bucket, ConsumerCell, RollingWindow};
use vizia::vg::{RRect, Rect};

pub struct RollingWindowUI<B: Bucket> {
    inner: ConsumerCell<RollingWindow<B>>,
}

impl<B: Bucket> RollingWindowUI<B> {
    pub fn new(cx: &mut Context, consumer: ConsumerCell<RollingWindow<B>>) -> Handle<'_, Self> {
        // TODO, change this
        // rn we use redraw at every frame but this is wrong.
        // We could have blocks of size 2048. Then, at 48000kHz it
        // has refresh 23 times per sec. So it redraw more than required.
        // Switch to a signal that ticks everytime the accumulator has new
        // data.
        let frame_signal = FrameSignal::signal(cx);

        Self { inner: consumer }
            .build(cx, |_| {})
            .bind(*frame_signal, |mut handle| handle.needs_redraw())
            .role(Role::Canvas)
    }
}

impl<B: Bucket> View for RollingWindowUI<B> {
    fn element(&self) -> Option<&'static str> {
        Some("wave")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Ok(mut consumer) = self.inner.try_borrow_mut() else {
            debug_assert!(false, "Cannot borrow consumer");
            return;
        };

        let BoundingBox { x, y, w, h } = cx.bounds();
        if w < 1.0 || h < 1.0 {
            return;
        }

        // Noop if nothing changes
        consumer.set_columns(w.floor() as usize);

        let visible = consumer.visible_columns();
        if !visible.is_finite() || visible <= 0.0 {
            return;
        }

        let scale = cx.scale_factor();
        let Slots { accent, .. } = Slots::read(cx);

        let paint = fill_paint(accent);

        let x0 = x.round();
        let right = (x + w).round();
        let center = y + h * 0.5;
        let half = h * 0.5;
        let min_height = scale.max(1.0).min(h);
        let shift = consumer.phase().round();

        // TODO: rendering is bucket-dependent. PeakBucket wants min/max rects/step path.
        // RmsBucket's min/max are synthetic ±rms, so a filled top/bottom path looks
        // smoother. A single filled path over min/max would work for both and cut draw calls.
        for (i, bucket) in consumer.iter().enumerate() {
            if bucket.count() == 0 {
                continue;
            }
            let (lo, hi) = (bucket.min(), bucket.max());
            if !lo.is_finite() || !hi.is_finite() {
                continue;
            }

            // Always an integer: left edge and width are whole pixels.
            let col_left = right - shift - i as f32;
            let left = col_left.max(x0);
            let col_right = (col_left + 1.0).min(right);
            if col_right <= left {
                continue;
            }

            let mut top = center - hi.clamp(-1.0, 1.0) * half;
            let mut bottom = center - lo.clamp(-1.0, 1.0) * half;
            if bottom - top < min_height {
                let mid = (top + bottom) * 0.5;
                top = mid - min_height * 0.5;
                bottom = mid + min_height * 0.5;
            }
            let top = top.max(y);
            let bottom = bottom.min(y + h);
            if bottom <= top {
                continue;
            }

            let rect = Rect::from_xywh(left, top, col_right - left, bottom - top);
            canvas.draw_rrect(RRect::new_rect_xy(rect, 0.0, 0.0), &paint);
        }
    }
}
