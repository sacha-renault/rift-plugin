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
///
/// - `Hz(f32)` - absolute frequency in hertz. Independent of tempo.
/// - `Beats(f32)` - period in beats (e.g. `4.0` = one full cycle every 4 beats).
///   Requires a valid `tempo` in [`BlockInfo`].
#[derive(Copy, Clone, PartialEq)]
pub enum LfoFrequency {
    Hz(f32),
    Beats(f32),
}

/// LFO that reads its waveform from a set of [`ControlPoints`] (x/y in [0, 1]).
///
/// See [`LfoMode`] and [`LfoFrequency`]
///
/// # Position
/// `position` is always in [0, 1] and represents where we are in the control-point
/// curve. [`update_position`](Lfo::update_position) must be called once at the end
/// of each block. The first call after a retrigger always reads position 0.
///
/// # Usage
/// Since `get_value` computes position from `sample_idx` without mutating state,
/// it returns the same value for a given index regardless of iteration order.
/// This means both per-sample and per-channel iteration are correct:
///
/// # Per-block lifecycle
///
/// ```text
/// prepare_block(points, infos, block_size)
///         │
///         ▼
///   get_value(0) ──► sample 0
///   get_value(1) ──► sample 1
///   get_value(2) ──► sample 2
///         ...
///   get_value(N) ──► sample N
///         │
///         ▼
///   update_position(block_size)
/// ```
pub struct Lfo {
    position: f32,
    samplerate_recip: f32,
    mode: LfoMode,
    frequency: LfoFrequency,
}

impl Lfo {
    /// Creates a new LFO with the given mode and frequency.
    /// Defaults to 44100 Hz samplerate.
    pub fn new(mode: LfoMode, frequency: LfoFrequency) -> Self {
        Self {
            samplerate_recip: 44100f32.recip(),
            position: 0.,
            mode,
            frequency,
        }
    }

    /// Sets the samplerate. Call once at initialization or when samplerate changes.
    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.samplerate_recip = samplerate.recip();
    }

    /// Sets the LFO mode. See [`LfoMode`].
    pub fn set_mode(&mut self, mode: LfoMode) {
        self.mode = mode;
    }

    /// Sets the LFO frequency. See [`LfoFrequency`].
    pub fn set_frequency(&mut self, frequency: LfoFrequency) {
        self.frequency = frequency;
    }

    /// Prepares the LFO for a new processing block.
    pub fn prepare_block<'a>(
        &'a mut self,
        lfo_points: &'a ControlPoints,
        infos: Option<BlockInfo>,
        block_length: usize,
    ) -> LfoBlock<'a> {
        LfoBlock {
            lfo: self,
            lfo_points,
            block_length,
            infos,
        }
    }

    /// Resets position to 0 for [`LfoMode::Retrigger`] and [`LfoMode::Enveloppe`].
    /// No-op for [`LfoMode::Classic`]. Typically called on note-on.
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
    fn get_value(
        &self,
        points: &ControlPoints,
        infos: Option<&BlockInfo>,
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

    /// Advances the internal position by `block_size` samples.
    /// Must be called once at the end of each block, after all [`get_value`](Lfo::get_value) calls.
    pub fn update_position(&mut self, infos: Option<&BlockInfo>, block_size: usize) {
        self.position = self.get_position(infos, block_size);
    }

    fn get_position(&self, infos: Option<&BlockInfo>, sample_idx: usize) -> f32 {
        let Some(infos) = infos else {
            return self.position;
        };

        let sample_idx = sample_idx as f32;

        // Is is_playing is false, then both second and beat position
        // would be fixed. it will be processed as a retrig
        if self.mode == LfoMode::Classic && infos.is_playing() {
            self.get_position_classic(infos, sample_idx)
        } else {
            self.get_position_retrig(infos, sample_idx)
        }
    }

    fn get_position_classic(&self, infos: &BlockInfo, sample_idx: f32) -> f32 {
        match self.frequency {
            LfoFrequency::Hz(hz) => {
                let period = hz.recip();
                let offset = sample_idx * self.samplerate_recip;
                let pos = infos.seconds as f32 + offset;
                pos.rem_euclid(period) / period
            }
            LfoFrequency::Beats(beats) => {
                let offset_seconds = sample_idx * self.samplerate_recip;
                let beat_offset = offset_seconds * infos.tempo as f32 / 60.;
                let pos = infos.beats as f32 + beat_offset;
                pos.rem_euclid(beats) / beats
            }
        }
    }

    fn get_position_retrig(&self, infos: &BlockInfo, sample_idx: f32) -> f32 {
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

pub struct LfoBlock<'a> {
    lfo: &'a mut Lfo,
    lfo_points: &'a ControlPoints,

    /// We could get this one in lfo but this saves one indirection
    block_length: usize,
    infos: Option<BlockInfo>,
}

impl<'a> LfoBlock<'a> {
    pub fn get_value(&self, sample_idx: usize) -> f32 {
        self.lfo
            .get_value(self.lfo_points, self.infos.as_ref(), sample_idx)
    }

    pub fn retrigger(&mut self) {
        self.lfo.retrigger();
    }

    pub fn update_position(&mut self) {
        self.lfo
            .update_position(self.infos.as_ref(), self.block_length);
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
