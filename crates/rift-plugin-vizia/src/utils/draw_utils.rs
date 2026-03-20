use vizia::{
    prelude::DrawContext,
    style::Color,
    vg::{Canvas, ClipOp, Path, Rect},
};

use crate::utils::ViewportTransform;

pub fn make_strokepath(
    points: impl Iterator<Item = (f32, f32)>,
    vtransform: &ViewportTransform,
) -> Option<Path> {
    let mut path = Path::new();
    let mut points = points.map(|(x, y)| vtransform.transform(x, y));

    if let Some((x, y)) = points.next() {
        path.move_to((x, y));

        for (x, y) in points {
            path.line_to((x, y));
        }

        Some(path)
    } else {
        None
    }
}

pub fn close_path(path: &mut Path, vtransform: &ViewportTransform, y_low: f32) {
    let Some(first_point) = path.get_point(0) else {
        return;
    };

    let Some(last_point) = path.last_pt() else {
        return;
    };

    let (_, zero_y) = vtransform.transform(0., y_low);
    path.line_to((last_point.x, zero_y))
        .line_to((first_point.x, zero_y))
        .close();
}

pub fn change_color_opacity(color: Color, opacity: u8) -> Color {
    Color::rgba(color.r(), color.g(), color.b(), opacity)
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
    fn test_empty_iterator_returns_none() {
        assert!(make_strokepath(std::iter::empty(), &identity()).is_none());
    }

    #[test]
    fn test_single_point_creates_path() {
        let points = vec![(0.5, 0.5)];
        assert!(make_strokepath(points.into_iter(), &identity()).is_some());
    }

    #[test]
    fn test_multiple_points_creates_path() {
        let points = vec![(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)];
        assert!(make_strokepath(points.into_iter(), &identity()).is_some());
    }

    #[test]
    fn test_change_color_opacity_sets_alpha() {
        let color = Color::rgb(100, 150, 200);
        let result = change_color_opacity(color, 128);
        assert_eq!(result.r(), 100);
        assert_eq!(result.g(), 150);
        assert_eq!(result.b(), 200);
        assert_eq!(result.a(), 128);
    }

    #[test]
    fn test_change_color_opacity_zero() {
        let color = Color::rgb(255, 255, 255);
        let result = change_color_opacity(color, 0);
        assert_eq!(result.a(), 0);
    }

    #[test]
    fn test_change_color_opacity_full() {
        let color = Color::rgb(255, 255, 255);
        let result = change_color_opacity(color, 255);
        assert_eq!(result.a(), 255);
    }
}
