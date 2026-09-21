/// Return evenly spaced numbers over a specified interval.
///
/// Returns num evenly spaced samples, calculated over the interval [start, stop].
pub struct Linspace {
    start: f32,
    step: f32,
    num_points: usize,
    current_index: usize,
}

impl Linspace {
    /// Create the [`Linspace`] iterator.
    ///
    /// # Arguments:
    /// - start: starting value of the sequence
    /// - stop: end of the sequence
    /// - num_points: number of points that will be created
    ///
    /// # Panics:
    /// - if number of points < 2
    pub fn new(start: f32, end: f32, num_points: usize) -> Self {
        assert!(num_points >= 2, "num_points must be at least 2");
        let step = (end - start) / (num_points - 1) as f32;
        Self {
            start,
            step,
            num_points,
            current_index: 0,
        }
    }
}

impl Iterator for Linspace {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.num_points {
            let out = self.start + self.step * self.current_index as f32;
            self.current_index += 1;
            Some(out)
        } else {
            None
        }
    }
}

/// Generate a sequence of numbers spaced evenly on a logarithmic scale.
///
/// The sequence is generated between `start` and `end`
/// with `num_points` points, using the specified `base` for the logarithm.
pub struct Logspace {
    linspace: Linspace,
    start: f32,
    ratio: f32,
}

impl Logspace {
    /// Create the [`Logspace`] iterator
    ///
    /// # Arguments
    ///
    /// - `min`: the minimum value (must be > 0)
    /// - `max`: the maximum value (must be > 0)
    /// - `num_points`: number of points to generate (must be >= 2)
    /// # Panics
    ///
    /// - If `min` or `max` are less than or equal to zero.
    /// - If `num_points` < 2.
    pub fn new(start: f32, end: f32, num_points: usize) -> Self {
        assert!(start > 0.0, "start must be greater than zero");
        assert!(end > 0.0, "end must be greater than zero");
        let ratio = end / start;
        Self {
            ratio,
            linspace: Linspace::new(0.0, 1.0, num_points),
            start,
        }
    }
}

impl Iterator for Logspace {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.linspace
            .next()
            .map(|exp| self.start * self.ratio.powf(exp))
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_approx_eq;

    use super::*;

    #[test]
    fn test_linspace() {
        let iterator = Linspace::new(0., 1., 3);
        assert_eq!(iterator.collect::<Vec<_>>(), vec![0., 0.5, 1.]);
    }

    #[test]
    fn test_linspace_rev() {
        let iterator = Linspace::new(1., 0., 3);
        assert_eq!(iterator.collect::<Vec<_>>(), vec![1., 0.5, 0.,]);
    }

    #[test]
    #[should_panic]
    fn test_linspace_panic() {
        // Panics for linspace on a single point or less
        Linspace::new(1., 0., 1);
    }

    #[test]
    fn test_logspace_endpoints() {
        let points: Vec<f32> = Logspace::new(1.0, 100.0, 3).collect();
        assert_approx_eq!(points[0], 1.0, 1e-4);
        assert_approx_eq!(points[2], 100.0, 1e-4);
    }

    #[test]
    fn test_logspace_midpoint_is_geometric_mean() {
        // For [1, 100] with 3 points, the middle should be sqrt(1 * 100) = 10
        let points: Vec<f32> = Logspace::new(1.0, 100.0, 3).collect();
        assert_approx_eq!(points[1], 10.0, 1e-4);
    }

    #[test]
    fn test_logspace_num_points() {
        for &n in &[2_usize, 10, 100] {
            assert_eq!(Logspace::new(1.0, 1000.0, n).count(), n);
        }
    }

    #[test]
    fn test_logspace_all_positive() {
        Logspace::new(0.1, 1000.0, 50)
            .for_each(|v| assert!(v > 0.0, "expected positive value, got {v}"));
    }

    #[test]
    fn test_logspace_is_monotonically_increasing() {
        let points: Vec<f32> = Logspace::new(1.0, 1000.0, 20).collect();
        points.windows(2).for_each(|w| {
            assert!(
                w[1] > w[0],
                "expected monotonic increase: {} <= {}",
                w[1],
                w[0]
            );
        });
    }

    #[test]
    #[should_panic]
    fn test_logspace_panic_zero_start() {
        Logspace::new(0.0, 100.0, 10);
    }

    #[test]
    #[should_panic]
    fn test_logspace_panic_zero_end() {
        Logspace::new(1.0, 0.0, 10);
    }
}
