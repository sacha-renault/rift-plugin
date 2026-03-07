/// Linear interpolation between two points
#[inline]
pub fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

/// Lerp between two values in a slice
///
/// # Panics:
/// - if float_idx.floor() is out of `buf` range.
#[inline]
pub fn lerp_array(buf: &[f32], float_idx: f32) -> f32 {
    let prev = float_idx.floor() as usize;
    let next = prev + 1;
    if buf.len() < prev {
        lerp(buf[prev], buf[next], float_idx.fract())
    } else {
        buf[prev]
    }
}

/// Smooth cubic interpolation for 1D frequency spectra.
///
/// Returns the interpolated value smoothly blending 4 neighbors.
/// This uses Catmull-Rom coefficients.
/// <https://en.wikipedia.org/wiki/Catmull%E2%80%93Rom_spline>
#[inline]
pub fn cubic_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let a0 = -0.5 * x0 + 1.5 * x1 - 1.5 * x2 + 0.5 * x3;
    let a1 = x0 - 2.5 * x1 + 2.0 * x2 - 0.5 * x3;
    let a2 = -0.5 * x0 + 0.5 * x2;
    let a3 = x1;

    (a0 * (t * t * t)) + (a1 * (t * t)) + (a2 * t) + (a3)
}

/// Converts an audio sample amplitude to decibels.
///
/// # Notes:
/// This function will clamp any value below 1e-6, thus anything below 1e-6
/// will be neg infinity. No processing should be done with this result.
#[inline]
pub fn linear_to_db(sample: f32) -> f32 {
    if sample < 1e-6 {
        f32::NEG_INFINITY
    } else {
        // When sample is 0. this function will return -120dB
        // which is silence ...
        sample.abs().log10() * 20.0
    }
}

/// Converts decibel value back to linear audio sample amplitude.
///
/// # Notes:
/// This is the inverse of [`linear_to_db`]. Values below -120dB are clamped
/// to 0.0, treating extreme attenuation as complete silence.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    if db < -120. { 0. } else { 10f32.powf(db / 20.) }
}
