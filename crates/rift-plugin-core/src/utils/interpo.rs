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

/// Lerp between two values in a slice
///
/// # Panics:
/// - if float_idx.floor() is out of `buf` range.
#[inline]
pub fn lerp_array(buf: &[f32], float_idx: f32) -> f32 {
    let prev = float_idx.floor() as usize;
    let next = prev + 1;
    if next < buf.len() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx_eq;

    #[test]
    fn test_lerp() {
        assert_approx_eq!(lerp(0., 1., 0.5), 0.5);
        assert_approx_eq!(lerp(10., 20., 0.8), 18.);
    }

    #[test]
    fn test_lerp_n_times() {
        let current = 0.;
        let number = 100;
        let target = 10.;
        let factor = 0.01;

        let oneshot_lerp = lerp_n(current, target, factor, number);
        let fold_lerp = (0..number).fold(current, |v, _| lerp(v, target, factor));

        assert_approx_eq!(oneshot_lerp, fold_lerp, 1e-5)
    }

    #[test]
    #[should_panic]
    fn test_lerp_array_out_of_range() {
        let array = vec![0., 1., 2.];
        lerp_array(&array, 4.);
    }

    #[test]
    fn test_lerp_array() {
        let array = vec![0., 1., 2.];
        let value = lerp_array(&array, 1.5);
        assert_approx_eq!(value, 1.5);
    }

    #[test]
    fn test_lerp_array_last() {
        let array = vec![0., 1., 2.];
        let value = lerp_array(&array, 2.);
        assert_approx_eq!(value, 2.);
    }

    #[test]
    fn test_cubic_interpolate_start_point() {
        // At t=0, the spline should start exactly at x1
        let x0 = -1.0;
        let x1 = 0.0;
        let x2 = 1.0;
        let x3 = 2.0;

        assert_approx_eq!(cubic_interpolate(x0, x1, x2, x3, 0.0), x1);
    }

    #[test]
    fn test_cubic_interpolate_constant() {
        // If all points are the same, result should be that value regardless of t
        let val = 5.0;
        let result = cubic_interpolate(val, val, val, val, 0.2);
        assert_approx_eq!(result, val);
    }
}
