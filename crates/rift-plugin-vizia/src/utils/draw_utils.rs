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

#[cfg(test)]
mod tests {
    use vizia::layout::BoundingBox;

    use super::*;
    use crate::utils::ViewportTransform;

    fn identity() -> ViewportTransform {
        ViewportTransform::new(BoundingBox {
            x: 0.,
            y: 0.,
            w: 1.,
            h: 1.,
        })
    }

    #[test]
    fn empty_iterator_returns_none() {
        assert!(make_strokepath(std::iter::empty(), identity(), 0.0).is_none());
    }

    #[test]
    fn single_point_closing_uses_same_x() {
        let res = make_strokepath(std::iter::once((3.0, 5.0)), identity(), 0.0).unwrap();
        let [a, b] = res.closing_points;
        assert_eq!(a.0, b.0);
        assert_eq!(a.1, b.1);
    }

    #[test]
    fn multiple_points_closing_spans_first_to_last() {
        let pts = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 0.5)];
        let res = make_strokepath(pts.into_iter(), identity(), 0.0).unwrap();
        let [a, b] = res.closing_points;
        assert!(a.0 > b.0);
        assert_eq!(a.1, b.1);
    }
}
