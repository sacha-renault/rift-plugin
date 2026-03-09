pub struct Linespace {
    start: f32,
    step: f32,
    num_points: usize,
    current_index: usize,
}

impl Linespace {
    pub fn new(start: f32, end: f32, num_points: usize) -> Self {
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
