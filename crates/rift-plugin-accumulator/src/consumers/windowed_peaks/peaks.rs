use crate::consumers::windowed_peaks::window::Bucket;

#[derive(Clone)]
pub struct PeakBucket {
    min: f32,
    max: f32,
    count: usize,
}

impl Bucket for PeakBucket {
    #[allow(dead_code)]
    fn new(x: f32) -> Self {
        Self {
            min: x,
            max: x,
            count: 1,
        }
    }

    fn empty() -> Self {
        PeakBucket {
            min: 0.,
            max: 0.,
            count: 0,
        }
    }

    #[inline]
    fn add_sample(&mut self, x: f32) {
        if self.count == 0 {
            (self.min, self.max) = (x, x);
            self.count += 1;
        } else {
            self.count += 1;

            if x > self.max {
                self.max = x
            } else if x < self.min {
                self.min = x;
            }
        }
    }

    #[inline]
    fn value(&self) -> f32 {
        if self.min.abs() > self.max.abs() {
            self.min
        } else {
            self.max
        }
    }

    #[inline]
    fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let bucket = PeakBucket::new(5.);

        assert_eq!(bucket.max, 5.);
        assert_eq!(bucket.min, 5.);
        assert_eq!(bucket.count, 1);
    }

    #[test]
    fn test_empty() {
        let bucket = PeakBucket::empty();

        assert_eq!(bucket.max, 0.);
        assert_eq!(bucket.min, 0.);
        assert_eq!(bucket.count(), 0);
    }

    #[test]
    fn test_add() {
        let mut bucket = PeakBucket::empty();

        bucket.add_sample(1.);
        assert_eq!(bucket.max, 1.);
        assert_eq!(bucket.min, 1.);
        assert_eq!(bucket.count(), 1);

        bucket.add_sample(2.);
        assert_eq!(bucket.max, 2.);
        assert_eq!(bucket.min, 1.);
        assert_eq!(bucket.count(), 2);
        assert_eq!(bucket.value(), 2.);

        bucket.add_sample(-3.);
        assert_eq!(bucket.max, 2.);
        assert_eq!(bucket.min, -3.);
        assert_eq!(bucket.count(), 3);
        assert_eq!(bucket.value(), -3.);
    }
}
