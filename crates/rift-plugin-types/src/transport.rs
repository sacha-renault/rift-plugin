pub use clack_plugin::events::event_types::TransportFlags;
use clack_plugin::{
    events::event_types::TransportEvent,
    utils::{BeatTime, SecondsTime},
};
use rift_plugin_utils::{DEFAULT_SAMPLERATE, DEFAULT_TEMPO};

#[derive(Clone, Copy)]
pub struct ChannelsInfo {
    pub current: usize,
    pub total_channels: usize,
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
            .then_some(LoopRegion {
                start_beats: event.loop_start_beats,
                end_beats: event.loop_end_beats,
                start_seconds: event.loop_start_seconds,
                end_seconds: event.loop_end_seconds,
            });

        self.time_signature = event
            .flags
            .contains(TransportFlags::HAS_TIME_SIGNATURE)
            .then_some(TimeSignature {
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
        let samples = self.last_sample_count as f64;
        self.shift(samples);
    }

    /// Moves the position by `samples`, positive or negative.
    fn shift(&mut self, samples: f64) {
        if self.sample_rate <= 0.0 {
            return;
        }

        let seconds = samples / self.sample_rate;
        self.song_pos_seconds = self.song_pos_seconds + seconds.into();

        // Tempo increment is constant so avg tempo is (start_tempo + end_tempo) / 2
        let avg_tempo = self.tempo + self.tempo_inc * samples / 2.0;
        self.song_pos_beats = self.song_pos_beats + (seconds * avg_tempo / 60.0).into();
    }

    /// Returns this transport `samples` later (or earlier if negative).
    /// Stopped transports do not move.
    pub fn advanced_by(&self, samples: i64) -> Self {
        let mut next = *self;
        if next.is_playing {
            next.shift(samples as f64);
        }
        next
    }
}

/// Timing of the first sample of a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockTime {
    pub sample_pos: u64,
    pub transport: TransportInfo,
}

impl BlockTime {
    /// Builds the timing of the sample at `sample_pos` from its transport.
    #[inline]
    pub fn new(sample_pos: u64, transport: TransportInfo) -> Self {
        Self {
            sample_pos,
            transport,
        }
    }

    #[inline]
    pub fn none() -> Self {
        Self::new(0, TransportInfo::new(DEFAULT_SAMPLERATE))
    }

    #[inline]
    pub fn empty_with_sr(samplerate: f64) -> Self {
        Self::new(0, TransportInfo::new(samplerate))
    }

    /// Returns the timing `samples` later (or earlier if negative),
    /// extrapolating the transport.
    #[inline]
    pub fn advanced_by(&self, samples: i64) -> Self {
        Self {
            sample_pos: self.sample_pos.saturating_add_signed(samples),
            transport: self.transport.advanced_by(samples),
        }
    }

    #[inline]
    pub fn seconds(&self) -> f64 {
        self.transport.song_pos_seconds.to_float()
    }

    #[inline]
    pub fn beats(&self) -> f64 {
        self.transport.song_pos_beats.to_float()
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        self.transport.is_playing
    }

    #[inline]
    pub fn beat_phase(&self) -> f64 {
        self.beats().rem_euclid(1.0)
    }

    #[inline]
    pub fn beat_num(&self) -> i64 {
        self.beats().floor() as i64
    }

    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.transport.sample_rate
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

    fn transport(beats: f64, tempo: f64, playing: bool) -> TransportInfo {
        let mut info = TransportInfo::new(48_000.);
        info.is_playing = playing;
        info.tempo = tempo;
        info.song_pos_beats = beats.into();
        info.song_pos_seconds = (beats * 60. / tempo).into();
        info
    }

    #[test]
    fn test_block_time_none() {
        let block = BlockTime::none();

        assert_eq!(block.sample_pos, 0);
        assert_eq!(block.seconds(), 0.);
        assert_eq!(block.beats(), 0.);
        assert_eq!(block.beat_num(), 0);
        assert_eq!(block.beat_phase(), 0.);
        assert!(!block.is_playing());
        assert_eq!(block.sample_rate(), DEFAULT_SAMPLERATE);
    }

    #[test]
    fn test_block_time_some() {
        let block = BlockTime::new(0, transport(1.5, 60., true));

        assert_eq!(block.seconds(), 1.5);
        assert_eq!(block.beats(), 1.5);
        assert_eq!(block.beat_num(), 1);
        assert_eq!(block.beat_phase(), 0.5);
        assert!(block.is_playing());
        assert_eq!(block.sample_rate(), 48_000.);
    }

    #[test]
    fn test_block_time_negative_beats_phase() {
        // Pre-roll can report negative positions
        let block = BlockTime::new(0, transport(-0.25, 60., true));

        assert_eq!(block.beat_num(), -1);
        assert_eq!(block.beat_phase(), 0.75);
    }

    #[test]
    fn test_block_time_advanced_playing() {
        let block = BlockTime::new(100, transport(0., 120., true));

        let later = block.advanced_by(24000);

        assert_eq!(later.sample_pos, 24100);
        assert_approx_eq!(later.seconds(), 0.5);
        assert_approx_eq!(later.beats(), 1.0);
    }

    #[test]
    fn test_block_time_advanced_stopped() {
        let block = BlockTime::new(0, transport(4., 120., false));

        let later = block.advanced_by(48000);

        assert_eq!(later.sample_pos, 48000);
        assert_eq!(later.beats(), 4.);
    }

    #[test]
    fn test_block_time_advanced_default_transport() {
        let later = BlockTime::none().advanced_by(10);

        assert_eq!(later.sample_pos, 10);
        assert_eq!(later.beats(), 0.);
    }
}
