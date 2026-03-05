use hug_shared::RcCell;
use vizia::vg;

use super::gui_prelude::*;

pub trait OscilloscopeData {
    fn with_points<F, R>(&self, denorm: ViewportTransform, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R;
}

/// Displays an audio waveform buffer as a stroked and filled line.
///
/// The `Oscilloscope` visualizes data from a [`OscilloscopeData`] by plotting peak values
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
pub struct Oscilloscope<D: 'static> {
    data: D,

    #[extension(ext)]
    min: f32,

    #[extension(ext)]
    max: f32,

    #[extension(ext)]
    filled_path: bool,
}

impl<D: OscilloscopeData> View for Oscilloscope<D> {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        cx.draw_background(canvas);
        clip_bounds(cx, canvas);

        let bounds = cx.bounds();
        let denorm = ViewportTransform::new(bounds, self.min, self.max);
        let (_, zero_y) = denorm.transform(0., 0.);
        let Some(path_with_closing) = self.data.with_points(denorm, bounds.width(), |points| {
            make_strokepath(points, zero_y)
        }) else {
            return;
        };

        self.draw_stroke(cx, canvas, &path_with_closing.path);
        if self.filled_path {
            let mut path = path_with_closing.path;
            let [pt1, pt2] = path_with_closing.closing_points;
            path.line_to(pt1);
            path.line_to(pt2);
            path.close();
            self.draw_fill(cx, canvas, &path);
        }
    }
}

impl<D: OscilloscopeData> Oscilloscope<D> {
    pub fn new(cx: &mut Context, data: D) -> Handle<'_, Self> {
        Self {
            data,
            min: -1.0,
            max: 1.0,
            filled_path: true,
        }
        .build(cx, |_| {})
    }

    /// Draw the stroke path
    fn draw_stroke(&self, cx: &mut DrawContext, canvas: &Canvas, path: &vg::Path) {
        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_stroke_width(cx.border_width());
        paint.set_stroke_cap(vg::PaintCap::Round);
        paint.set_style(vg::PaintStyle::Stroke);
        paint.set_anti_alias(true);
        canvas.draw_path(&path, &paint);
    }

    /// Draw the filled path (lower opacity)
    fn draw_fill(&self, cx: &mut DrawContext, canvas: &Canvas, path: &vg::Path) {
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
        canvas.clip_path(&path, vg::ClipOp::Intersect, false);
        canvas.draw_rect(rect, &fill_paint);
        canvas.restore();
    }
}

// Must implement for useage in oscilloscope
impl OscilloscopeData for Vec<f32> {
    fn with_points<F, R>(&self, denorm: ViewportTransform, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let length = self.len() as f32;
        let mut iterator = self
            .iter()
            .copied()
            .enumerate()
            .map(|(i, y)| denorm.transform((i as f32) / length, y));
        f(&mut iterator)
    }
}

impl OscilloscopeData for Vec<(f32, f32)> {
    fn with_points<F, R>(&self, denorm: ViewportTransform, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let mut iterator = self.iter().copied().map(|(x, y)| denorm.transform(x, y));
        f(&mut iterator)
    }
}

impl OscilloscopeData for RcCell<WindowBuffer> {
    fn with_points<F, R>(&self, denorm: ViewportTransform, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let borrow = self.borrow();
        let length = borrow.num_points() as f32;
        let mut iterator = borrow
            .iter_peaks()
            .enumerate()
            .map(|(i, y)| denorm.transform((i as f32) / length, y));
        f(&mut iterator)
    }
}

impl<Func> OscilloscopeData for Func
where
    Func: Fn(f32) -> f32,
{
    fn with_points<F, R>(&self, denorm: ViewportTransform, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let max = width.ceil() / 2.0;
        let mut iterator = (0..max as usize).map(|v| {
            let normalized = v as f32 / max;
            denorm.transform(normalized, self(normalized))
        });
        f(&mut iterator)
    }
}
