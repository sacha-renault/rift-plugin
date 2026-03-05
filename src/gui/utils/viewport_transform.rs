use vizia::layout::BoundingBox;

/// Maps data-space coordinates into screen-space coordinates within a bounding box.
///
/// # Coordinate contract
/// - `x` — expected to already be **normalized** in `[0.0, 1.0]`, where `0.0` is the left
///   edge and `1.0` is the right edge of the viewport.
/// - `y` — provided in **raw data units** (e.g. amplitude values). The transform normalizes
///   it internally using the `[min, max]` range supplied at construction, then flips the axis
///   so that larger values appear *higher* on screen (screen-space y grows downward).
pub struct ViewportTransform {
    bounds: BoundingBox,
    min: f32,
    range: f32,
}

impl ViewportTransform {
    pub fn new(bounds: BoundingBox, min: f32, max: f32) -> Self {
        let range = max - min;
        Self { bounds, min, range }
    }

    /// Transforms a data-space point into a screen-space point.
    ///
    /// - `x_norm`: normalized horizontal position in `[0.0, 1.0]`.
    /// - `y`: raw data value in `[min, max]` (clamping is not enforced).
    ///
    /// Returns `(screen_x, screen_y)` in pixels, offset by the viewport origin.
    pub fn transform(&self, x_norm: f32, y: f32) -> (f32, f32) {
        // Normalize y into [0, 1], then flip so that higher values go upward on screen.
        // Screen-space y increases downward, so we subtract from 1.0 before scaling.
        let y_norm = 1.0 - (y - self.min) / self.range;
        (
            x_norm * self.bounds.w + self.bounds.x,
            y_norm * self.bounds.h + self.bounds.y,
        )
    }
}
