use rift_plugin_accumulator::prelude::AudioPeaks;
use rift_plugin_shared::{
    RcCell,
    utils::conversion::{linear_to_db, normalize_by_range},
};
use vizia::vg::{Paint, PaintCap, PaintStyle, Rect};

use super::gui_prelude::*;

/// A UI Element to display audio peaks on one or more channels
///
/// # Examples:
/// ```ignore
/// PeaksViewer::new(audio_peaks.clone())
///     .range((-54., 6.))
///     .graduations(vec![3., 0., -3., -6., -12., -18., -30.].into())
///     .build_view(cx)
///     .redraw_on(redraw_lens);
/// ```
#[derive(ParamViewBuilder)]
pub struct PeaksViewer {
    #[builder(new)]
    data: RcCell<AudioPeaks>,

    /// Range of the peak meter (in db).
    #[builder(default = (-120., 6.))]
    range: (f32, f32),

    /// A vec representing the graduations
    #[builder(default = vec![])]
    graduations: Vec<f32>,
}

impl DestructThenBuildView for PeaksViewer {
    fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            data,
            range,
            graduations,
        } = self;

        ZStack::new(cx, |cx| {
            PeakGraduation { range, graduations }
                .build(cx, |_| {})
                .class("peak-graduation");

            HStack::new(cx, |cx| {
                for i in 0..data.borrow().num_channels() {
                    PeakAmplitude::new(cx, data.clone(), i, range).class("peak-amplitude");
                }
            })
            .class("peak-viewer-inner");
        })
        .class("peak-viewer")
    }
}

/// Draw the amplitude of a single peak
///
/// # TODO:
/// This component might overlap on the border of parent if no padding is added.
struct PeakAmplitude {
    data: RcCell<AudioPeaks>,
    channel: usize,
    range: (f32, f32),
}

impl PeakAmplitude {
    fn new(
        cx: &mut Context,
        data: RcCell<AudioPeaks>,
        channel: usize,
        range: (f32, f32),
    ) -> Handle<'_, Self> {
        Self {
            data,
            channel,
            range,
        }
        .build(cx, |_| {})
    }
}

/// Simply draw the amplitude based on peak value
///
/// # TODO:
/// We might wanna add some color palette here!
impl View for PeakAmplitude {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let Some(peak) = self.data.borrow().peak(self.channel) else {
            return;
        };

        let (min, max) = self.range;
        let vtransform = ViewportTransform::with_range(cx.bounds(), (0., 1.), self.range);
        let mut paint = Paint::default();

        paint.set_color(cx.font_color());
        paint.set_stroke_cap(PaintCap::Square);
        paint.set_style(PaintStyle::Fill);
        let db_peak = linear_to_db(peak).clamp(min, max);

        let (left, bottom) = vtransform.transform(0., min);
        let (right, top) = vtransform.transform(1., db_peak);

        let rect = Rect {
            left,
            top,
            right,
            bottom,
        };
        canvas.draw_rect(rect, &paint);
    }
}

/// Graduation to be draw under the peaks
struct PeakGraduation {
    graduations: Vec<f32>,
    range: (f32, f32),
}

impl View for PeakGraduation {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let vtransform = ViewportTransform::new(cx.bounds());

        let mut paint = Paint::default();
        paint.set_color(cx.border_color());
        paint.set_stroke_cap(PaintCap::Square);
        paint.set_style(PaintStyle::Fill);
        paint.set_stroke_width(cx.border_width());

        let (start, end) = self.range;
        let range = end - start;

        let chunk_start = cx.padding_left().to_px(cx.bounds().w, 0.);
        let chunk_end = cx.padding_right().to_px(cx.bounds().w, 0.);

        for &graduation in self.graduations.iter() {
            let grad_height = normalize_by_range(graduation, start, range);

            let mut left = vtransform.transform(0., grad_height);
            let mut right = vtransform.transform(1.0, grad_height);
            left.0 += chunk_start;
            right.0 -= chunk_end;

            canvas.draw_line(left, right, &paint);
        }
    }
}
