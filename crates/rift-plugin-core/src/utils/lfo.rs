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
/// curve. [`update_lfo_position`](Lfo::update_lfo_position) must be called once at the end
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
/// get_lfo_block(points, infos)
///         │
///         ▼
///   get_value(0) ──► sample 0
///   get_value(1) ──► sample 1
///   get_value(2) ──► sample 2
///         ...
///   get_value(N) ──► sample N
///         │
///         ▼
///   update_lfo_position(block_size)
/// ```
///
/// # Example:
///
/// ```ignore
/// let lfo_points = self.params.lfo.value();
/// let lfo1_block = self.lfo1.get_lfo_block(lfo_points, ctx.block_info());
/// let lfo2_block = self.lfo2.get_lfo_block(lfo_points, ctx.block_info());

/// let main_buffer = buffers.main()?;
/// for (idx, sample_iter) in main_buffer.iter_samples().enumerate() {
///     let lfo_value = lfo1_block.get_value(idx);
///     let lfo_value2 = lfo2_block.get_value(idx);
///     for sample in sample_iter {
///         *sample *= lfo_value;
///         // Do something with lfo_value2 :)
///     }
/// }
/// // We just need to update the param at the very end
/// self.lfo1.update_lfo_position(main_buffer.samples());
/// self.lfo2.update_lfo_position(main_buffer.samples());
/// ```
pub struct Lfo {
    position: f32,
    samplerate_recip: f32,
    mode: LfoMode,
    frequency: LfoFrequency,
    infos: Option<BlockInfo>,
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
            infos: None,
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

    /// Advances the internal position by `block_size` samples.
    /// Must be called once at the end of each block, after all [`get_value`](Lfo::get_value) calls.
    pub fn update_lfo_position(&mut self, block_size: usize) {
        self.position = get_position(
            self.position,
            self.samplerate_recip,
            self.frequency,
            self.mode,
            self.infos.as_ref(),
            block_size as f32,
        );
    }

    /// Prepares the LFO for a new processing block.
    /// Can be called as many time as wanted in the process function
    pub fn get_lfo_block<'a>(
        &mut self,
        lfo_points: &'a ControlPoints,
        infos: Option<BlockInfo>,
    ) -> LfoBlock<'a> {
        self.infos = infos.clone();

        LfoBlock {
            frequency: self.frequency,
            mode: self.mode,
            position: self.position,
            samplerate_recip: self.samplerate_recip,
            lfo_points,
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
}

pub struct LfoBlock<'a> {
    lfo_points: &'a ControlPoints,
    infos: Option<BlockInfo>,

    /// We could get this one in lfo but this saves one indirection
    mode: LfoMode,
    frequency: LfoFrequency,
    position: f32,
    samplerate_recip: f32,
}

impl<'a> LfoBlock<'a> {
    /// Returns the current LFO value in [0, 1] and advances the position.
    ///
    /// `points` must be sorted by `x` and fully within [0, 1] (debug-asserted).
    /// Freezes when transport is stopped (`time` is `None`).
    #[inline]
    pub fn get_value(&self, sample_idx: usize) -> f32 {
        debug_assert!(
            self.lfo_points
                .iter()
                .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)),
            "ControlPoints must be in [0, 1]"
        );
        debug_assert!(
            self.lfo_points.is_sorted_by(|l, r| l.x <= r.x),
            "ControlPoints must be sorted by x"
        );

        let position = get_position(
            self.position,
            self.samplerate_recip,
            self.frequency,
            self.mode,
            self.infos.as_ref(),
            sample_idx as f32,
        );
        calculate_value(&self.lfo_points, position)
    }
}

fn get_position(
    position: f32,
    samplerate_recip: f32,
    frequency: LfoFrequency,
    mode: LfoMode,
    infos: Option<&BlockInfo>,
    sample_idx: f32,
) -> f32 {
    let Some(infos) = infos else {
        return position;
    };

    // Is is_playing is false, then both second and beat position
    // would be fixed. it will be processed as a retrig
    if mode == LfoMode::Classic && infos.is_playing() {
        get_position_classic(samplerate_recip, frequency, infos, sample_idx)
    } else {
        get_position_retrig(
            position,
            samplerate_recip,
            frequency,
            mode,
            infos,
            sample_idx,
        )
    }
}

fn get_position_classic(
    samplerate_recip: f32,
    frequency: LfoFrequency,
    infos: &BlockInfo,
    sample_idx: f32,
) -> f32 {
    match frequency {
        LfoFrequency::Hz(hz) => {
            let period = hz.recip();
            let offset = sample_idx * samplerate_recip;
            let pos = infos.seconds as f32 + offset;
            pos.rem_euclid(period) / period
        }
        LfoFrequency::Beats(beats) => {
            let offset_seconds = sample_idx * samplerate_recip;
            let beat_offset = offset_seconds * infos.tempo as f32 / 60.;
            let pos = infos.beats as f32 + beat_offset;
            pos.rem_euclid(beats) / beats
        }
    }
}

fn get_position_retrig(
    mut position: f32,
    samplerate_recip: f32,
    frequency: LfoFrequency,
    mode: LfoMode,
    infos: &BlockInfo,
    sample_idx: f32,
) -> f32 {
    match frequency {
        LfoFrequency::Hz(hz) => position += sample_idx * hz * samplerate_recip,
        LfoFrequency::Beats(beats) => {
            let hz = infos.tempo as f32 / (beats * 60.);
            position += sample_idx * hz * samplerate_recip
        }
    }

    if mode != LfoMode::Enveloppe {
        position = position.rem_euclid(1.);
    }

    position
}

fn calculate_value(points: &ControlPoints, position: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use clack_plugin::events::event_types::TransportFlags;

    use super::*;

    const SAMPLERATE: f32 = 44100.0;

    fn make_infos(seconds: f64, beats: f64, tempo: f64, playing: bool) -> BlockInfo {
        let mut info = BlockInfo::new(seconds, beats, SAMPLERATE as f64, tempo);
        if playing {
            info.flags = TransportFlags::IS_PLAYING;
        }
        info
    }

    fn make_points(pts: &[(f32, f32)]) -> ControlPoints {
        let points: Vec<ControlPoint> = pts
            .iter()
            .map(|&(x, y)| ControlPoint { x, y, tension: 0.0 })
            .collect();
        ControlPoints::with_value(points, 64)
    }

    fn make_points_with_tension(pts: &[(f32, f32, f32)]) -> ControlPoints {
        let points: Vec<ControlPoint> = pts
            .iter()
            .map(|&(x, y, tension)| ControlPoint { x, y, tension })
            .collect();
        ControlPoints::with_value(points, 64)
    }

    fn ramp_points() -> ControlPoints {
        make_points(&[(0.0, 0.0), (1.0, 1.0)])
    }

    mod normal_tests {
        use crate::assert_approx_eq;

        use super::*;

        #[test]
        fn classic_hz_position_from_absolute_time() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.25, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.25, 1e-4);
        }

        #[test]
        fn classic_hz_wraps_at_period() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(1.25, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.25, 1e-4);
        }

        #[test]
        fn classic_hz_sample_accurate_within_block() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));

            let diff = block.get_value(1) - block.get_value(0);
            assert_approx_eq!(diff, 1.0 / SAMPLERATE);
        }

        #[test]
        fn classic_beats_position_from_beat_time() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Beats(4.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 1.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.25, 1e-4);
        }

        #[test]
        fn classic_beats_wraps() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Beats(4.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 5.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.25, 1e-4);
        }

        #[test]
        fn retrigger_hz_advances_and_wraps() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position(SAMPLERATE as usize);

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.0, 1e-2);
        }

        #[test]
        fn retrigger_resets_on_retrigger() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 0.5) as usize);

            lfo.retrigger();
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.0, 1e-4);
        }

        #[test]
        fn envelope_does_not_wrap() {
            let mut lfo = Lfo::new(LfoMode::Enveloppe, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 1.5) as usize);

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 1.0, 1e-4);
        }

        #[test]
        fn envelope_retrigger_resets() {
            let mut lfo = Lfo::new(LfoMode::Enveloppe, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 1.5) as usize);

            lfo.retrigger();
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.0, 1e-4);
        }

        #[test]
        fn classic_retrigger_is_noop() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.5, 0.0, 120.0, true);

            let block = lfo.get_lfo_block(&points, Some(infos.clone()));
            let before = block.get_value(0);

            lfo.retrigger();

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), before);
        }

        #[test]
        fn interpolation_midpoint() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.5, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.5, 1e-4);
        }

        #[test]
        fn tension_affects_interpolation() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let linear_points = make_points(&[(0.0, 0.0), (1.0, 1.0)]);
            let curved_points = make_points_with_tension(&[(0.0, 0.0, 2.0), (1.0, 1.0, 0.0)]);

            let infos = make_infos(0.5, 0.0, 120.0, true);

            let block_lin = lfo.get_lfo_block(&linear_points, Some(infos.clone()));
            let val_lin = block_lin.get_value(0);

            let block_cur = lfo.get_lfo_block(&curved_points, Some(infos));
            let val_cur = block_cur.get_value(0);

            assert!(
                val_cur < val_lin,
                "curved ({val_cur}) should be below linear ({val_lin})"
            );
        }

        #[test]
        fn multiple_lfo_blocks_same_result() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(2.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position(1000);

            let block1 = lfo.get_lfo_block(&points, Some(infos.clone()));
            let block2 = lfo.get_lfo_block(&points, Some(infos));

            for idx in [0, 10, 50, 100] {
                assert_approx_eq!(block1.get_value(idx), block2.get_value(idx));
            }
        }

        #[test]
        fn iteration_order_does_not_matter() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(2.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));

            let forward: Vec<f32> = (0..128).map(|i| block.get_value(i)).collect();
            let backward: Vec<f32> = (0..128).rev().map(|i| block.get_value(i)).collect();
            let backward_rev: Vec<f32> = backward.into_iter().rev().collect();

            for i in 0..128 {
                assert_approx_eq!(forward[i], backward_rev[i]);
            }
        }

        #[test]
        fn update_position_advances_across_blocks() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position(4410);

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.1, 1e-3);
        }
    }

    mod edge_tests {
        use crate::assert_approx_eq;

        use super::*;

        #[test]
        fn empty_points_returns_zero() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = make_points(&[]);
            let infos = make_infos(0.5, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.0);
        }

        #[test]
        fn single_point_returns_its_value() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = make_points(&[(0.5, 0.7)]);
            let infos = make_infos(0.0, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.7, 1e-4);
        }

        #[test]
        fn position_exactly_on_point() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = make_points(&[(0.0, 0.3), (0.5, 0.8), (1.0, 0.1)]);
            let infos = make_infos(0.5, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.8, 1e-4);
        }

        #[test]
        fn past_all_points_holds_last() {
            let points = make_points(&[(0.0, 0.2), (0.5, 0.9)]);
            assert_approx_eq!(calculate_value(&points, 0.8), 0.9, 1e-4);
        }

        #[test]
        fn none_infos_freezes_position() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos));
            lfo.update_lfo_position((SAMPLERATE * 0.3) as usize);

            let block = lfo.get_lfo_block(&points, None);
            assert_approx_eq!(block.get_value(0), block.get_value(100));
        }

        #[test]
        fn none_infos_update_does_not_advance() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos));
            lfo.update_lfo_position((SAMPLERATE * 0.3) as usize);

            let block = lfo.get_lfo_block(&points, None);
            let before = block.get_value(0);

            lfo.update_lfo_position(4096);

            let block = lfo.get_lfo_block(&points, None);
            assert_approx_eq!(block.get_value(0), before);
        }

        #[test]
        fn classic_not_playing_falls_back_to_retrig() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(2.0, 0.0, 120.0, false);

            let block = lfo.get_lfo_block(&points, Some(infos));
            let v0 = block.get_value(0);
            let v100 = block.get_value(100);

            assert!(
                (v100 - v0).abs() > 1e-6,
                "not-playing Classic should still advance within block via retrig"
            );
        }

        #[test]
        fn high_frequency_wraps_correctly() {
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFrequency::Hz(100.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0025, 0.0, 120.0, true);
            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.25, 1e-3);
        }

        #[test]
        fn block_rate_usage() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Hz(1.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let block = lfo.get_lfo_block(&points, Some(infos.clone()));
            assert_approx_eq!(block.get_value(0), 0.0, 1e-4);

            lfo.update_lfo_position(4410);

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.1, 1e-3);
        }

        #[test]
        fn position_at_boundaries() {
            let points = make_points(&[(0.0, 0.2), (0.5, 0.7), (1.0, 0.4)]);
            assert_approx_eq!(calculate_value(&points, 0.0), 0.2, 1e-4);
            assert_approx_eq!(calculate_value(&points, 1.0), 0.4, 1e-4);
        }

        #[test]
        fn retrigger_beats_advances_correctly() {
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFrequency::Beats(4.0));
            lfo.set_samplerate(SAMPLERATE);

            let points = ramp_points();
            let infos = make_infos(0.0, 0.0, 120.0, true);

            let _ = lfo.get_lfo_block(&points, Some(infos.clone()));
            lfo.update_lfo_position(SAMPLERATE as usize);

            let block = lfo.get_lfo_block(&points, Some(infos));
            assert_approx_eq!(block.get_value(0), 0.5, 1e-2);
        }
    }
}
