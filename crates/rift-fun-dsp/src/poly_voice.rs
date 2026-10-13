use fundsp::prelude32::*;
use rift_plugin_dsp::oscillator::OscillatorVoice;
use rift_plugin_types::{MidiMessage, MidiMessageKind};
use rift_plugin_utils::BoundedVec;

mod adsr;

pub use self::adsr::Adsr;
use self::adsr::{ACTIVE, INACTIVE, RELEASED};

pub trait VoiceInit {
    type Config;

    fn create(note: u8, config: &Self::Config) -> Self;

    /// Sample rate in Hz, extracted from the voice configuration. Used to
    /// advance each voice's [`Adsr`].
    fn sample_rate(config: &Self::Config) -> f32;
}

impl VoiceInit for OscillatorVoice {
    type Config = f32;
    fn create(note: u8, &samplerate: &Self::Config) -> Self {
        Self::new(samplerate, note)
    }
    fn sample_rate(&samplerate: &Self::Config) -> f32 {
        samplerate
    }
}

struct Voice<T> {
    note: u8,
    inner: T,
    adsr: Adsr,
    processed_samples: u64,
    velocity: u8,

    /// 1.0 = active ; 0,0 = inactive ; 2.0 = released (about to get inactive)
    active: Shared,
}

pub struct PolyVoice<V, const N: usize>
where
    V: VoiceInit,
{
    config: V::Config,
    sample_rate: f32,

    /// Shared ADSR parameters, cloned into every voice together with that
    /// voice's own `active` gate.
    attack: An<Var>,
    decay: An<Var>,
    sustain: An<Var>,
    release: An<Var>,

    active_voices: BoundedVec<Voice<V>, N>,
    pending_remove: BoundedVec<usize, N>,
}

impl<V, const N: usize> PolyVoice<V, N>
where
    V: VoiceInit,
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
        Self {
            sample_rate: V::sample_rate(&config),
            config,
            attack,
            decay,
            sustain,
            release,
            active_voices: BoundedVec::new(),
            pending_remove: BoundedVec::new(),
        }
    }

    pub fn event(&mut self, midi: MidiMessage) {
        if self.active_voices.is_full() {
            return;
        }

        match midi.kind {
            MidiMessageKind::NoteOn { note, velocity } => {
                let active = Shared::new(ACTIVE);
                let mut adsr = Adsr::new(
                    self.attack.clone(),
                    self.decay.clone(),
                    self.sustain.clone(),
                    self.release.clone(),
                    var(&active),
                );
                adsr.set_sample_rate(self.sample_rate);

                self.active_voices.push(Voice {
                    note,
                    inner: V::create(note, &self.config),
                    adsr,
                    processed_samples: 0,
                    velocity,
                    active,
                });
            }
            MidiMessageKind::NoteOff { note, .. } => {
                if let Some(voice) = self.active_voices.iter().find(|v| v.note == note) {
                    // Hand the voice over to its envelope, which releases and
                    // eventually marks it `INACTIVE`.
                    voice.active.set(RELEASED);
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
        for (i, voice) in self.active_voices.iter_mut().enumerate() {
            if voice.active.value() == INACTIVE {
                self.pending_remove.push(i);
                continue;
            }

            let adsr_gain = voice.adsr.tick();

            f(&mut voice.inner, adsr_gain, voice.velocity);

            voice.processed_samples += 1;
        }

        while let Some(i) = self.pending_remove.pop() {
            self.active_voices.remove(i);
        }
    }
}
