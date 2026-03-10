#[macro_export]
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => {{
        let diff = f32::abs($a - $b);
        const DEFAULT_DELTA: f32 = 1e-6;
        if diff > DEFAULT_DELTA {
            panic!(
                "assert_approx_eq failed: {} ~= {} (diff: {}, default tolerance: {})",
                $a, $b, diff, DEFAULT_DELTA
            );
        }
    }};
    ($a:expr, $b:expr, $delta:expr) => {{
        let diff = f32::abs($a - $b);
        if diff > $delta {
            panic!(
                "assert_approx_eq failed: {} ~= {} (diff: {}, tolerance: {})",
                $a, $b, diff, $delta
            );
        }
    }};
}
