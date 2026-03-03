use vizia::{
    prelude::DrawContext,
    vg::{self, Canvas},
};

use super::Denormalizer;

pub fn make_open_strokepath(
    denorm: Denormalizer,
    points: impl Iterator<Item = f32>,
    num_points: usize,
) -> vg::path::Path {
    let mut path = vg::Path::new();
    let total = num_points as f32;

    let mut points = points
        .enumerate()
        .map(|(x, y)| denorm.denormalize(x as f32 / total, y));

    if let Some((x, y)) = points.next() {
        path.move_to((x, y));

        for (x, y) in points {
            path.line_to((x, y));
        }
    }

    path
}

pub fn make_closed_strokepath(
    denorm: Denormalizer,
    points: impl Iterator<Item = f32>,
    num_points: usize,
) -> vg::path::Path {
    let mut path = vg::Path::new();
    let total = num_points as f32;

    let points = points
        .enumerate()
        .map(|(x, y)| denorm.denormalize(x as f32 / total, y));

    path.move_to(denorm.denormalize(0., 0.));
    for (x, y) in points {
        path.line_to((x, y));
    }

    if let Some(point) = path.last_pt() {
        let (_, y) = denorm.denormalize(1.0, 0.);
        path.line_to((point.x, y));
    }

    path.close();

    path
}

pub fn clip_bounds(cx: &mut DrawContext, canvas: &Canvas) {
    let bounds = cx.bounds();
    let mut clip_path = vg::Path::new();
    clip_path.add_rect(
        vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
        None,
    );
    canvas.clip_path(&clip_path, vg::ClipOp::Intersect, true);
}
