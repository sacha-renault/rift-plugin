use crate::{
    params::param_queue_impl::{ControlPoint, ControlPoints},
    prelude::BlockInfo,
};

/// - **Classic** - phase is derived from absolute transport time, so all Classic
///   LFOs with the same frequency stay in sync.
/// - **Retrigger** - free-running, resets to 0 on [`retrigger()`](Lfo::retrigger),
///   wraps at 1.0. Keeps running when transport is stopped.
/// - **Envelope** - same as Retrigger but does *not* wrap: once past the last
///   control point, holds the last value indefinitely.
#[derive(Copy, Clone, PartialEq)]
pub enum LfoMode {
    Enveloppe,
    Retrigger,
    Classic,
}

/// LFO rate, either as raw Hz or as a beat division relative to tempo.
#[derive(Copy, Clone, PartialEq)]
pub enum LfoFrequency {
    Hz(f32),
    Beats(f32),
}

/// LFO that reads its waveform from a set of [`ControlPoints`] (x/y in [0, 1]).
///
/// # Modes
/// See [`LfoMode`]
///
/// # Position
/// `position` is always in [0, 1] and represents where we are in the control-point
/// curve. It is updated *after* the current value is read, so the first call after
/// a retrigger always reads position 0.
pub struct Lfo {
    position: f32,
    samplerate_recip: f32,
    mode: LfoMode,
    frequency: LfoFrequency,
}

impl Lfo {
    pub fn new(mode: LfoMode, frequency: LfoFrequency) -> Self {
        Self {
            samplerate_recip: 44100f32.recip(),
            position: 0.,
            mode,
            frequency,
        }
    }

    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.samplerate_recip = samplerate.recip();
    }

    pub fn set_mode(&mut self, mode: LfoMode) {
        self.mode = mode;
    }

    pub fn set_frequency(&mut self, frequency: LfoFrequency) {
        self.frequency = frequency;
    }

    pub fn retrigger(&mut self) {
        match self.mode {
            LfoMode::Enveloppe | LfoMode::Retrigger => self.position = 0.,
            LfoMode::Classic => {}
        }
    }

    /// Returns the current LFO value in [0, 1] and advances the position.
    ///
    /// `points` must be sorted by `x` and fully within [0, 1] (debug-asserted).
    /// Freezes when transport is stopped (`time` is `None`).
    #[inline]
    pub fn get_value(
        &mut self,
        points: &ControlPoints,
        infos: Option<BlockInfo>,
        sample_idx: usize,
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

        let position = self.get_position(infos, sample_idx);
        self.calculate_value(points, position)
    }

    pub fn update_position(&mut self, infos: Option<BlockInfo>, block_size: usize) {
        self.position = self.get_position(infos, block_size)
    }

    fn get_position(&self, time: Option<BlockInfo>, sample_idx: usize) -> f32 {
        let Some(time) = time else {
            return self.position;
        };

        let sample_idx = sample_idx as f32;

        // Is is_playing is false, then both second and beat position
        // would be fixed. it will be processed as a retrig
        if self.mode == LfoMode::Classic && time.is_playing() {
            self.get_position_classic(time, sample_idx)
        } else {
            self.get_position_retrig(time, sample_idx)
        }
    }

    fn get_position_classic(&self, infos: BlockInfo, sample_idx: f32) -> f32 {
        match self.frequency {
            LfoFrequency::Hz(hz) => {
                let seconds = hz.recip();
                let offset = sample_idx * hz * self.samplerate_recip;
                let pos = infos.seconds as f32 + offset;
                pos.rem_euclid(seconds) / seconds
            }
            LfoFrequency::Beats(beats) => {
                let offset_seconds = sample_idx * self.samplerate_recip;
                let beat_offset = offset_seconds * infos.tempo as f32 / 60.;
                let pos = infos.beats as f32 + beat_offset;
                pos.rem_euclid(beats) / beats
            }
        }
    }

    fn get_position_retrig(&self, infos: BlockInfo, sample_idx: f32) -> f32 {
        let mut position = self.position;

        match self.frequency {
            LfoFrequency::Hz(hz) => position += sample_idx * hz * self.samplerate_recip,
            LfoFrequency::Beats(beats) => {
                let hz = infos.tempo as f32 / (beats * 60.);
                position += sample_idx * hz * self.samplerate_recip
            }
        }

        if self.mode != LfoMode::Enveloppe {
            position = position.rem_euclid(1.);
        }

        position
    }

    fn calculate_value(&self, points: &ControlPoints, position: f32) -> f32 {
        let Some(right_idx) = points.iter().position(|p| p.x >= position) else {
            // Past all points - hold last value
            return points.last().map(|p| p.y).unwrap_or_default();
        };

        let right = &points[right_idx];

        let value = if right_idx == 0 {
            right.y
        } else {
            let left = &points[right_idx - 1];
            let fract = (position - left.x) / (right.x - left.x);
            let (_, y) = pow_interpolation(left, right, fract);
            y
        };

        value
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
