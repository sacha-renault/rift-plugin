/// Linear interpolation between two points
#[inline]
pub fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

/// Compute n linear interpolation
///
/// Equivalent to (0..number).fold(current, |v, _| lerp(v, target, factor))
#[inline]
pub fn lerp_n(current: f32, target: f32, factor: f32, number: i32) -> f32 {
    lerp(current, target, 1.0 - (1.0 - factor).powi(number))
}

/// Smooth cubic interpolation for 1D frequency spectra.
///
/// Returns the interpolated value smoothly blending 4 neighbors.
/// This uses Catmull-Rom coefficients.
/// <https://en.wikipedia.org/wiki/Catmull%E2%80%93Rom_spline>
#[inline]
pub fn cubic_catmull_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let a0 = -0.5 * x0 + 1.5 * x1 - 1.5 * x2 + 0.5 * x3;
    let a1 = x0 - 2.5 * x1 + 2.0 * x2 - 0.5 * x3;
    let a2 = -0.5 * x0 + 0.5 * x2;
    let a3 = x1;

    (a0 * (t * t * t)) + (a1 * (t * t)) + (a2 * t) + (a3)
}
