use crate::consumers::Bucket;

#[derive(Clone)]
pub struct PeakRmsBucket {
    min: f32,
    max: f32,
    sum_sq: f64,
    count: usize,
}

impl PeakRmsBucket {
    #[inline]
    pub fn min(&self) -> f32 {
        self.min
    }
    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }
    #[inline]
    pub fn rms(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_sq / self.count as f64).sqrt() as f32
        }
    }
}

impl Bucket for PeakRmsBucket {
    #[allow(dead_code)]
    fn new(x: f32) -> Self {
        Self {
            min: x,
            max: x,
            sum_sq: (x as f64) * (x as f64),
            count: 1,
        }
    }

    fn empty() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            sum_sq: 0.0,
            count: 0,
        }
    }

    #[inline]
    fn add_sample(&mut self, x: f32) {
        if self.count == 0 {
            self.min = x;
            self.max = x;
        } else {
            if x > self.max {
                self.max = x;
            }
            if x < self.min {
                self.min = x;
            }
        }
        self.sum_sq += (x as f64) * (x as f64);
        self.count += 1;
    }

    /// Larger-magnitude extreme, same as `PeakBucket` (mirrored look).
    #[inline]
    fn value(&self) -> f32 {
        if self.min.abs() > self.max.abs() {
            self.min
        } else {
            self.max
        }
    }

    #[inline]
    fn min(&self) -> f32 {
        self.min
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max
    }

    #[inline]
    fn count(&self) -> usize {
        self.count
    }

    #[inline]
    fn merge(&mut self, other: &Self) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other.clone();
        } else {
            self.min = self.min.min(other.min);
            self.max = self.max.max(other.max);
            self.sum_sq += other.sum_sq;
            self.count += other.count;
        }
    }
}
