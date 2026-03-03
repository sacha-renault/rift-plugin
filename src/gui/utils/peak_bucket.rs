#[derive(Clone)]
pub struct PeakBucket {
    min: f32,
    max: f32,
    count: usize,
}

impl PeakBucket {
    pub fn new(x: f32) -> Self {
        Self {
            min: x,
            max: x,
            count: 1,
        }
    }

    pub fn empty() -> Self {
        PeakBucket {
            min: 0.,
            max: 0.,
            count: 0,
        }
    }

    pub fn add_sample(&mut self, x: f32) {
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
    pub fn peak(&self) -> f32 {
        if self.min.abs() > self.max.abs() {
            self.min
        } else {
            self.max
        }
    }

    #[inline]
    pub fn reset_count(&mut self) {
        self.count = 0;
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }
}
