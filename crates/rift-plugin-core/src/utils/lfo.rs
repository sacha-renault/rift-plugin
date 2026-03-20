use crate::{
    params::param_queue_impl::{ControlPoint, ControlPoints},
    prelude::BlockInfo,
};

#[derive(Copy, Clone, PartialEq)]
pub enum LfoMode {
    Enveloppe,
    Retrigger,
    Classic,
}

/// LFO rate, either as raw Hz or as a beat division relative to tempo.
pub enum LfoFrequency {
    Hz(f32),
    Beats(f32),
}

/// LFO that reads its waveform from a set of [`ControlPoints`] (x/y in [0, 1]).
///
/// # Modes
/// - **Classic** - phase is derived from absolute transport time, so all Classic
///   LFOs with the same frequency stay in sync. Freezes when transport is stopped
///   (`time` is `None`).
/// - **Retrigger** - free-running, resets to 0 on [`retrigger()`](Lfo::retrigger),
///   wraps at 1.0. Keeps running when transport is stopped.
/// - **Envelope** - same as Retrigger but does *not* wrap: once past the last
///   control point, holds the last value indefinitely.
///
/// # Position
/// `position` is always in [0, 1] and represents where we are in the control-point
/// curve. It is updated *after* the current value is read, so the first call after
/// a retrigger always reads position 0.
pub struct Lfo {
    position: f32,
    samplerate: f32,
    mode: LfoMode,
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            samplerate: 44100.,
            position: 0.,
            mode: LfoMode::Classic,
        }
    }
}

impl Lfo {
    pub fn new(mode: LfoMode) -> Self {
        Self {
            samplerate: 44100.,
            position: 0.,
            mode,
        }
    }

    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.samplerate = samplerate;
    }

    pub fn set_mode(&mut self, mode: LfoMode) {
        self.mode = mode;
    }

    pub fn retrigger(&mut self) {
        match self.mode {
            LfoMode::Enveloppe | LfoMode::Retrigger => self.position = 0.,
            LfoMode::Classic => {}
        }
    }

    pub fn get_position_normalized(&self) -> f32 {
        self.position
    }

    /// Returns the current LFO value in [0, 1] and advances the position.
    ///
    /// `points` must be sorted by `x` and fully within [0, 1] (debug-asserted).
    /// Pass `None` for `time` when the transport is stopped.
    #[inline]
    pub fn get_value(
        &mut self,
        points: &ControlPoints,
        lfo_freq: LfoFrequency,
        time: Option<BlockInfo>,
    ) -> f32 {
        debug_assert!(
            points
                .iter()
                .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)),
            "ControlPoints must be in [0, 1]"
        );
        debug_assert!(
            points.is_sorted_by(|l, r| l.x <= r.x),
            "ControlPoints must be sorted by x"
        );

        let value = self.calculate_value(points);
        self.update_position(lfo_freq, time);
        value
    }

    fn calculate_value(&self, points: &ControlPoints) -> f32 {
        let Some(right_idx) = points.iter().position(|p| p.x >= self.position) else {
            // Past all points - hold last value
            return points.last().map(|p| p.y).unwrap_or_default();
        };

        let right = &points[right_idx];

        let value = if right_idx == 0 {
            right.y
        } else {
            let left = &points[right_idx - 1];
            let fract = (self.position - left.x) / (right.x - left.x);
            let (_, y) = pow_interpolation(left, right, fract);
            y
        };

        value
    }

    fn update_position(&mut self, lfo_freq: LfoFrequency, time: Option<BlockInfo>) {
        if self.mode == LfoMode::Classic {
            let Some(time) = time else {
                return;
            };

            match lfo_freq {
                LfoFrequency::Hz(hz) => {
                    let seconds = hz.recip();
                    self.position = (time.seconds as f32).rem_euclid(seconds) / seconds;
                    return;
                }
                LfoFrequency::Beats(beats) => {
                    self.position = (time.beats as f32).rem_euclid(beats) / beats;
                    return;
                }
            }
        } else {
            let Some(time) = time else {
                return;
            };

            match lfo_freq {
                LfoFrequency::Hz(hz) => self.position += hz / self.samplerate,
                LfoFrequency::Beats(beats) => {
                    let seconds = beats * 60. / time.tempo as f32;
                    self.position += 1. / (seconds * self.samplerate)
                }
            }

            if self.mode == LfoMode::Retrigger {
                self.position = self.position.rem_euclid(1.);
            }
        }
    }
}

fn pow_interpolation(p1: &ControlPoint, p2: &ControlPoint, t: f32) -> (f32, f32) {
    let x = p1.x + (p2.x - p1.x) * t;
    let y = p1.y + (p2.y - p1.y) * shape(t, p1.tension);
    (x, y)
}

fn shape(t: f32, curve_amount: f32) -> f32 {
    if curve_amount.abs() < 1e-6 {
        return t;
    }

    let exp = curve_amount.exp2();
    if curve_amount > 0.0 {
        t.powf(exp)
    } else {
        1.0 - (1.0 - t).powf(exp.recip())
    }
}
