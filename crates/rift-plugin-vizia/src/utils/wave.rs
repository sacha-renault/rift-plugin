use rift_plugin_accumulator::prelude::Bucket;
use vizia::vg::{RRect, Rect};

use crate::dev_prelude::*;

pub fn draw_rms_buckets<'a, I, B>(cx: &mut DrawContext, canvas: &Canvas, shift: f32, buckets: I)
where
    I: Iterator<Item = &'a B>,
    B: Bucket,
{
    let BoundingBox { x, y, w, h } = cx.bounds();

    let scale = cx.scale_factor();
    let Slots { accent, .. } = Slots::read(cx);

    let paint = fill_paint(accent);

    let x0 = x.round();
    let right = (x + w).round();
    let center = y + h * 0.5;
    let half = h * 0.5;
    let min_height = scale.max(1.0).min(h);

    // TODO: rendering is bucket-dependent. PeakBucket wants min/max rects/step path.
    // RmsBucket's min/max are synthetic ±rms, so a filled top/bottom path looks
    // smoother. A single filled path over min/max would work for both and cut draw calls.
    for (i, bucket) in buckets.enumerate() {
        if bucket.count() == 0 {
            continue;
        }
        let (lo, hi) = (bucket.min(), bucket.max());
        if !lo.is_finite() || !hi.is_finite() {
            continue;
        }

        // Always an integer: left edge and width are whole pixels.
        let col_left = right - shift - i as f32;
        let left = col_left.max(x0);
        let col_right = (col_left + 1.0).min(right);
        if col_right <= left {
            continue;
        }

        let mut top = center - hi.clamp(-1.0, 1.0) * half;
        let mut bottom = center - lo.clamp(-1.0, 1.0) * half;
        if bottom - top < min_height {
            let mid = (top + bottom) * 0.5;
            top = mid - min_height * 0.5;
            bottom = mid + min_height * 0.5;
        }
        let top = top.max(y);
        let bottom = bottom.min(y + h);
        if bottom <= top {
            continue;
        }

        let rect = Rect::from_xywh(left, top, col_right - left, bottom - top);
        canvas.draw_rrect(RRect::new_rect_xy(rect, 0.0, 0.0), &paint);
    }
}
