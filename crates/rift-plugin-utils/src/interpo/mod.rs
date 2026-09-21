mod buffer_interpo;
mod function_interpo;

pub use buffer_interpo::*;
pub use function_interpo::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx_eq;

    #[test]
    fn lerp_once() {
        assert_approx_eq!(lerp(0., 1., 0.5), 0.5);
        assert_approx_eq!(lerp(10., 20., 0.8), 18.);
    }

    #[test]
    fn lerp_n_times() {
        let current = 0.;
        let number = 100;
        let target = 10.;
        let factor = 0.01;

        let oneshot_lerp = lerp_n(current, target, factor, number);
        let fold_lerp = (0..number).fold(current, |v, _| lerp(v, target, factor));

        assert_approx_eq!(oneshot_lerp, fold_lerp, 1e-5)
    }

    #[test]
    fn lerp_array_out_of_range() {
        let array = vec![0., 1., 2.];
        let value = lerp_interpolate_buffer(&array, 4.);
        assert_approx_eq!(value, array.last().unwrap()); // approx eq to last value
    }

    #[test]
    fn lerp_array() {
        let array = vec![0., 1., 2.];
        let value = lerp_interpolate_buffer(&array, 1.5);
        assert_approx_eq!(value, 1.5);
    }

    #[test]
    fn lerp_array_last() {
        let array = vec![0., 1., 2.];
        let value = lerp_interpolate_buffer(&array, 2.);
        assert_approx_eq!(value, 2.);
    }

    #[test]
    fn cubic_catmull_interpolate_start_point() {
        // At t=0, the spline should start exactly at x1
        let x0 = -1.0;
        let x1 = 0.0;
        let x2 = 1.0;
        let x3 = 2.0;

        assert_approx_eq!(cubic_catmull_interpolate(x0, x1, x2, x3, 0.0), x1);
    }

    #[test]
    fn cubic_catmull_interpolate_constant() {
        // If all points are the same, result should be that value regardless of t
        let val = 5.0;
        let result = cubic_catmull_interpolate(val, val, val, val, 0.2);
        assert_approx_eq!(result, val);
    }

    #[test]
    fn cubic_catmull_interpolate_array_single_bin() {
        assert_eq!(catmull_interpolate_buffer(&[0.5], 0.0), 0.5);
    }

    #[test]
    fn cubic_catmull_interpolate_array_exact_index() {
        let bins = [0.0, 1.0, 2.0, 3.0];
        assert_approx_eq!(catmull_interpolate_buffer(&bins, 0.0), 0.0);
        assert_approx_eq!(catmull_interpolate_buffer(&bins, 1.0), 1.0);
        assert_approx_eq!(catmull_interpolate_buffer(&bins, 2.0), 2.0);
    }

    #[test]
    fn cubic_catmull_interpolate_array_clamps_below_zero() {
        let bins = [1.0, 2.0, 3.0];
        assert_approx_eq!(catmull_interpolate_buffer(&bins, -1.0), 1.0);
    }

    #[test]
    fn cubic_catmull_interpolate_array_clamps_above_max() {
        let bins = [1.0, 2.0, 3.0];
        assert_approx_eq!(catmull_interpolate_buffer(&bins, 99.0), 3.0);
    }

    #[test]
    fn cubic_catmull_interpolate_array_interpolates_midpoint() {
        // For a linear ramp, cubic interpolation at 0.5 between two points should be ~midpoint
        let bins = [0.0, 0.0, 1.0, 1.0];
        let mid = catmull_interpolate_buffer(&bins, 1.5);
        assert!((0.0..=1.0).contains(&mid));
    }
}
