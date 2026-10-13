use fundsp::prelude32::*;
use rift_plugin_dsp::oscillator::OscillatorVoice;
use rift_plugin_types::{MidiMessage, MidiMessageKind};
use rift_plugin_utils::BoundedVec;

pub trait VoiceInit {
    type Config;

    fn create(note: u8, config: &Self::Config) -> Self;
}

impl VoiceInit for OscillatorVoice {
    type Config = f32;
    fn create(note: u8, &samplerate: &Self::Config) -> Self {
        Self::new(samplerate, note)
    }
}

const INACTIVE: f32 = 0.0;
const ACTIVE: f32 = 1.0;
const RELEASED: f32 = 2.0;

struct Voice<T> {
    note: u8,
    inner: T,
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
    active_voices: BoundedVec<Voice<V>, N>,
    pending_remove: BoundedVec<usize, N>,
}

impl<V, const N: usize> PolyVoice<V, N>
where
    V: VoiceInit,
{
    pub fn new(config: V::Config) -> Self {
        Self {
            active_voices: BoundedVec::new(),
            pending_remove: BoundedVec::new(),
            config,
        }
    }

    pub fn event(&mut self, midi: MidiMessage) {
        if self.active_voices.is_full() {
            return;
        }

        match midi.kind {
            MidiMessageKind::NoteOn { note, velocity } => self.active_voices.push(Voice {
                note,
                inner: V::create(note, &self.config),
                processed_samples: 0,
                velocity,
                active: Shared::new(ACTIVE),
            }),
            MidiMessageKind::NoteOff { note, .. } => {
                self.active_voices
                    .iter()
                    .find(|v| v.note == note)
                    // TODO!(), current we directly set inactive
                    .map(|v| v.active.set(INACTIVE));
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

            // todo!, calculate some sort of "gain" from adsr
            let adsr_gain = 1.0;

            f(&mut voice.inner, adsr_gain, voice.velocity);

            voice.processed_samples += 1;
        }

        while let Some(i) = self.pending_remove.pop() {
            self.active_voices.remove(i);
        }
    }
}
