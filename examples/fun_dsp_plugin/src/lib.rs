use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::*;
use rift_plugin_vizia::prelude::*;

pub mod oscillator;

#[derive(Params)]
pub struct OscillatorParams {
    #[param(id = "Wt Position", default = 0)]
    pub wt_position: FloatParam,

    #[param(id = "Pan", default = 0, range=-0.5..0.5)]
    pub pan: FloatParam,

    #[param(id = "Gain", default = 1, range=0..2)]
    pub gain: FloatParam,
}

#[derive(Params)]
pub struct FilterParams {
    #[param(
        id = "Cutoff",
        range = 20..2000,
        default = 440.0,
    )]
    pub cutoff: SharedFloatParam,
}

const OSC_COUNT: usize = 1;

#[derive(Params)]
pub struct FunDspParams {
    #[nested]
    pub filters: [FilterParams; 1],

    #[nested]
    pub oscillators: [OscillatorParams; OSC_COUNT],
}

struct FunDspPlugin {
    oscillators: [crate::oscillator::Oscillator; OSC_COUNT],
}

impl ClapPlugin for FunDspPlugin {
    type Params = FunDspParams;
    type SharedData = ();

    const EVENT_PRE_PROCESS: EventPreProcess = EventPreProcess::all();

    fn create(
        _params: &Self::Params,
        config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        // Band-limited wavetable range shared by every morph position.
        const MIN_PITCH: f64 = 20.0;
        const MAX_PITCH: f64 = 20_000.0;
        const TABLES_PER_OCTAVE: f64 = 4.0;

        let wt = (0..256)
            .map(|idx| {
                // `idx / 255` morphs 0 (pure sine) -> 1 (band-limited square).
                let morph = idx as f64 / 255.0;

                // All partials are sine-phased, keeping the wave odd and symmetric.
                let phase = |_| 0.0;

                // A sine is just the fundamental; a square adds odd harmonics at 1/i.
                let amplitude = move |_pitch: f64, i: u32| {
                    if i.is_multiple_of(2) {
                        0.0
                    } else {
                        (1.0 - morph) * if i == 1 { 1.0 } else { 0.0 } + morph * (1.0 / i as f64)
                    }
                };

                Arc::new(Wavetable::new(
                    MIN_PITCH,
                    MAX_PITCH,
                    TABLES_PER_OCTAVE,
                    &phase,
                    &amplitude,
                ))
            })
            .collect();

        let mut oscillator = crate::oscillator::Oscillator::new(config.sample_rate);
        oscillator.set_wavetables(wt);

        // let mono_filter = || (pass() | var(&params.filters[0].cutoff) | dc(0.5f32)) >> lowpass();
        // let mut filter = mono_filter() | mono_filter();

        // AudioUnit::set_sample_rate(&mut filter, config.sample_rate);

        Self {
            oscillators: [oscillator],
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _ctx: ProcessContext,
        events: &InputEvents,
        params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        for (events, frame) in buffers.main().iter_samples().zip_events(events) {
            for _event in events {}

            let mut fr = [0f32; 2];

            for (oscillator, osc_params) in self.oscillators.iter_mut().zip(&params.oscillators) {
                let wt_pos = osc_params.wt_position.value();
                let pan = osc_params.pan.value();
                let gain = osc_params.gain.value();

                let v = oscillator.tick(wt_pos);

                fr[0] += gain * v * (0.5 - pan).clamp(0f32, 1f32);
                fr[1] += gain * v * (0.5 + pan).clamp(0f32, 1f32);
            }

            frame.fill(fr);
        }

        // self.synth.process(size, input, output);

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, midi: MidiMessage, _: &Self::Params, _: &Self::SharedData) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, .. } => {
                for osc in &mut self.oscillators {
                    osc.trigger(note, || 0f32);
                }
            }
            MidiMessageKind::NoteOff { note, .. } => {
                for osc in &mut self.oscillators {
                    osc.deactivate(note);
                }
            }
            _ => {}
        }
    }

    fn gui(_params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        vizia_gui(200, 200, move |cx, ctx| {})
    }

    const ID: &str = "com.rift.fun-dsp-generator";
    const NAME: &str = "Fun Dsp Generator";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&CStr] = &[
        features::INSTRUMENT,
        features::SYNTHESIZER,
        features::STEREO,
    ];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::OutputOnly(2);
    const MIDI_PORTS: &[MidiPort<'_>] = &[MidiPort::input(b"Notes")
        .supported_dialects(NoteDialects::MIDI)
        .preferred_dialect(NoteDialect::Midi)];
    const AUX_AUDIO_PORTS: &[AudioPort<'_>] = &[];
}

export_clap_plugin!(FunDspPlugin);
