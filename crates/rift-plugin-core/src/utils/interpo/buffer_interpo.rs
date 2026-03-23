use super::function_interpo::*;

/// Interpolate at a fractional index using a 2-sample kernel.
#[inline]
pub fn interpolate_buffer2(buf: &[f32], x: f32, kernel: fn(f32, f32, f32) -> f32) -> f32 {
    let len = buf.len();
    assert_ne!(len, 0);
    if len == 1 || x <= 0.0 {
        return buf[0];
    }
    if x >= (len - 1) as f32 {
        return buf[len - 1];
    }

    let i = x.floor() as usize;
    let t = x.fract();
    if t == 0.0 {
        return buf[i];
    }

    kernel(buf[i], buf[i + 1], t)
}

/// Interpolate at a fractional index using a 4-sample kernel.
///
/// See [`catmull_interpolate_buffer`] implementation to see how to use.
#[inline]
pub fn interpolate_buffer4(buf: &[f32], x: f32, kernel: fn(f32, f32, f32, f32, f32) -> f32) -> f32 {
    let len = buf.len();
    assert_ne!(len, 0);
    if len == 1 || x <= 0.0 {
        return buf[0];
    }
    if x >= (len - 1) as f32 {
        return buf[len - 1];
    }

    let i = x.floor() as usize;
    let t = x.fract();
    if t == 0.0 {
        return buf[i];
    }

    let x0 = buf[i.saturating_sub(1)];
    let x1 = buf[i];
    let x2 = buf[(i + 1).min(len - 1)];
    let x3 = buf[(i + 2).min(len - 1)];

    kernel(x0, x1, x2, x3, t)
}

/// Linearly interpolate at a fractional index in a slice.
///
/// Boundary behavior:
/// - Values outside `[0, len-1]` are clamped to the nearest element.
///
/// Uses [`lerp`] under the hood.
///
/// # Panics
/// - If `array` is empty.
#[inline]
pub fn lerp_interpolate_buffer(array: &[f32], x: f32) -> f32 {
    interpolate_buffer2(array, x, lerp)
}

/// Retrieve a value at a fractional index in a slice using cubic interpolation.
///
/// Boundary behavior:
/// - Values outside `[0, len-1]` are clamped to the nearest element.
/// - At the edges, the missing neighbors are repeated.
///
/// Uses [`cubic_catmull_interpolate`] under the hood.
///
/// # Panics
/// - If `array` is empty.
#[inline]
pub fn catmull_interpolate_buffer(array: &[f32], x: f32) -> f32 {
    interpolate_buffer4(array, x, cubic_catmull_interpolate)
}
