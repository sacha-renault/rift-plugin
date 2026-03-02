use std::{cell::RefCell, sync::Arc};

use hug_accumulator::{AudioAccumulator, AudioConsumer};
use vizia::vg;

use super::gui_prelude::*;

pub struct Oscilloscope<LACC>
where
    LACC: Lens<Target = Arc<AudioAccumulator<128>>>,
{
    accumulator: LACC,
    buffer: RefCell<WindowedBuffer>,
}

impl<LACC> View for Oscilloscope<LACC>
where
    LACC: Lens<Target = Arc<AudioAccumulator<128>>>,
{
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        // Push new samples into buckets
        let acc = self.accumulator.get(cx);

        let mut buckets = self.buffer.borrow_mut();
        acc.drain(|block, infos, time| buckets.consume(block, infos, time));
        drop(buckets);

        cx.draw_background(canvas);
        clip_bounds(cx, canvas);
        self.draw_stroke(cx, canvas);
        self.draw_fill(cx, canvas);
    }
}

impl<LACC> Oscilloscope<LACC>
where
    LACC: Lens<Target = Arc<AudioAccumulator<128>>>,
{
    pub fn new<LBPM>(
        cx: &mut Context,
        accumulator: LACC,
        samplerate: impl Res<f64>,
        n_beats: impl Res<f64>,
        bpm: LBPM,
    ) -> Handle<'_, Self>
    where
        LBPM: Lens<Target = f64>,
    {
        let mut buffer = WindowedBuffer::new(samplerate.get(cx), 900, n_beats.get(cx), bpm.get(cx));
        buffer.set_beats(1.0);

        let mut handle = Self {
            accumulator,
            buffer: RefCell::new(buffer),
        }
        .build(cx, |_| {});
        let entity = handle.entity();

        // num writes will tick everytime
        // the audio thread writes data
        Binding::new(
            handle.context(),
            accumulator.map(|acc| acc.num_writes()),
            move |cx, _| {
                cx.needs_redraw(entity);
            },
        );

        // num writes will tick everytime
        // the audio thread writes data
        Binding::new(handle.context(), bpm, move |cx, bpm| {
            let new_bpm = bpm.get(cx);

            if let Some(view_state) = cx.data::<Self>() {
                view_state.buffer.borrow_mut().set_tempo(new_bpm);
                cx.needs_redraw(entity);
            }
        });

        handle
    }

    fn draw_stroke(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let buckets = self.buffer.borrow();
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
        let buckets = self.buffer.borrow();
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
