use hug_shared::RcCell;
use vizia::vg;

use super::gui_prelude::*;

/// Displays an audio waveform buffer as a stroked and filled line.
///
/// The `Oscilloscope` visualizes data from a [`WindowBufferAvg`] by plotting peak values
/// across all frequency buckets. It supports dynamic updates via a redraw lens
/// that invalidates the view whenever new data arrives in the bound buffer.
///
/// # Configuration
/// - `buffer`: Optional reference to the audio buffer. If present, peaks are drawn.
/// - `min` / `max`: Clamping bounds for the y-axis normalization (defaults: -1.0 to 1.0).
///
/// # Note:
/// This struct will redraw only if a lens for redraw is given. You must have [`RedrawOnExt`] trait in
/// scope to add the lens.
#[derive(HandleExtension)]
pub struct Oscilloscope {
    /// Might change to some more generic struct
    /// Oscilloscope could draw any buffer actually
    buffer: RcCell<WindowBufferAvg>,

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
    pub fn new(cx: &mut Context, buffer: RcCell<WindowBufferAvg>) -> Handle<'_, Self> {
        Self {
            buffer,
            min: -1.0,
            max: 1.0,
        }
        .build(cx, |_| {})
    }

    /// Draw the stroke path
    fn draw_stroke(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Ok(buckets) = self.buffer.try_borrow() else {
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

    /// Draw the filled path (lower opacity)
    fn draw_fill(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Ok(buckets) = self.buffer.try_borrow() else {
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
