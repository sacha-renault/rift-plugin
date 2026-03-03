use hug_shared::RcCell;
use vizia::vg;

use super::gui_prelude::*;

pub trait OscilloscopeExt {
    fn data(self, data: RcCell<WindowedBuffer>) -> Self;
}

pub struct Oscilloscope {
    buffer: Option<RcCell<WindowedBuffer>>,
}

impl View for Oscilloscope {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        cx.draw_background(canvas);
        clip_bounds(cx, canvas);
        self.draw_stroke(cx, canvas);
        self.draw_fill(cx, canvas);
    }
}

impl Oscilloscope {
    pub fn new<LSEC>(cx: &mut Context) -> Handle<'_, Self>
    where
        LSEC: Lens<Target = f64>,
    {
        Self { buffer: None }.build(cx, |_| {})
    }

    fn draw_stroke(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Some(buckets) = self.buffer.as_ref().map(|b| b.borrow()) else {
            return;
        };

        let path = make_open_strokepath(
            Denormalizer::from_cx(cx),
            buckets.iter_peaks(),
            buckets.num_points(),
        );

        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_stroke_width(cx.border_width());
        paint.set_stroke_cap(vg::PaintCap::Round);
        paint.set_style(vg::PaintStyle::Stroke);
        paint.set_anti_alias(true);
        canvas.draw_path(&path, &paint);
    }

    fn draw_fill(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Some(buckets) = self.buffer.as_ref().map(|b| b.borrow()) else {
            return;
        };

        let stroke_path = make_closed_strokepath(
            Denormalizer::from_cx(cx),
            buckets.iter_peaks(),
            buckets.num_points(),
        );

        let mut fill_paint = vg::Paint::default();
        let font_color = cx.font_color();
        let color = Color::rgba(
            font_color.r(),
            font_color.g(),
            font_color.b(),
            (font_color.a() as f32 * 0.4) as u8,
        );
        fill_paint.set_color(color);
        fill_paint.set_stroke_cap(vg::PaintCap::Round);
        fill_paint.set_style(vg::PaintStyle::Fill);
        fill_paint.set_anti_alias(false);

        let bounds = cx.bounds();
        let rect = vg::Rect::new(bounds.x, bounds.y, bounds.x + bounds.w, bounds.y + bounds.h);

        canvas.save();
        canvas.clip_path(&stroke_path, vg::ClipOp::Intersect, false);
        canvas.draw_rect(rect, &fill_paint);
        canvas.restore();
    }
}

impl OscilloscopeExt for Handle<'_, Oscilloscope> {
    /// This view assume that data are drain and feed into data in an other component
    fn data(self, data: RcCell<WindowedBuffer>) -> Self {
        self.modify(|osc| osc.buffer = Some(data))
    }
}
