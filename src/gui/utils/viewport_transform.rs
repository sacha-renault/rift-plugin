use vizia::layout::BoundingBox;

/// Maps data-space coordinates into screen-space coordinates within a bounding box.
pub struct ViewportTransform {
    bounds: BoundingBox,
}

impl ViewportTransform {
    pub fn new(bounds: BoundingBox) -> Self {
        Self { bounds }
    }

    #[inline]
    pub fn transform(&self, x_norm: f32, y_norm: f32) -> (f32, f32) {
        // flip so that higher values go upward on screen.
        // Screen-space y increases downward, so we subtract from 1.0 before scaling.
        let y_norm = 1.0 - y_norm;
        (
            x_norm * self.bounds.w + self.bounds.x,
            y_norm * self.bounds.h + self.bounds.y,
        )
    }
}
