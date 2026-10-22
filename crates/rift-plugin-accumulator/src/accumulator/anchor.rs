use rift_plugin_types::TripleBuffer;
use rift_plugin_types::transport::{BlockTime, TransportInfo};

/// Timing of the first sample of the latest written buffer. Any other
/// sample's timing is extrapolated from it, so loops/seeks inside a single UI
/// read are not tracked.
pub(crate) struct Anchor(TripleBuffer<BlockTime>);

impl Anchor {
    pub fn new() -> Self {
        Self(TripleBuffer::new(BlockTime::none()))
    }

    /// Records that the sample at absolute position `pos` has timing `info`.
    ///
    /// Audio thread only.
    pub fn store(&self, pos: u64, info: &TransportInfo) {
        // SAFETY: only the audio thread writes.
        unsafe { self.0.write(BlockTime::new(pos, *info)) };
    }

    /// Timing of the sample at absolute position `pos`, which may be before
    /// or after the anchor.
    ///
    /// UI thread only.
    pub fn time_at(&self, pos: u64) -> BlockTime {
        // SAFETY: only the UI thread reads.
        let anchor = unsafe { self.0.read() };
        anchor.advanced_by(pos as i64 - anchor.sample_pos as i64)
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_utils::assert_approx_eq;

    use super::*;

    fn playing_info(beats: f64, tempo: f64) -> TransportInfo {
        let mut info = TransportInfo::new(48_000.);
        info.is_playing = true;
        info.tempo = tempo;
        info.song_pos_beats = beats.into();
        info.song_pos_seconds = (beats * 60. / tempo).into();
        info
    }

    #[test]
    fn starts_without_timing() {
        let anchor = Anchor::new();
        let t = anchor.time_at(0);

        assert_eq!(t.sample_pos, 0);
        assert_eq!(t.beats(), 0.);
        assert!(!t.is_playing());
    }

    #[test]
    fn exact_position() {
        let anchor = Anchor::new();
        anchor.store(1000, &playing_info(2., 120.));

        let t = anchor.time_at(1000);
        assert_eq!(t.sample_pos, 1000);
        assert_eq!(t.sample_rate(), 48_000.);
        assert_eq!(t.beats(), 2.);
        assert!(t.is_playing());
    }

    #[test]
    fn extrapolates_forward_and_backward() {
        // 120 bpm at 48k: 24000 samples per beat
        let anchor = Anchor::new();
        anchor.store(48000, &playing_info(4., 120.));

        assert_approx_eq!(anchor.time_at(72000).beats(), 5.);
        assert_approx_eq!(anchor.time_at(24000).beats(), 3.);
    }
}
