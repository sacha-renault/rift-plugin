#[macro_export]
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => {{
        let diff = ($a - $b).abs();
        let delta = 1e-6;
        if diff > delta {
            panic!(
                "assert_approx_eq failed: {} ~= {} (diff: {}, default tolerance: {})",
                $a, $b, diff, delta
            );
        }
    }};
    ($a:expr, $b:expr, $delta:expr) => {{
        let diff = ($a - $b).abs();
        if diff > $delta {
            panic!(
                "assert_approx_eq failed: {} ~= {} (diff: {}, tolerance: {})",
                $a, $b, diff, $delta
            );
        }
    }};
}
