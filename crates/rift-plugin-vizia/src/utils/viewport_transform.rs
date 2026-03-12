use rift_plugin_shared::utils::conversion::normalize_by_range;
use vizia::layout::BoundingBox;

/// Maps data-space coordinates into screen-space coordinates within a bounding box.
pub struct ViewportTransform {
    bounds: BoundingBox,
    x_min: f32,
    x_range: f32,
    y_min: f32,
    y_range: f32,
}

impl ViewportTransform {
    pub fn new(bounds: BoundingBox) -> Self {
        Self::with_range(bounds, (0., 1.), (0., 1.))
    }
    pub fn with_range(
        bounds: BoundingBox,
        (x_min, x_max): (f32, f32),
        (y_min, y_max): (f32, f32),
    ) -> Self {
        Self {
            bounds,
            x_min,
            x_range: x_max - x_min,
            y_min,
            y_range: y_max - y_min,
        }
    }

    #[inline]
    pub fn transform(&self, x: f32, y: f32) -> (f32, f32) {
        // flip so that higher values go upward on screen.
        // Screen-space y increases downward, so we subtract from 1.0 before scaling.
        let y_norm = 1.0 - normalize_by_range(y, self.y_min, self.y_range);
        let x_norm = normalize_by_range(x, self.x_min, self.x_range);
        (
            x_norm.clamp(0., 1.) * self.bounds.w + self.bounds.x,
            y_norm.clamp(0., 1.) * self.bounds.h + self.bounds.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let view_transform = ViewportTransform::new(BoundingBox {
            x: 0.,
            y: 0.,
            w: 10.,
            h: 10.,
        });

        assert_eq!(view_transform.transform(0.5, 0.), (5., 10.)); // y is flipped
        assert_eq!(view_transform.transform(1.5, -1.0), (10., 10.)); // x and y are clamped
    }
}
