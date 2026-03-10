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
            x_norm * self.bounds.w + self.bounds.x,
            y_norm * self.bounds.h + self.bounds.y,
        )
    }
}
