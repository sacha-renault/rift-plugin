/// Return evenly spaced numbers over a specified interval.
///
/// Returns num evenly spaced samples, calculated over the interval [start, stop].
pub struct Linespace {
    start: f32,
    step: f32,
    num_points: usize,
    current_index: usize,
}

impl Linespace {
    /// Create the [`Linespace`] iterator.
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

impl Iterator for Linespace {
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
    linespace: Linespace,
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
            linespace: Linespace::new(0.0, 1.0, num_points),
            start,
        }
    }
}

impl Iterator for Logspace {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.linespace
            .next()
            .map(|exp| self.start * self.ratio.powf(exp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linespace() {
        let iterator = Linespace::new(0., 1., 3);
        assert_eq!(iterator.collect::<Vec<_>>(), vec![0., 0.5, 1.]);
    }

    #[test]
    fn test_linespace_rev() {
        let iterator = Linespace::new(1., 0., 3);
        assert_eq!(iterator.collect::<Vec<_>>(), vec![1., 0.5, 0.,]);
    }

    #[test]
    #[should_panic]
    fn test_linespace_panic() {
        // Panics for linespace on a single point or less
        Linespace::new(1., 0., 1);
    }
}
