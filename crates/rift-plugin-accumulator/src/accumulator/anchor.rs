use rift_plugin_types::TripleBuffer;
use rift_plugin_types::transport::{BlockInfo, BlockTime, Transport};

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
    pub fn store(&self, pos: u64, info: Option<&BlockInfo>) {
        let time = BlockTime {
            sample_pos: pos,
            sample_rate: info.map_or(0.0, |i| i.samplerate),
            transport: info.map(Transport::from),
        };

        // SAFETY: only the audio thread writes.
        unsafe { self.0.write(time) };
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
    use rift_plugin_types::transport::TransportFlags;

    use super::*;

    fn playing_info(beats: f64, tempo: f64) -> BlockInfo {
        let mut info = BlockInfo::new(beats * 60. / tempo, beats, 48000., tempo);
        info.flags = TransportFlags::IS_PLAYING;
        info
    }

    #[test]
    fn starts_without_timing() {
        let anchor = Anchor::new();
        let t = anchor.time_at(0);

        assert_eq!(t.sample_pos, 0);
        assert_eq!(t.transport, None);
    }

    #[test]
    fn exact_position() {
        let anchor = Anchor::new();
        anchor.store(1000, Some(&playing_info(2., 120.)));

        let t = anchor.time_at(1000);
        assert_eq!(t.sample_pos, 1000);
        assert_eq!(t.sample_rate, 48000.);
        assert_eq!(t.beats(), Some(2.));
        assert!(t.is_playing());
    }

    #[test]
    fn extrapolates_forward_and_backward() {
        // 120 bpm at 48k: 24000 samples per beat
        let anchor = Anchor::new();
        anchor.store(48000, Some(&playing_info(4., 120.)));

        assert_eq!(anchor.time_at(72000).beats(), Some(5.));
        assert_eq!(anchor.time_at(24000).beats(), Some(3.));
    }

    #[test]
    fn without_transport() {
        let anchor = Anchor::new();
        anchor.store(10, None);

        let t = anchor.time_at(15);
        assert_eq!(t.sample_pos, 15);
        assert_eq!(t.transport, None);
    }
}
