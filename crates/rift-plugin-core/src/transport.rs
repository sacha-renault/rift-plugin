#[derive(Clone, Copy)]
pub struct ChannelsInfo {
    pub current: usize,
    pub total_channels: usize,
}

impl ChannelsInfo {
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
}

impl BlockInfo {
    /// Increments the internal timestamps based on a number of processed samples.
    pub fn advance_by_samples(&mut self, samples: usize) {
        let delta_seconds = samples as f64 / self.samplerate;

        // Advance seconds if they exist
        self.seconds += delta_seconds;
        self.beats += delta_seconds / self.tempo;
    }
}

#[derive(Clone, Copy)]
pub struct BlockTime {
    /// This define the timing (seconds and beats) withing the song
    /// of the first beat of the BUFFER this block belongs to
    /// We might see many blocks with same seconds or beats if buffer_size > N
    seconds: f64,
    beats: f64,
}

impl BlockTime {
    #[inline]
    pub fn new(seconds: f64, beats: f64) -> Self {
        Self { seconds, beats }
    }

    #[inline]
    pub fn new_opt(seconds: Option<f64>, beats: Option<f64>) -> Self {
        Self {
            seconds: seconds.unwrap_or(f64::NAN),
            beats: beats.unwrap_or(f64::NAN),
        }
    }

    #[inline]
    pub fn none() -> Self {
        Self {
            seconds: f64::NAN,
            beats: f64::NAN,
        }
    }

    pub fn seconds(&self) -> Option<f64> {
        if self.seconds.is_nan() {
            None
        } else {
            Some(self.seconds)
        }
    }

    pub fn beats(&self) -> Option<f64> {
        if self.beats.is_nan() {
            None
        } else {
            Some(self.beats)
        }
    }

    #[inline]
    pub fn beat_phase(&self) -> Option<f64> {
        self.beats().map(|b| b.fract())
    }

    #[inline]
    pub fn beat_num(&self) -> Option<i64> {
        self.beats().map(|b| b.floor() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx_eq;

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
            tempo: 1., // Case BPM = 60
        };

        // Advance by a full second
        infos.advance_by_samples(44100);

        assert_approx_eq!(infos.seconds, 1.);
        assert_approx_eq!(infos.beats, 1.);

        // Shouldn't change
        assert_approx_eq!(infos.samplerate, 44100.);
        assert_approx_eq!(infos.tempo, 1.);
    }

    #[test]
    fn test_block_time_none() {
        let block = BlockTime::none();

        assert_eq!(block.seconds(), None);
        assert_eq!(block.beats(), None);
        assert_eq!(block.beat_num(), None);
        assert_eq!(block.beat_phase(), None);
    }

    #[test]
    fn test_block_time_some() {
        let block = BlockTime::new(1.5, 1.5);

        assert_eq!(block.seconds(), Some(1.5));
        assert_eq!(block.beats(), Some(1.5));
        assert_eq!(block.beat_num(), Some(1));
        assert_eq!(block.beat_phase(), Some(0.5));
    }

    #[test]
    fn test_block_time_opt() {
        let block = BlockTime::new_opt(None, None);

        assert_eq!(block.seconds(), None);
        assert_eq!(block.beats(), None);

        let block = BlockTime::new_opt(Some(1.), Some(1.));

        assert!(block.seconds().is_some());
        assert!(block.beats().is_some());
    }
}
