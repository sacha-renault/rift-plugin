use vizia::{
    prelude::DrawContext,
    vg::{Canvas, ClipOp, Path, Point, Rect},
};

use crate::utils::ViewportTransform;

pub struct PathWithClosing {
    pub path: Path,
    pub closing_points: [(f32, f32); 2],
}

pub fn make_strokepath(
    points: impl Iterator<Item = (f32, f32)>,
    vtransform: ViewportTransform,
    x_low: f32,
) -> Option<PathWithClosing> {
    let mut path = Path::new();
    let mut points = points.map(|(x, y)| vtransform.transform(x, y));
    let (_, zero_y) = vtransform.transform(0., x_low);

    if let Some((x, y)) = points.next() {
        let first_x = x;

        path.move_to((x, y));

        for (x, y) in points {
            path.line_to((x, y));
        }

        let last_x = if let Some(Point { x, .. }) = path.last_pt() {
            x
        } else {
            // Very defensive, i might unwrap here since we have at least
            // a point but i don't like unwraping
            first_x
        };

        Some(PathWithClosing {
            path,
            closing_points: [(last_x, zero_y), (first_x, zero_y)],
        })
    } else {
        None
    }
}

pub fn clip_bounds(cx: &mut DrawContext, canvas: &Canvas) {
    let bounds = cx.bounds();
    let mut clip_path = Path::new();
    clip_path.add_rect(
        Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
        None,
    );
    canvas.clip_path(&clip_path, ClipOp::Intersect, true);
}
