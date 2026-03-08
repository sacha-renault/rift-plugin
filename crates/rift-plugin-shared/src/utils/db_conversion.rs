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
