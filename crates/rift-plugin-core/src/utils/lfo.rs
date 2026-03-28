use crate::params::param_queue_impl::ControlPoints;
use crate::prelude::BlockInfo;

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
pub enum LfoFreq {
    Hz(f32),
    Beats(f32),
}

/// LFO that reads its waveform from a set of [`ControlPoints`] (x/y in [0, 1]).
///
/// See [`LfoMode`] and [`LfoFreq`].
///
/// # Position
///
/// `position` is always in [0, 1] and represents where we are in the control-point
/// curve. The first call after a retrigger always reads position 0.
///
/// # Block setup
///
/// [`set_block_info`](Lfo::set_block_info) **must** be called at the start of every
/// processing block, before reading any value. It provides the transport state
/// (tempo, playback position, playing flag) needed to compute the LFO phase.
/// Control points can also be updated per-block via
/// [`set_control_points`](Lfo::set_control_points) if needed.
///
/// # Two ways to read the LFO
///
/// ## 1. Sample-accurate (per-sample loop)
///
/// Call [`get_next`](Lfo::get_next) once per sample. Each call advances the
/// internal position by one sample and returns the new value.
/// No call to [`update_lfo_position`](Lfo::update_lfo_position) is needed
/// since the position is already up to date.
///
/// ```text
/// set_block_info(infos)
///         │
///         ▼
///   get_next() --> sample 0
///   get_next() --> sample 1
///         ...
///   get_next() --> sample N-1
/// ```
///
/// ```ignore
/// self.lfo.set_block_info(ctx.block_info());
///
/// let main_buffer = buffers.main()?;
/// for sample_iter in main_buffer.iter_samples() {
///     let lfo_value = self.lfo.get_next();
///     for sample in sample_iter {
///         *sample *= lfo_value;
///     }
/// }
/// ```
///
/// ## 2. Block-rate (one value per block)
///
/// Call [`get_current`](Lfo::get_current) once to read the value at the current
/// position, then [`update_lfo_position`](Lfo::update_lfo_position) at the end
/// of the block to advance by `block_size` samples.
///
/// ```text
/// set_block_info(infos)
///         │
///         ▼
///   get_current() --> use for entire block
///         │
///         ▼
///   update_lfo_position(block_size)
/// ```
///
/// ```ignore
/// self.lfo.set_block_info(ctx.block_info());
/// let lfo_value = self.lfo.get_current();
///
/// let main_buffer = buffers.main()?;
/// for sample_iter in main_buffer.iter_samples() {
///     for sample in sample_iter {
///         *sample *= lfo_value;
///     }
/// }
///
/// self.lfo.update_lfo_position(main_buffer.samples());
/// ```
pub struct Lfo {
    position: f32,
    samplerate_recip: f32,
    mode: LfoMode,
    frequency: LfoFreq,
    points: ControlPoints,
    infos: Option<BlockInfo>,
}

impl Lfo {
    /// Creates a new LFO with the given mode and frequency.
    /// Defaults to 44100 Hz samplerate.
    pub fn new(mode: LfoMode, frequency: LfoFreq, samplerate: f32, points: ControlPoints) -> Self {
        Self {
            samplerate_recip: samplerate.recip(),
            position: 0.,
            mode,
            frequency,
            infos: None,
            points,
        }
    }

    /// Sets the samplerate. Call once at initialization or when samplerate changes.
    pub fn set_samplerate(&mut self, samplerate: f32) {
        self.samplerate_recip = samplerate.recip();
    }

    /// Sets the LFO mode. See [`LfoMode`].
    pub fn set_mode(&mut self, mode: LfoMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Sets the transport info for the current block. **Must** be called at the
    /// start of every processing block, before [`get_current`](Lfo::get_current)
    /// or [`get_next`](Lfo::get_next). Pass `None` to freeze the LFO position.
    pub fn set_block_info(&mut self, infos: Option<BlockInfo>) -> &mut Self {
        self.infos = infos;
        self
    }

    /// Sets the LFO frequency. See [`LfoFreq`].
    pub fn set_frequency(&mut self, frequency: LfoFreq) -> &mut Self {
        self.frequency = frequency;
        self
    }

    /// Replaces the control points that define the LFO waveform.
    /// Can be called per-block if points are automated.
    pub fn set_control_points(&mut self, control_points: &ControlPoints) {
        self.points.copy_from(control_points);
    }

    /// Returns the LFO value at the current position without advancing.
    /// Useful for block-rate usage where one value is used for the entire block.
    /// Call [`update_lfo_position`](Lfo::update_lfo_position) at the end of the block to advance.
    ///
    /// # Notes:
    /// You **MUST** call [`Self::set_block_info`] before calling this function !
    pub fn get_current(&self) -> f32 {
        debug_assert!(
            self.points
                .iter()
                .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)),
            "ControlPoints must be in [0, 1]"
        );
        debug_assert!(
            self.points.is_sorted_by(|l, r| l.x <= r.x),
            "ControlPoints must be sorted by x"
        );

        self.points.get_value(self.position)
    }

    /// Advances the position by one sample and returns the new LFO value.
    /// Use this in a per-sample loop for sample-accurate modulation.
    ///
    /// # Notes:
    /// You **MUST** call [`Self::set_block_info`] before calling this function !
    #[inline]
    pub fn get_next(&mut self) -> f32 {
        self.advance_by(1);
        self.get_current()
    }

    /// Advances the internal position by `block_size` samples and clears block info.
    /// Must be called once at the end of each block when using block-rate mode
    /// ([`get_current`](Lfo::get_current)). Not needed when using [`get_next`](Lfo::get_next)
    /// per sample.
    pub fn update_lfo_position(&mut self, block_size: usize) {
        self.advance_by(block_size);
    }

    /// Get the current position in the lfo. This will always be in [0., 1.] bounds
    pub fn get_position(&self) -> f32 {
        self.position.clamp(0., 1.)
    }

    /// Resets position to 0 for [`LfoMode::Retrigger`] and [`LfoMode::Enveloppe`].
    /// No-op for [`LfoMode::Classic`]. Typically called on note-on.
    pub fn retrigger(&mut self) {
        match self.mode {
            LfoMode::Enveloppe | LfoMode::Retrigger => self.position = 0.,
            LfoMode::Classic => {}
        }
    }

    /// Internal help to advance position by `samples`.
    fn advance_by(&mut self, samples: usize) {
        self.position = get_position(
            self.position,
            self.samplerate_recip,
            self.frequency,
            self.mode,
            self.infos.as_ref(),
            samples as f32,
        );
    }
}

fn get_position(
    position: f32,
    samplerate_recip: f32,
    frequency: LfoFreq,
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
    frequency: LfoFreq,
    infos: &BlockInfo,
    sample_idx: f32,
) -> f32 {
    match frequency {
        LfoFreq::Hz(hz) => {
            let period = hz.recip();
            let offset = sample_idx * samplerate_recip;
            let pos = infos.seconds as f32 + offset;
            pos.rem_euclid(period) / period
        }
        LfoFreq::Beats(beats) => {
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
    frequency: LfoFreq,
    mode: LfoMode,
    infos: &BlockInfo,
    sample_idx: f32,
) -> f32 {
    match frequency {
        LfoFreq::Hz(hz) => position += sample_idx * hz * samplerate_recip,
        LfoFreq::Beats(beats) => {
            let hz = infos.tempo as f32 / (beats * 60.);
            position += sample_idx * hz * samplerate_recip
        }
    }

    if mode != LfoMode::Enveloppe {
        position = position.rem_euclid(1.);
    }

    position
}

#[cfg(test)]
mod tests {
    use crate::params::param_queue_impl::ControlPoint;
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
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.25, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            // get_next advances by 1 sample which triggers position computation
            assert_approx_eq!(lfo.get_next(), 0.25, 1e-3);
        }

        #[test]
        fn classic_hz_wraps_at_period() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(1.25, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.25, 1e-3);
        }

        #[test]
        fn classic_hz_sample_accurate_within_block() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));

            let v0 = lfo.get_current();
            let v1 = lfo.get_next();
            let diff = v1 - v0;
            assert_approx_eq!(diff, 1.0 / SAMPLERATE);
        }

        #[test]
        fn classic_beats_position_from_beat_time() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Beats(4.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 1.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.25, 1e-3);
        }

        #[test]
        fn classic_beats_wraps() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Beats(4.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 5.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.25, 1e-3);
        }

        #[test]
        fn retrigger_hz_advances_and_wraps() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position(SAMPLERATE as usize);

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.0, 1e-2);
        }

        #[test]
        fn retrigger_resets_on_retrigger() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 0.5) as usize);

            lfo.retrigger();
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.0, 1e-4);
        }

        #[test]
        fn envelope_does_not_wrap() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Enveloppe, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 1.5) as usize);

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 1.0, 1e-4);
        }

        #[test]
        fn envelope_retrigger_resets() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Enveloppe, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position((SAMPLERATE * 1.5) as usize);

            lfo.retrigger();
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.0, 1e-4);
        }

        #[test]
        fn classic_retrigger_is_noop() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.5, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            let before = lfo.get_next();

            lfo.retrigger();

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), before, 1e-3);
        }

        #[test]
        fn interpolation_midpoint() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.5, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.5, 1e-3);
        }

        #[test]
        fn tension_affects_interpolation() {
            let linear_points = make_points(&[(0.0, 0.0), (1.0, 1.0)]);
            let curved_points = make_points_with_tension(&[(0.0, 0.0, 2.0), (1.0, 1.0, 0.0)]);

            let mut lfo_lin = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, linear_points);
            lfo_lin.set_samplerate(SAMPLERATE);

            let mut lfo_cur = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, curved_points);
            lfo_cur.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.5, 0.0, 120.0, true);

            lfo_lin.set_block_info(Some(infos.clone()));
            let val_lin = lfo_lin.get_next();

            lfo_cur.set_block_info(Some(infos));
            let val_cur = lfo_cur.get_next();

            assert!(
                val_cur < val_lin,
                "curved ({val_cur}) should be below linear ({val_lin})"
            );
        }

        #[test]
        fn get_next_advances_monotonically() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(2.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));

            let mut prev = lfo.get_current();
            for _ in 0..100 {
                let val = lfo.get_next();
                assert!(val >= prev, "expected monotonic increase on ramp");
                prev = val;
            }
        }

        #[test]
        fn update_position_advances_across_blocks() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position(4410);

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.1, 1e-3);
        }
    }

    mod edge_tests {
        use crate::assert_approx_eq;

        use super::*;

        #[test]
        fn empty_points_returns_zero() {
            let points = make_points(&[]);
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.5, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.0);
        }

        #[test]
        fn single_point_returns_its_value() {
            let points = make_points(&[(0.5, 0.7)]);
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.7, 1e-4);
        }

        #[test]
        fn position_exactly_on_point() {
            let points = make_points(&[(0.0, 0.3), (0.5, 0.8), (1.0, 0.1)]);
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.5, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.8, 1e-3);
        }

        #[test]
        fn none_infos_freezes_position() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            lfo.update_lfo_position((SAMPLERATE * 0.3) as usize);

            lfo.set_block_info(None);
            let v0 = lfo.get_current();
            // Advancing with None infos should not change position
            let v1 = lfo.get_next();
            assert_approx_eq!(v0, v1);
        }

        #[test]
        fn none_infos_update_does_not_advance() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            lfo.update_lfo_position((SAMPLERATE * 0.3) as usize);

            lfo.set_block_info(None);
            let before = lfo.get_current();

            lfo.update_lfo_position(4096);

            lfo.set_block_info(None);
            assert_approx_eq!(lfo.get_current(), before);
        }

        #[test]
        fn classic_not_playing_falls_back_to_retrig() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(2.0, 0.0, 120.0, false);
            lfo.set_block_info(Some(infos));

            let v0 = lfo.get_current();
            let v100 = {
                // Advance 100 samples
                for _ in 0..100 {
                    lfo.get_next();
                }
                lfo.get_current()
            };

            assert!(
                (v100 - v0).abs() > 1e-6,
                "not-playing Classic should still advance within block via retrig"
            );
        }

        #[test]
        fn high_frequency_wraps_correctly() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Classic, LfoFreq::Hz(100.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0025, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_next(), 0.25, 1e-2);
        }

        #[test]
        fn block_rate_usage() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Hz(1.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            assert_approx_eq!(lfo.get_current(), 0.0, 1e-4);

            lfo.update_lfo_position(4410);

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.1, 1e-3);
        }

        #[test]
        fn retrigger_beats_advances_correctly() {
            let points = ramp_points();
            let mut lfo = Lfo::new(LfoMode::Retrigger, LfoFreq::Beats(4.0), 44100f32, points);
            lfo.set_samplerate(SAMPLERATE);

            let infos = make_infos(0.0, 0.0, 120.0, true);
            lfo.set_block_info(Some(infos.clone()));
            lfo.update_lfo_position(SAMPLERATE as usize);

            lfo.set_block_info(Some(infos));
            assert_approx_eq!(lfo.get_current(), 0.5, 1e-2);
        }
    }
}
