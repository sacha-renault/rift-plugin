use vizia::layout::BoundingBox;

pub struct Denormalizer {
    bounds: BoundingBox,
    min: f32,
    range: f32,
}

impl Denormalizer {
    pub fn new(bounds: BoundingBox, min: f32, max: f32) -> Self {
        let range = max - min;
        Self { bounds, min, range }
    }

    /// TODO, use min + range
    pub fn denormalize(&self, x: f32, y: f32) -> (f32, f32) {
        let y = 1.0 - (y + 1.0) / 2.0;
        (
            x * self.bounds.w + self.bounds.x,
            y * self.bounds.h + self.bounds.y,
        )
    }
}
