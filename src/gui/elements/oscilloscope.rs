use hug_shared::RcCell;
use vizia::vg;

use super::gui_prelude::*;

#[derive(HandleExtension)]
pub struct Oscilloscope {
    #[extension(ext)]
    buffer: Option<RcCell<WindowBufferAvg>>,

    #[extension(ext)]
    min: f32,

    #[extension(ext)]
    max: f32,
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
    pub fn new<L>(cx: &mut Context, redraw_lens: L) -> Handle<'_, Self>
    where
        L: Lens<Target = u64>,
    {
        let mut handle = Self {
            buffer: None,
            min: -1.0,
            max: 1.0,
        }
        .build(cx, |_| {});
        let entity = handle.entity();

        Binding::new(handle.context(), redraw_lens, move |cx, _| {
            cx.needs_redraw(entity);
        });

        handle
    }

    fn draw_stroke(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let buckets = match self.buffer.as_ref().map(|b| b.try_borrow()) {
            Some(Ok(buckets)) => buckets,
            Some(Err(err)) => {
                log::error!("{err}");
                return;
            }
            None => {
                return;
            }
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
        let buckets = match self.buffer.as_ref().map(|b| b.try_borrow()) {
            Some(Ok(buckets)) => buckets,
            Some(Err(err)) => {
                log::error!("{err}");
                return;
            }
            None => return, // This is just no set
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
