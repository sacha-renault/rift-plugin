use crate::consumers::Bucket;

#[derive(Clone)]
pub struct RmsBucket {
    sum_sq: f64,
    count: usize,
}

impl RmsBucket {
    #[inline]
    pub fn rms(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_sq / self.count as f64).sqrt() as f32
        }
    }
}

impl Bucket for RmsBucket {
    #[allow(dead_code)]
    fn new(x: f32) -> Self {
        Self {
            sum_sq: (x as f64) * (x as f64),
            count: 1,
        }
    }

    fn empty() -> Self {
        Self {
            sum_sq: 0.0,
            count: 0,
        }
    }

    #[inline]
    fn add_sample(&mut self, x: f32) {
        self.sum_sq += (x as f64) * (x as f64);
        self.count += 1;
    }

    #[inline]
    fn value(&self) -> f32 {
        self.rms()
    }

    #[inline]
    fn min(&self) -> f32 {
        -self.rms()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.rms()
    }

    #[inline]
    fn count(&self) -> usize {
        self.count
    }

    #[inline]
    fn merge(&mut self, other: &Self) {
        self.sum_sq += other.sum_sq;
        self.count += other.count;
    }
}
