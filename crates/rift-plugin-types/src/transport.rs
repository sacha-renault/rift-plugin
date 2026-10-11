pub use clack_plugin::events::event_types::TransportFlags;

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
