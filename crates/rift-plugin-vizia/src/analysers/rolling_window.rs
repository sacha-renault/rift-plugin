use crate::dev_prelude::*;
use rift_plugin_accumulator::prelude::*;

pub struct RollingWindowUI<B: Bucket> {
    inner: SharedConsumer<RollingWindow<B>>,
}

impl<B: Bucket> RollingWindowUI<B> {
    pub fn new(
        cx: &mut Context,
        consumer: SharedConsumer<RollingWindow<B>>,
        redraw: Signal<()>,
    ) -> Handle<'_, Self> {
        Self { inner: consumer }
            .build(cx, |_| {})
            .bind(redraw, |mut handle| handle.needs_redraw())
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

        let BoundingBox { w, h, .. } = cx.bounds();
        if w < 1.0 || h < 1.0 {
            return;
        }

        // Noop if nothing changes
        consumer.set_columns(w.floor() as usize);

        let visible = consumer.visible_columns();
        if !visible.is_finite() || visible <= 0.0 {
            return;
        }

        draw_rms_buckets(cx, canvas, consumer.phase().round(), consumer.iter());
    }
}
