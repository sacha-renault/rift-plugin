pub struct Linespace {
    start: f32,
    step: f32,
    num_points: usize,
    current_index: usize,
}

impl Linespace {
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
