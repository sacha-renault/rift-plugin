use fundsp::prelude32::*;
use rift_plugin_types::{MidiMessage, MidiMessageKind};
use rift_plugin_utils::{BoundedVec, DEFAULT_SAMPLERATE};

mod adsr;
mod oscillator_voice;

pub use self::adsr::Adsr;
pub use self::oscillator_voice::OscillatorVoice;

/// A single voice of polyphony.
///
/// Implementors own the per-voice synthesis state. The voice pool builds one
/// instance up front and reuses it for every note by calling [`reset`], instead
/// of allocating a fresh voice on each note-on.
///
/// [`reset`]: VoiceSource::reset
pub trait VoiceSource {
    type Config;

    /// Creates an idle voice. It stays silent until [`reset`] is called with
    /// the note to play.
    ///
    /// [`reset`]: VoiceSource::reset
    fn new(config: &Self::Config) -> Self;

    /// Re-initialises the voice to play `note` from a clean state, reusing its
    /// already-allocated state.
    fn reset(&mut self, note: u8);

    /// Sample rate in Hz, extracted from the voice configuration. Used to
    /// advance each voice's [`Adsr`].
    fn set_samplerate(&mut self, samplerate: f32);
}

/// One slot of the voice pool: a reusable voice plus its envelope and gate.
struct VoiceSlot<T> {
    note: u8,
    inner: T,
    adsr: Adsr,
    processed_samples: u64,
    velocity: u8,
}

impl<T> VoiceSlot<T>
where
    T: VoiceSource,
{
    /// Builds an idle slot whose envelope is driven by the shared ADSR
    /// parameters.
    fn new(
        config: &T::Config,
        attack: &An<Var>,
        decay: &An<Var>,
        sustain: &An<Var>,
        release: &An<Var>,
        sample_rate: f32,
    ) -> Self {
        let mut adsr = Adsr::new(
            attack.clone(),
            decay.clone(),
            sustain.clone(),
            release.clone(),
        );
        adsr.set_sample_rate(sample_rate);

        let mut inner = T::new(config);
        inner.set_samplerate(sample_rate);

        Self {
            note: 0,
            inner,
            adsr,
            processed_samples: 0,
            velocity: 0,
        }
    }

    /// Restarts the slot on `note` without allocating a new voice.
    fn trigger(&mut self, note: u8, velocity: u8) {
        self.inner.reset(note);
        self.note = note;
        self.velocity = velocity;
        self.processed_samples = 0;
        self.adsr.trigger();
    }
}

pub struct PolyVoice<V>
where
    V: VoiceSource,
{
    /// Fixed pool of voices, allocated once. Slots are reused across notes.
    voices: [VoiceSlot<V>; 256],
    /// Indices into `voices` that are currently sounding, in trigger order.
    active_voices: BoundedVec<usize, 256>,
}

impl<V> PolyVoice<V>
where
    V: VoiceSource,
{
    /// Creates a poly voice with a short, click-free default envelope
    /// (10 ms attack, 100 ms decay, full sustain, 100 ms release).
    pub fn new(config: &V::Config) -> Self {
        Self::with_adsr(
            config,
            var(&shared(0.01)),
            var(&shared(0.1)),
            var(&shared(1.0)),
            var(&shared(0.1)),
        )
    }

    /// Creates a poly voice whose envelopes are driven by the four shared
    /// ADSR parameters. `attack`, `decay` and `release` are times in seconds;
    /// `sustain` is a level in `0..=1`.
    pub fn with_adsr(
        config: &V::Config,
        attack: An<Var>,
        decay: An<Var>,
        sustain: An<Var>,
        release: An<Var>,
    ) -> Self {
        let sample_rate = DEFAULT_SAMPLERATE as f32;
        let voices = std::array::from_fn(|_| {
            VoiceSlot::new(config, &attack, &decay, &sustain, &release, sample_rate)
        });

        Self {
            voices,
            active_voices: BoundedVec::new(),
        }
    }

    /// Forwards the sample rate to every voice and its envelope. Call this once
    /// the real audio sample rate is known.
    pub fn set_samplerate(&mut self, samplerate: f32) {
        for voice in &mut self.voices {
            voice.inner.set_samplerate(samplerate);
            voice.adsr.set_sample_rate(samplerate);
        }
    }

    /// Returns the index of an unused slot, or `None` when every voice is
    /// already sounding.
    fn free_slot(&self) -> Option<usize> {
        (0..256).find(|slot| !self.active_voices.contains(slot))
    }

    /// Returns the active slot currently playing `note`, if any.
    fn slot_for_note(&self, note: u8) -> Option<usize> {
        self.active_voices
            .iter()
            .copied()
            .find(|&slot| self.voices[slot].note == note)
    }

    pub fn event(&mut self, midi: MidiMessage) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, velocity } => {
                // Reuse the slot already playing this note, if any, so rapid
                // retriggers don't pile up voices that the matching note-off
                // can never reach.
                let slot = match self.slot_for_note(note) {
                    Some(slot) => slot,
                    None => match self.free_slot() {
                        Some(slot) => {
                            self.active_voices.push(slot);
                            slot
                        }
                        None => return,
                    },
                };
                self.voices[slot].trigger(note, velocity);
            }
            MidiMessageKind::NoteOff { note, .. } => {
                // Hand the voice over to its envelope, which releases and
                // eventually marks it `Idle`.
                for &slot in self.active_voices.iter() {
                    if self.voices[slot].note == note {
                        self.voices[slot].adsr.release();
                    }
                }
            }
            _ => {}
        }
    }

    /// F: FnMut(voice: &mut V, adsr_gain: f32, velocity: u8),
    pub fn tick<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut V, f32, u8),
    {
        for slot in self.active_voices.iter().copied() {
            let voice = &mut self.voices[slot];

            if !voice.adsr.is_sounding() {
                continue;
            }

            let adsr_gain = voice.adsr.tick();

            f(&mut voice.inner, adsr_gain, voice.velocity);

            voice.processed_samples += 1;
        }

        // Drop the slots whose envelope has fully released.
        let voices = &self.voices;
        self.active_voices
            .retain(|&slot| voices[slot].adsr.is_sounding());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(kind: MidiMessageKind) -> MidiMessage {
        MidiMessage {
            channel: 0,
            port_index: 0,
            kind,
            time: 0,
        }
    }

    fn note_on(note: u8) -> MidiMessage {
        msg(MidiMessageKind::NoteOn {
            note,
            velocity: 100,
        })
    }

    fn note_off(note: u8) -> MidiMessage {
        msg(MidiMessageKind::NoteOff { note, velocity: 0 })
    }

    #[test]
    fn retrigger_reuses_slot_and_still_releases() {
        let mut poly = PolyVoice::<OscillatorVoice>::new(&());

        // First tap, released while its tail is still ringing.
        poly.event(note_on(60));
        poly.tick(|_, _, _| {});
        poly.event(note_off(60));

        // Second tap before the tail finished must reuse the same slot.
        poly.event(note_on(60));
        assert_eq!(
            poly.active_voices.len(),
            1,
            "retrigger should reuse the slot"
        );

        // Releasing the second tap must eventually free every voice.
        poly.event(note_off(60));
        for _ in 0..200_000 {
            poly.tick(|_, _, _| {});
            if poly.active_voices.is_empty() {
                break;
            }
        }
        assert!(poly.active_voices.is_empty(), "note never released");
    }
}
