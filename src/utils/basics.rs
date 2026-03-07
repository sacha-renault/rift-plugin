#[inline]
pub fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}
