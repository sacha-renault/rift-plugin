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

/// Normalizes a value to the [0.0, 1.0] range.
///
/// # Notes:
/// If `min` >= `max`, prevents division by zero or reverse by returning `min`.
#[inline]
pub fn normalize(value: f32, min: f32, max: f32) -> f32 {
    normalize_by_range(value, min, max - min)
}

/// Normalizes a value relative to a minimum and a specific range size.
///
/// # Notes:
/// If `range` is  <= 0.0, prevents division by zero by returning `min`.
#[inline]
pub fn normalize_by_range(value: f32, min: f32, range: f32) -> f32 {
    if range <= 0. {
        min
    } else {
        (value - min) / range
    }
}

/// Denormalizes a value from the [0.0, 1.0] range back to its original scale.
#[inline]
pub fn denormalize(normalized: f32, min: f32, max: f32) -> f32 {
    denormalize_by_range(normalized, min, max - min)
}

/// Denormalizes a value from [0.0, 1.0] back to its original scale using a fixed range.
#[inline]
pub fn denormalize_by_range(normalized: f32, min: f32, range: f32) -> f32 {
    normalized * range + min
}
