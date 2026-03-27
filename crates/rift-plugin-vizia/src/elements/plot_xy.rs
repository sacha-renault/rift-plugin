use rift_plugin_accumulator::prelude::*;
use rift_plugin_core::prelude::ConsumerCell;
use rift_plugin_core::utils::interpo::{catmull_interpolate_buffer, lerp_interpolate_buffer};
use rift_plugin_core::utils::spaces::Linspace;

use vizia::vg;

use crate::utils::draw_utils::close_path;

use super::gui_prelude::*;

/// Displays an audio waveform buffer as a stroked and filled line.
///
/// The [`PlotXY`] visualizes data from a [`PlotData`] by plotting peak values
/// across all frequency buckets. It supports dynamic updates via a redraw lens
/// that invalidates the view whenever new data arrives in the bound buffer.
///
/// # Configuration
/// - `buffer`: Optional reference to the audio buffer. If present, peaks are drawn.
/// - `min` / `max`: Clamping bounds for the y-axis normalization (defaults: -1.0 to 1.0).
///
/// **Notes**:
/// This struct will redraw only if a lens for redraw is given. You must have [`RedrawOnExt`] trait in
/// scope to add the lens.
#[derive(HandleExtension)]
pub struct PlotXY<D: 'static> {
    data: D,

    #[extension(ext)]
    filled_path: bool,

    /// This is used to know where to close the wave for the fill
    /// By default it's 0.0
    #[extension(ext)]
    fill_lign_height: f32,

    #[extension(ext)]
    x_range: (f32, f32),

    #[extension(ext)]
    y_range: (f32, f32),

    #[extension(ext)]
    resolution: f32,

    #[extension(ext)]
    fill_opacity: u8,

    #[extension(ext)]
    filter_transform: Option<Box<dyn Fn(f32, f32) -> Option<(f32, f32)>>>,

    #[extension(ext, set = CachedTexture::new(), setter_name = use_cache_texture)]
    cache: Option<CachedTexture>,
}

impl<D: PlotData> View for PlotXY<D> {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        if let Some(cache) = self.cache.as_ref() {
            cache.draw(cx, canvas, |cx, canvas| self.draw_all(cx, canvas));
        } else {
            self.draw_all(cx, canvas);
        }
    }

    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|_: &RedrawLensEvent, _| {
            if let Some(cache) = self.cache.as_ref() {
                cache.invalidate();
            }
        });
    }

    fn element(&self) -> Option<&'static str> {
        Some("oscilloscope")
    }
}

impl<D: PlotData> PlotXY<D> {
    pub fn new(cx: &mut Context, data: D) -> Handle<'_, Self> {
        Self {
            data,
            filled_path: true,
            fill_lign_height: 0.,
            filter_transform: None,
            x_range: (0., 1.),
            y_range: (0., 1.),
            resolution: 1.0,
            cache: None,
            fill_opacity: 100,
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
        canvas.draw_path(path, &paint);
    }

    /// Draw the filled path (lower opacity)
    fn draw_fill(&self, cx: &mut DrawContext, canvas: &Canvas, path: &vg::Path) {
        let mut fill_paint = vg::Paint::default();
        let color = change_color_opacity(cx.font_color(), self.fill_opacity);

        fill_paint.set_color(color);
        fill_paint.set_stroke_cap(vg::PaintCap::Round);
        fill_paint.set_style(vg::PaintStyle::Fill);
        fill_paint.set_anti_alias(false);

        let bounds = cx.bounds();
        let rect = vg::Rect::new(bounds.x, bounds.y, bounds.x + bounds.w, bounds.y + bounds.h);

        canvas.save();
        canvas.clip_path(path, vg::ClipOp::Intersect, false);
        canvas.draw_rect(rect, &fill_paint);
        canvas.restore();
    }

    fn draw_all(&self, cx: &mut DrawContext, canvas: &Canvas) {
        cx.draw_background(canvas);
        clip_bounds(cx, canvas);

        let bounds = cx.bounds();
        let vtransform = ViewportTransform::with_range(bounds, self.x_range, self.y_range);
        let scaled_width = bounds.width() * self.resolution;
        let Some(mut path) = self.data.with_points(scaled_width, |points| {
            if let Some(transform) = &self.filter_transform {
                let points = points.filter_map(|(x, y)| transform(x, y));
                make_strokepath(points, &vtransform)
            } else {
                make_strokepath(points, &vtransform)
            }
        }) else {
            return;
        };

        self.draw_stroke(cx, canvas, &path);
        if self.filled_path {
            close_path(&mut path, &vtransform, self.fill_lign_height);
            self.draw_fill(cx, canvas, &path);
        }
    }
}

pub trait PlotData {
    /// This function return raw values for x and y
    /// You may choose [`Oscilloscope::x_range`] and [`Oscilloscope::y_range`]
    /// so it fits what u wanna display
    fn with_points<F, R>(&self, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R;
}

// Must implement for useage in oscilloscope
impl PlotData for Vec<f32> {
    fn with_points<F, R>(&self, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let width = width.ceil();
        let max = (self.len() - 1) as f32;

        // todo!(), maybe we don't always want this interpolation ?
        // Should interpolation be like ... a choice ?
        // To think ...
        if width <= max {
            let mut iterator = self
                .iter()
                .copied()
                .enumerate()
                .map(|(i, y)| ((i as f32) / max, y));
            f(&mut iterator)
        } else {
            let mut iterator = Linspace::new(0., 1., width as usize).map(|x| {
                let y = lerp_interpolate_buffer(self, x * max);
                (x, y)
            });
            f(&mut iterator)
        }
    }
}

impl PlotData for Vec<(f32, f32)> {
    fn with_points<F, R>(&self, _: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        // todo!()
        // use the fcking with to interpolate
        let mut iterator = self.iter().copied();
        f(&mut iterator)
    }
}

impl<B: Bucket> PlotData for ConsumerCell<WindowBuckets<B>> {
    fn with_points<F, R>(&self, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let mut borrow = self.borrow_mut();
        // todo!()
        // this is bs, we need to find a way to interpolate instead
        // of raw set the num buckets to fit width ...
        borrow.set_num_buckets(width.ceil() as usize);
        let length = (borrow.num_points() - 1) as f32;
        let mut iterator = borrow
            .iter_values()
            .enumerate()
            .map(|(i, y)| ((i as f32) / length, y));

        f(&mut iterator)
    }
}

impl<Func> PlotData for Func
where
    Func: Fn(f32) -> f32,
{
    fn with_points<F, R>(&self, width: f32, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut dyn Iterator<Item = (f32, f32)>) -> R,
    {
        let num_points = width.ceil() as usize;
        let mut iterator = Linspace::new(0., 1., num_points).map(|x| (x, self(x)));
        f(&mut iterator)
    }
}

/// Yes, we do display spectrogram in the oscilloscope, it works the same
/// anyway, we can use Oscilloscope for any data that for y has <= 1 x.
impl PlotData for ConsumerCell<StftConsumer> {
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
        let num_points = width.ceil() as usize;

        // Would be preferable to use logspace here, but we need the normalized linear value
        // of x for visual mapping, so we'll stick with linspace.
        let mut iterator = Linspace::new(0.0, 1.0, num_points).map(|x| {
            let freq = f_min * log_ratio.powf(x);
            let bin_idx = (freq * fft_size) / samplerate;
            let val = catmull_interpolate_buffer(bins, bin_idx);
            let db = 20.0 * val.max(1e-5).log10();
            (x, db)
        });

        f(&mut iterator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rift_plugin_core::assert_approx_eq;

    #[test]
    fn test_vec_f32_points_normalized_x() {
        let data = vec![0.0, 0.5, 1.0];
        data.with_points(3., |iter| {
            let pts: Vec<_> = iter.collect();
            assert_eq!(pts.len(), 3);
            assert_approx_eq!(pts[0].0, 0.0);
            assert_approx_eq!(pts[1].0, 1.0 / 2.);
            assert_approx_eq!(pts[2].0, 1.0);
        });
    }

    #[test]
    fn test_vec_f32_points_y_values_preserved() {
        let data = vec![0.1, 0.5, 0.9];
        data.with_points(3., |iter| {
            let ys: Vec<f32> = iter.map(|(_, y)| y).collect();
            assert_approx_eq!(ys[0], 0.1);
            assert_approx_eq!(ys[1], 0.5);
            assert_approx_eq!(ys[2], 0.9);
        });
    }

    #[test]
    fn test_vec_tuple_points_passthrough() {
        let data = vec![(0.1, 0.2), (0.5, 0.6), (0.9, 1.0)];
        data.with_points(3., |iter| {
            let pts: Vec<_> = iter.collect();
            assert_eq!(pts, vec![(0.1, 0.2), (0.5, 0.6), (0.9, 1.0)]);
        });
    }

    #[test]
    fn test_fn_data_samples_at_resolution() {
        let f = |x: f32| x * 2.0;
        f.with_points(5.0, |iter| {
            let pts: Vec<_> = iter.collect();
            assert_eq!(pts.len(), 5);
            for (x, y) in pts {
                assert_approx_eq!(y, x * 2.0);
            }
        });
    }
}
