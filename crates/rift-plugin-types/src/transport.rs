pub use clack_plugin::events::event_types::TransportFlags;
use clack_plugin::{
    events::event_types::TransportEvent,
    utils::{BeatTime, SecondsTime},
};

const DEFAULT_TEMPO: f64 = 130.;

#[derive(Clone, Copy)]
pub struct ChannelsInfo {
    pub current: usize,
    pub total_channels: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct BlockIndex(pub i64);

impl BlockIndex {
    pub fn increment(&mut self) -> Self {
        if self.0 == i64::MAX {
            self.0 = 0;
            *self
        } else {
            self.0 += 1;
            *self
        }
    }
}

impl ChannelsInfo {
    pub fn mono() -> Self {
        ChannelsInfo {
            current: 0,
            total_channels: 1,
        }
    }
    pub fn is_last_channel(&self) -> bool {
        // idx starts at 0
        self.current + 1 == self.total_channels
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LoopRegion {
    pub start_beats: BeatTime,
    pub end_beats: BeatTime,
    pub start_seconds: SecondsTime,
    pub end_seconds: SecondsTime,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TimeSignature {
    pub numerator: u16,
    pub denominator: u16,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BarInfo {
    pub start: BeatTime,
    pub number: i32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TransportInfo {
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_preroll: bool,
    pub sample_rate: f64,

    pub song_pos_beats: BeatTime,
    pub song_pos_seconds: SecondsTime,

    pub tempo: f64,
    pub tempo_inc: f64,

    pub loop_region: Option<LoopRegion>,
    pub time_signature: Option<TimeSignature>,
    pub bar: Option<BarInfo>,
    last_sample_count: usize,
}

impl TransportInfo {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            is_preroll: false,
            sample_rate,
            song_pos_beats: 0f64.into(),
            song_pos_seconds: 0f64.into(),
            tempo: DEFAULT_TEMPO,
            tempo_inc: 0.,
            loop_region: None,
            time_signature: None,
            bar: None,
            last_sample_count: 0,
        }
    }

    pub fn is_loop_active(&self) -> bool {
        self.loop_region.is_some()
    }

    /// Must be called exactly once per processed block, with that block's frame count.
    /// If the host omits a timeline, the position is extrapolated from the previous
    /// block's tempo and play state. It is overwritten as soon as the host provides it.
    pub fn update(&mut self, event: Option<&TransportEvent>, sample_count: usize) {
        // Advance over the previous block using the previous block's state
        // Will be overwritten if outdated
        if self.is_playing {
            self.advance_both();
        }

        match event {
            Some(ev) => self.update_with_event(ev),
            None => {
                self.is_preroll = false;
                self.tempo_inc = 0.0;
                self.loop_region = None;
                self.time_signature = None;
                self.bar = None;
            }
        }

        self.last_sample_count = sample_count;
    }

    fn update_with_event(&mut self, event: &TransportEvent) {
        self.is_playing = event.flags.contains(TransportFlags::IS_PLAYING);
        self.is_recording = event.flags.contains(TransportFlags::IS_RECORDING);
        self.is_preroll = event.flags.contains(TransportFlags::IS_WITHIN_PRE_ROLL);

        if event.flags.contains(TransportFlags::HAS_TEMPO) {
            self.tempo = event.tempo;
            self.tempo_inc = event.tempo_inc;
        } else {
            // I think we just keep old tempo
            // reset to default seems odd ...
            self.tempo_inc = 0f64;
        }

        self.loop_region = event
            .flags
            .contains(TransportFlags::IS_LOOP_ACTIVE)
            .then(|| LoopRegion {
                start_beats: event.loop_start_beats,
                end_beats: event.loop_end_beats,
                start_seconds: event.loop_start_seconds,
                end_seconds: event.loop_end_seconds,
            });

        self.time_signature = event
            .flags
            .contains(TransportFlags::HAS_TIME_SIGNATURE)
            .then(|| TimeSignature {
                numerator: event.time_signature_numerator,
                denominator: event.time_signature_denominator,
            });

        if event.flags.contains(TransportFlags::HAS_BEATS_TIMELINE) {
            self.song_pos_beats = event.song_pos_beats;
            self.bar = Some(BarInfo {
                start: event.bar_start,
                number: event.bar_number,
            });
        } else {
            self.bar = None;
        }

        if event.flags.contains(TransportFlags::HAS_SECONDS_TIMELINE) {
            self.song_pos_seconds = event.song_pos_seconds;
        }
    }

    fn advance_both(&mut self) {
        self.advance_beats();
        self.advance_seconds();
    }

    fn advance_seconds(&mut self) {
        let n = self.last_sample_count as f64;
        let increment = (n / self.sample_rate).into();
        self.song_pos_seconds = self.song_pos_seconds + increment;
    }

    fn advance_beats(&mut self) {
        // Tempo increment is constant so avg tempo is (start_tempo + end_tempo) / 2
        let n = self.last_sample_count as f64;
        let avg_tempo = self.tempo + self.tempo_inc * n / 2.0;
        let increment = (n / self.sample_rate * avg_tempo / 60f64).into();
        self.song_pos_beats = self.song_pos_beats + increment;
    }
}

#[derive(Clone)]
pub struct BlockInfo {
    pub seconds: f64,
    pub beats: f64,
    pub samplerate: f64,
    pub tempo: f64,
    pub flags: TransportFlags,
}

impl BlockInfo {
    pub fn new(seconds: f64, beats: f64, samplerate: f64, tempo: f64) -> Self {
        Self {
            seconds,
            beats,
            samplerate,
            tempo,
            flags: TransportFlags::empty(),
        }
    }

    /// Increments the internal timestamps based on a number of processed samples.
    pub fn advance_by_samples(&mut self, samples: usize) {
        if !self.is_playing() {
            return;
        }

        let delta_seconds = samples as f64 / self.samplerate;

        self.seconds += delta_seconds;
        self.beats += delta_seconds * self.tempo / 60.0;
    }

    pub fn is_playing(&self) -> bool {
        self.flags.contains(TransportFlags::IS_PLAYING)
    }
}

impl From<&BlockInfo> for Transport {
    fn from(info: &BlockInfo) -> Self {
        Transport {
            seconds: info.seconds,
            beats: info.beats,
            tempo: info.tempo,
            playing: info.is_playing(),
        }
    }
}

/// Host transport state at a given sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transport {
    pub seconds: f64,
    pub beats: f64,
    pub tempo: f64,
    pub playing: bool,
}

impl Transport {
    /// Returns the transport state `seconds` later, assuming constant tempo
    /// and no jump. Stopped transports do not move.
    #[inline]
    pub fn advanced_by_seconds(mut self, seconds: f64) -> Self {
        if self.playing {
            self.seconds += seconds;
            self.beats += seconds * self.tempo / 60.0;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockTime {
    pub sample_pos: u64,
    pub sample_rate: f64,
    pub transport: Option<Transport>,
}

impl BlockTime {
    #[inline]
    pub fn none() -> Self {
        Self {
            sample_pos: 0,
            sample_rate: 0.0,
            transport: None,
        }
    }

    /// Returns the timing `samples` later (or earlier if negative),
    /// extrapolating the transport.
    #[inline]
    pub fn advanced_by(&self, samples: i64) -> Self {
        let seconds = if self.sample_rate > 0.0 {
            samples as f64 / self.sample_rate
        } else {
            0.0
        };

        Self {
            sample_pos: self.sample_pos.saturating_add_signed(samples),
            sample_rate: self.sample_rate,
            transport: self.transport.map(|t| t.advanced_by_seconds(seconds)),
        }
    }

    /// Song position in seconds, if the host provided a transport.
    #[inline]
    pub fn seconds(&self) -> Option<f64> {
        self.transport.map(|t| t.seconds)
    }

    /// Song position in beats, if the host provided a transport.
    #[inline]
    pub fn beats(&self) -> Option<f64> {
        self.transport.map(|t| t.beats)
    }

    /// Whether the host transport is playing. `false` without transport.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.transport.is_some_and(|t| t.playing)
    }

    #[inline]
    pub fn beat_phase(&self) -> Option<f64> {
        self.beats().map(|b| b.rem_euclid(1.0))
    }

    #[inline]
    pub fn beat_num(&self) -> Option<i64> {
        self.beats().map(|b| b.floor() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {{
            let diff = ($a - $b).abs();
            let delta = 1e-6;
            if diff > delta {
                panic!(
                    "assert_approx_eq failed: {} ~= {} (diff: {}, default tolerance: {})",
                    $a, $b, diff, delta
                );
            }
        }};
        ($a:expr, $b:expr, $delta:expr) => {{
            let diff = ($a - $b).abs();
            if diff > $delta {
                panic!(
                    "assert_approx_eq failed: {} ~= {} (diff: {}, tolerance: {})",
                    $a, $b, diff, $delta
                );
            }
        }};
    }

    #[test]
    fn test_channel_info_not_last() {
        let infos = ChannelsInfo {
            current: 0,
            total_channels: 2,
        };

        assert!(!infos.is_last_channel())
    }

    #[test]
    fn test_channel_info_last() {
        let infos = ChannelsInfo {
            // Starting at index 0, this is last
            current: 1,
            total_channels: 2,
        };

        assert!(infos.is_last_channel())
    }

    #[test]
    fn test_block_info_advance() {
        let mut infos = BlockInfo {
            seconds: 0.,
            beats: 0.,
            samplerate: 44100.,
            tempo: 60.,
            flags: TransportFlags::IS_PLAYING,
        };

        // Advance by a full second
        infos.advance_by_samples(44100);

        assert_approx_eq!(infos.seconds, 1.);
        assert_approx_eq!(infos.beats, 1.);

        // Shouldn't change
        assert_approx_eq!(infos.samplerate, 44100.);
        assert_approx_eq!(infos.tempo, 60.);
    }

    #[test]
    fn test_block_info_advance_120_bpm() {
        let mut infos = BlockInfo::new(0., 0., 48000., 120.);
        infos.flags = TransportFlags::IS_PLAYING;

        infos.advance_by_samples(48000);

        assert_approx_eq!(infos.seconds, 1.);
        assert_approx_eq!(infos.beats, 2.);
    }

    fn transport(beats: f64, tempo: f64, playing: bool) -> Transport {
        Transport {
            seconds: beats * 60. / tempo,
            beats,
            tempo,
            playing,
        }
    }

    #[test]
    fn test_block_time_none() {
        let block = BlockTime::none();

        assert_eq!(block.sample_pos, 0);
        assert_eq!(block.seconds(), None);
        assert_eq!(block.beats(), None);
        assert_eq!(block.beat_num(), None);
        assert_eq!(block.beat_phase(), None);
        assert!(!block.is_playing());
    }

    #[test]
    fn test_block_time_some() {
        let block = BlockTime {
            sample_pos: 0,
            sample_rate: 48000.,
            transport: Some(transport(1.5, 60., true)),
        };

        assert_eq!(block.seconds(), Some(1.5));
        assert_eq!(block.beats(), Some(1.5));
        assert_eq!(block.beat_num(), Some(1));
        assert_eq!(block.beat_phase(), Some(0.5));
        assert!(block.is_playing());
    }

    #[test]
    fn test_block_time_negative_beats_phase() {
        // Pre-roll can report negative positions
        let block = BlockTime {
            sample_pos: 0,
            sample_rate: 48000.,
            transport: Some(transport(-0.25, 60., true)),
        };

        assert_eq!(block.beat_num(), Some(-1));
        assert_eq!(block.beat_phase(), Some(0.75));
    }

    #[test]
    fn test_block_time_advanced_playing() {
        let block = BlockTime {
            sample_pos: 100,
            sample_rate: 48000.,
            transport: Some(transport(0., 120., true)),
        };

        let later = block.advanced_by(24000);

        assert_eq!(later.sample_pos, 24100);
        assert_approx_eq!(later.seconds().unwrap(), 0.5);
        assert_approx_eq!(later.beats().unwrap(), 1.0);
    }

    #[test]
    fn test_block_time_advanced_stopped() {
        let block = BlockTime {
            sample_pos: 0,
            sample_rate: 48000.,
            transport: Some(transport(4., 120., false)),
        };

        let later = block.advanced_by(48000);

        assert_eq!(later.sample_pos, 48000);
        assert_eq!(later.beats(), Some(4.));
    }

    #[test]
    fn test_block_time_advanced_without_transport() {
        let later = BlockTime::none().advanced_by(10);

        assert_eq!(later.sample_pos, 10);
        assert_eq!(later.transport, None);
    }
}
