use vizia::{layout::BoundingBox, prelude::DrawContext};

pub struct Denormalizer {
    bounds: BoundingBox,
}

impl Denormalizer {
    pub fn from_cx(cx: &mut DrawContext) -> Self {
        Self {
            bounds: cx.bounds(),
        }
    }

    pub fn denormalize(&self, x: f32, y: f32) -> (f32, f32) {
        let y = 1.0 - (y + 1.0) / 2.0;
        (
            x * self.bounds.w + self.bounds.x,
            y * self.bounds.h + self.bounds.y,
        )
    }
}
