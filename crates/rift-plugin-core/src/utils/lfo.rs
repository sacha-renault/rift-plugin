use crate::{params::param_queue_impl::ControlPoints, utils::interpo::lerp};

pub struct Lfo {
    position: f32,
    samplerate: f32,
    one_shot: bool,
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            samplerate: 44100.,
            position: 0.,
            one_shot: false,
        }
    }
}

impl Lfo {
    pub fn new(one_shot: bool) -> Self {
        Self {
            samplerate: 44100.,
            position: 0.,
            one_shot,
        }
    }

    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.samplerate = samplerate;
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn reset(&mut self) {
        self.position = 0.;
    }

    pub fn get_position_normalized(&self) -> f32 {
        self.position
    }

    pub fn get_value(&mut self, points: &ControlPoints, lfo_seconds: f32) -> f32 {
        if self.position > 1. && !self.one_shot {
            self.position = self.position.rem_euclid(1.);
        }

        let Some(right_idx) = points.iter().position(|p| p.x >= self.position) else {
            // Past all points — hold last value
            self.position += 1. / (self.samplerate * lfo_seconds);
            return points.last().map(|p| p.y).unwrap_or_default();
        };

        let right = &points[right_idx];

        let value = if right_idx == 0 {
            right.y
        } else {
            let left = &points[right_idx - 1];
            let fract = (self.position - left.x) / (right.x - left.x);
            lerp(left.y, right.y, fract)
        };

        // Advance position normalized
        self.position += 1. / (self.samplerate * lfo_seconds);
        value
    }
}
