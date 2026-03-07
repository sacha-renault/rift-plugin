use hug_fft::StftChannelConsumer;
use hug_shared::RcCell;
use vizia::vg;

use super::gui_prelude::*;

use crate::utils::basics::cubic_interpolate;

/// Given a stft, retrieve a value at a fractional index using
/// cubic interpolation
#[inline]
fn sample_spectrum(bins: &[f32], x: f32) -> f32 {
    let len = bins.len();

    // Safety checks
    if len <= 1 {
        return bins[0];
    }

    // handle borders of slice
    if x <= 0.0 {
        return bins[0];
    }
    if x >= (len - 1) as f32 {
        return bins[len - 1];
    }

    let i = x.floor() as usize;
    let t = x.fract();
    if t == 0. {
        return bins[i];
    }

    let x0 = bins[i.saturating_sub(1)];
    let x1 = bins[i];
    let x2 = bins[(i + 1).min(len - 1)];
    let x3 = bins[(i + 2).min(len - 1)];

    cubic_interpolate(x0, x1, x2, x3, t)
}

pub trait OscilloscopeData {
    /// This function return raw values for x and y
    /// You may choose [`Oscilloscope::x_range`] and [`Oscilloscope::y_range`]
    /// so it fits what u wanna display
    fn with_points<F, R>(&self, width: f32, f: F) -> R
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
    filled_path: bool,

    /// This is used to know where to close the wave for the fill
    /// By default it's 0.5 (the filled path goes through middle)
    /// but it can be change for abs display of the wave, etc ..
    #[extension(ext)]
    fill_lign_height: f32,

    #[extension(ext)]
    x_range: (f32, f32),

    #[extension(ext)]
    y_range: (f32, f32),

    #[extension(ext)]
    filter_transform: Option<Box<dyn Fn(f32, f32) -> Option<(f32, f32)>>>,
}

impl<D: OscilloscopeData> View for Oscilloscope<D> {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        cx.draw_background(canvas);
        clip_bounds(cx, canvas);

        let bounds = cx.bounds();
        let vtransform = ViewportTransform::with_range(bounds, self.x_range, self.y_range);
        let Some(path_with_closing) = self.data.with_points(bounds.width(), |points| {
            if let Some(transform) = &self.filter_transform {
                let points = points.filter_map(|(x, y)| transform(x, y));
                make_strokepath(points, vtransform, self.fill_lign_height)
            } else {
                make_strokepath(points, vtransform, self.fill_lign_height)
            }
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
            filled_path: true,
            fill_lign_height: 0.5,
            filter_transform: None,
            x_range: (0., 1.),
            y_range: (0., 1.),
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
    fn with_points<F, R>(&self, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let length = self.len() as f32;
        let mut iterator = self
            .iter()
            .copied()
            .enumerate()
            .map(|(i, y)| ((i as f32) / length, y));
        f(&mut iterator)
    }
}

impl OscilloscopeData for Vec<(f32, f32)> {
    fn with_points<F, R>(&self, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let mut iterator = self.iter().copied();
        f(&mut iterator)
    }
}

impl OscilloscopeData for RcCell<WindowBuffer> {
    fn with_points<F, R>(&self, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let borrow = self.borrow();
        let length = (borrow.num_points() - 1) as f32;
        let mut iterator = borrow
            .iter_peaks()
            .enumerate()
            .map(|(i, y)| ((i as f32) / length, y));
        f(&mut iterator)
    }
}

impl<Func> OscilloscopeData for Func
where
    Func: Fn(f32) -> f32,
{
    fn with_points<F, R>(&self, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let max = 250.;
        let mut iterator = (0..max as usize).map(|v| {
            let normalized = v as f32 / (max - 1.);
            (normalized, self(normalized))
        });
        f(&mut iterator)
    }
}

impl OscilloscopeData for RcCell<StftChannelConsumer> {
    fn with_points<F, R>(&self, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let borrow = self.borrow();

        let bins = borrow.bins();
        let samplerate = borrow.sample_rate();
        let fft_size = borrow.fft_size() as f32;

        let f_min = 20.0;
        let f_max = 20000.0;
        let log_ratio: f32 = f_max / f_min;

        let num_points = width as usize;
        let max = (num_points - 1) as f32;

        let mut iterator = (0..num_points).map(|idx| {
            let x = idx as f32 / max;
            let freq = f_min * log_ratio.powf(x);
            let bin_idx = (freq * fft_size as f32) / samplerate;

            let val = sample_spectrum(bins, bin_idx);

            let db = 20.0 * val.max(1e-5).log10();

            // We add up some db so higher spectrum looks more filled
            (x, db)
        });

        f(&mut iterator)
    }
}
