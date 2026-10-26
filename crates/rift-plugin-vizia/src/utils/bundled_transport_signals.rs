use rift_plugin_types::TransportInfo;

use crate::dev_prelude::*;

/// Handles for every transport signal the host publishes, installed as a single
/// model so the per-frame update does one tree lookup instead of eleven.
#[derive(Clone, Copy)]
pub struct BundledTransportSignals {
    pub samplerate: SamplerateSignal,
    pub is_playing: IsPlayingSignal,
    pub is_recording: IsRecordingSignal,
    pub is_preroll: IsPrerollSignal,
    pub song_pos_beats: SongPosBeatsSignal,
    pub song_pos_secs: SongPosSecsSignal,
    pub tempo: TempoSignal,
    pub tempo_inc: TempoIncSignal,
    pub loop_region: LoopRegionSignal,
    pub time_signature: TimeSignatureSignal,
    pub bar: BarSignal,
}

impl Model for BundledTransportSignals {}

impl BundledTransportSignals {
    pub(crate) fn new() -> Self {
        Self {
            samplerate: SamplerateSignal::default(),
            is_playing: IsPlayingSignal::default(),
            is_recording: IsRecordingSignal::default(),
            is_preroll: IsPrerollSignal::default(),
            song_pos_beats: SongPosBeatsSignal::default(),
            song_pos_secs: SongPosSecsSignal::default(),
            tempo: TempoSignal::default(),
            tempo_inc: TempoIncSignal::default(),
            loop_region: LoopRegionSignal::default(),
            time_signature: TimeSignatureSignal::default(),
            bar: BarSignal::default(),
        }
    }

    /// Installs every transport signal and the bundle itself on the root entity.
    pub(crate) fn install(self, cx: &mut Context) {
        cx.with_current(Entity::root(), |cx| {
            self.build(cx);
            self.samplerate.build(cx);
            self.is_playing.build(cx);
            self.is_recording.build(cx);
            self.is_preroll.build(cx);
            self.song_pos_beats.build(cx);
            self.song_pos_secs.build(cx);
            self.tempo.build(cx);
            self.tempo_inc.build(cx);
            self.loop_region.build(cx);
            self.time_signature.build(cx);
            self.bar.build(cx);
        });
    }

    /// Fetches the bundle from the root entity.
    ///
    /// # Panics
    /// If the signals haven't been installed.
    pub fn from_context(cx: &mut Context) -> Self {
        cx.with_current(Entity::root(), |cx| {
            *cx.try_data::<Self>()
                .expect("BundledTransportSignals weren't installed in the app")
        })
    }

    /// Publishes the host transport, touching only the signals whose value changed.
    pub fn update(&self, info: TransportInfo) {
        // Should never change but meh
        self.samplerate.set_if_changed(info.sample_rate);

        self.is_playing.set_if_changed(info.is_playing);
        self.is_recording.set_if_changed(info.is_recording);
        self.is_preroll.set_if_changed(info.is_preroll);

        self.song_pos_beats.set_if_changed(info.song_pos_beats);
        self.song_pos_secs.set_if_changed(info.song_pos_seconds);

        self.tempo.set_if_changed(info.tempo);
        self.tempo_inc.set_if_changed(info.tempo_inc);

        self.loop_region.set_if_changed(info.loop_region);
        self.time_signature.set_if_changed(info.time_signature);
        self.bar.set_if_changed(info.bar);
    }
}
