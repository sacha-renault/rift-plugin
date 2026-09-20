/// Converts an audio sample amplitude to decibels.
///
/// **Notes**:
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
/// **Notes**:
/// This is the inverse of [`linear_to_db`]. Values below -120dB are clamped
/// to 0.0, treating extreme attenuation as complete silence.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    if db < -120. { 0. } else { 10f32.powf(db / 20.) }
}

/// Normalizes a value to the [0.0, 1.0] range.
///
/// **Notes**:
/// If `min` >= `max`, prevents division by zero or reverse by returning `min`.
#[inline]
pub fn normalize(value: f32, min: f32, max: f32) -> f32 {
    normalize_by_range(value, min, max - min)
}

/// Normalizes a value relative to a minimum and a specific range size.
///
/// **Notes**:
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx_eq;

    /// Tests the boundary condition for very low amplitudes.
    #[test]
    fn test_linear_to_db_clamp() {
        // Any value < 1e-6 should be treated as silence (-inf)
        assert_eq!(linear_to_db(0.0), f32::NEG_INFINITY);
        assert_eq!(linear_to_db(f32::EPSILON / 2.0), f32::NEG_INFINITY);

        // The exact threshold is still valid (not clamped, as logic checks strictly <)
        let result = linear_to_db(1e-6);
        assert_ne!(result, f32::NEG_INFINITY);
    }

    /// Tests the inverse relationship between the two conversion functions.
    #[test]
    fn test_linear_db_roundtrip() {
        let start_db = -100.0;
        let end_db = 10.0;
        let step = 5.0;
        let db_steps_start_inclusive: i32 = (start_db / step) as i32;
        let db_steps_end_inclusive: i32 = (end_db / step) as i32;

        for i in db_steps_start_inclusive..=db_steps_end_inclusive {
            let db_val = i as f32 * step;
            let linear = db_to_linear(db_val);

            // Re-converting back should return the original dB value within float tolerance
            let restored_db = linear_to_db(linear);

            // Use approximate equality because of floating point precision
            assert_approx_eq!(restored_db, db_val);
        }

        // Test that converting silence (-inf) is handled gracefully (though it stays -inf)
        assert_eq!(linear_to_db(f32::NEG_INFINITY), f32::NEG_INFINITY);
    }

    /// Tests normalization logic including clamping cases.
    #[test]
    fn test_normalize_edge_cases() {
        // Case: min >= max -> should return min (prevent div by zero/reverse)
        assert_eq!(normalize(0.5, 10.0, 5.0), 10.0);

        // Case: range <= 0 in normalize_by_range -> should return min
        assert_eq!(normalize_by_range(5.0, 0.0, -2.0), 0.0);
    }

    /// Tests denormalization and round-trip accuracy.
    #[test]
    fn test_denormalize_accuracy() {
        let min = 10.0;
        let range = 100.0; // Implies max of 110.0

        let value = 50.0; // 40% normalized
        let norm = normalize(value, min, min + range);
        let denorm = denormalize(norm, min, min + range);

        assert_approx_eq!(denorm, value);
    }

    /// Tests that the internal helper functions behave identically to their wrappers
    #[test]
    fn test_helper_equivalence() {
        let val = 5.0;
        let min = 0.0;
        let max = 10.0;

        assert_eq!(
            normalize(val, min, max),
            normalize_by_range(val, min, max - min)
        );
        assert_eq!(
            denormalize(val, min, max),
            denormalize_by_range(val, min, max - min)
        );
    }
}
