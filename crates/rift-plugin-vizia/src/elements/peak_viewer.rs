use rift_plugin_accumulator::AudioPeaks;
use rift_plugin_shared::{RcCell, utils::db_conversion::linear_to_db};
use vizia::vg::{self, Rect};

use super::gui_prelude::*;

#[derive(HandleExtension)]
pub struct PeakViewer {
    data: RcCell<AudioPeaks>,

    #[extension(ext)]
    box_width: f32,

    #[extension(ext)]
    spacing: f32,

    #[extension(ext)]
    range: (f32, f32),
}

impl View for PeakViewer {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let num_channels = self.data.borrow().num_channels();
        self.draw_outer_rect(cx, canvas, num_channels);
        self.draw_peaks(cx, canvas, num_channels);
    }
}

impl PeakViewer {
    pub fn new(cx: &mut Context, data: RcCell<AudioPeaks>) -> Handle<'_, Self> {
        Self {
            data,
            box_width: 1.0,
            spacing: 0.05,
            range: (-120., 6.),
        }
        .build(cx, |_| {})
    }

    pub fn draw_outer_rect(&self, cx: &mut DrawContext, canvas: &Canvas, num_channels: usize) {
        let vtransform = ViewportTransform::new(cx.bounds());
        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_stroke_cap(vg::PaintCap::Square);
        paint.set_style(vg::PaintStyle::Stroke);
        paint.set_stroke_width(self.box_width);

        let num_spaces = (num_channels - 1) as f32;
        let width_per_viewer = (1.0 - num_spaces * self.spacing) / num_channels as f32;
        let mut current_position = 0.0;

        for _ in 0..num_channels {
            let (left, bottom) = vtransform.transform(current_position, 0.);
            let (right, top) = vtransform.transform(current_position + width_per_viewer, 1.0);

            let rect = Rect {
                left,
                top,
                right,
                bottom,
            };
            canvas.draw_rect(rect, &paint);

            current_position += width_per_viewer + self.spacing;
        }
    }

    pub fn draw_peaks(&self, cx: &mut DrawContext, canvas: &Canvas, num_channels: usize) {
        let vtransform = ViewportTransform::with_range(cx.bounds(), (0., 1.), self.range);
        let (min, max) = self.range;
        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_stroke_cap(vg::PaintCap::Square);
        paint.set_style(vg::PaintStyle::StrokeAndFill);
        paint.set_stroke_width(self.box_width);

        let num_spaces = (num_channels - 1) as f32;
        let width_per_viewer = (1.0 - num_spaces * self.spacing) / num_channels as f32;
        let mut current_position = 0.0;

        for channel in 0..num_channels {
            let Some(peak) = self.data.borrow().peak(channel) else {
                break;
            };
            let db_peak = linear_to_db(peak).clamp(min, max);

            let (left, bottom) = vtransform.transform(current_position, min);
            let (right, top) = vtransform.transform(current_position + width_per_viewer, db_peak);

            let rect = Rect {
                left,
                top,
                right,
                bottom,
            };
            canvas.draw_rect(rect, &paint);
            current_position += width_per_viewer + self.spacing;
        }
    }
}
