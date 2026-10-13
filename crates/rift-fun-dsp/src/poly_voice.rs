use fundsp::prelude32::*;
use rift_plugin_dsp::oscillator::OscillatorVoice;
use rift_plugin_types::{MidiMessage, MidiMessageKind};
use rift_plugin_utils::BoundedVec;

mod adsr;

pub use self::adsr::Adsr;
use self::adsr::{ACTIVE, INACTIVE, RELEASED};

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
    fn reset(&mut self, note: u8, config: &Self::Config);

    /// Sample rate in Hz, extracted from the voice configuration. Used to
    /// advance each voice's [`Adsr`].
    fn sample_rate(config: &Self::Config) -> f32;
}

impl VoiceSource for OscillatorVoice {
    type Config = f32;
    fn new(&samplerate: &Self::Config) -> Self {
        OscillatorVoice::new(samplerate, 0)
    }
    fn reset(&mut self, note: u8, &samplerate: &Self::Config) {
        OscillatorVoice::reset(self, samplerate, note);
    }
    fn sample_rate(&samplerate: &Self::Config) -> f32 {
        samplerate
    }
}

/// One slot of the voice pool: a reusable voice plus its envelope and gate.
struct VoiceSlot<T> {
    note: u8,
    inner: T,
    adsr: Adsr,
    processed_samples: u64,
    velocity: u8,

    /// 1.0 = active ; 0,0 = inactive ; 2.0 = released (about to get inactive)
    active: Shared,
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
        let active = Shared::new(INACTIVE);
        let mut adsr = Adsr::new(
            attack.clone(),
            decay.clone(),
            sustain.clone(),
            release.clone(),
            var(&active),
        );
        adsr.set_sample_rate(sample_rate);

        Self {
            note: 0,
            inner: T::new(config),
            adsr,
            processed_samples: 0,
            velocity: 0,
            active,
        }
    }

    /// Restarts the slot on `note` without allocating a new voice.
    fn trigger(&mut self, note: u8, velocity: u8, config: &T::Config) {
        self.inner.reset(note, config);
        self.note = note;
        self.velocity = velocity;
        self.processed_samples = 0;
        self.adsr.reset();
        self.active.set_value(ACTIVE);
    }
}

pub struct PolyVoice<V>
where
    V: VoiceSource,
{
    config: V::Config,

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
    pub fn new(config: V::Config) -> Self {
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
        config: V::Config,
        attack: An<Var>,
        decay: An<Var>,
        sustain: An<Var>,
        release: An<Var>,
    ) -> Self {
        let sample_rate = V::sample_rate(&config);
        let voices = std::array::from_fn(|_| {
            VoiceSlot::new(&config, &attack, &decay, &sustain, &release, sample_rate)
        });

        Self {
            config,
            voices,
            active_voices: BoundedVec::new(),
        }
    }

    /// Returns the index of an unused slot, or `None` when every voice is
    /// already sounding.
    fn free_slot(&self) -> Option<usize> {
        (0..256).find(|slot| !self.active_voices.contains(slot))
    }

    pub fn event(&mut self, midi: MidiMessage) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, velocity } => {
                let Some(slot) = self.free_slot() else {
                    return;
                };
                self.voices[slot].trigger(note, velocity, &self.config);
                self.active_voices.push(slot);
            }
            MidiMessageKind::NoteOff { note, .. } => {
                if let Some(slot) = self
                    .active_voices
                    .iter()
                    .copied()
                    .find(|&slot| self.voices[slot].note == note)
                {
                    // Hand the voice over to its envelope, which releases and
                    // eventually marks it `INACTIVE`.
                    self.voices[slot].active.set(RELEASED);
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
            // Safety:
            let voice = unsafe { self.voices.get_unchecked_mut(slot) };

            if voice.active.value() == INACTIVE {
                continue;
            }

            let adsr_gain = voice.adsr.tick();

            f(&mut voice.inner, adsr_gain, voice.velocity);

            voice.processed_samples += 1;
        }

        // Drop the slots whose envelope has fully released.
        let voices = &self.voices;
        self.active_voices
            .retain(|&slot| voices[slot].active.value() != INACTIVE);
    }
}
