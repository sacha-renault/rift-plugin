use std::ffi::CStr;
use std::sync::Arc;

use fundsp::wavetable::Wavetable;
use rift_plugin::prelude::utils::BoundedVec;
use rift_plugin::prelude::*;
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::*;

const WAVETABLES_CAPACITY: usize = 256;

#[derive(Params)]
pub struct OscillatorParams {
    #[param(id = "WtPosition", default = 0)]
    pub wt_position: FloatParam,

    #[param(id = "Pan", default = 0, range = skew(-0.5, 0.5, 1))]
    pub pan: FloatParam,

    #[param(id = "Gain", default = 1, range = linear(0, 2))]
    pub gain: FloatParam,

    #[param(id = "Attack", range = skew(0.001, 4.0, 3), default = 0.01)]
    pub attack: SharedFloatParam,

    #[param(id = "Decay", range = skew(0.001, 4.0, 3), default = 0.1)]
    pub decay: SharedFloatParam,

    #[param(id = "Sustain", range = linear(0.0, 1.0), default = 1.0)]
    pub sustain: SharedFloatParam,

    #[param(id = "Release", range = skew(0.001, 4.0, 3), default = 0.2)]
    pub release: SharedFloatParam,
}

#[derive(Params)]
pub struct FilterParams {
    #[param(
        id = "Cutoff",
        range = linear(20, 2000),
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
    oscillators: [PolyVoice<OscillatorVoice>; OSC_COUNT],
    wavetables: [BoundedVec<Arc<Wavetable>, WAVETABLES_CAPACITY>; OSC_COUNT],
}

impl ClapPlugin for FunDspPlugin {
    type Params = FunDspParams;
    type SharedData = ();

    const EVENT_PRE_PROCESS: EventPreProcess = EventPreProcess::all();

    fn create(
        params: &Self::Params,
        config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        // Band-limited wavetable range shared by every morph position.
        const MIN_PITCH: f64 = 20.0;
        const MAX_PITCH: f64 = 20_000.0;
        const TABLES_PER_OCTAVE: f64 = 4.0;

        let wt: Vec<_> = (0..256)
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

        let oscp = &params.oscillators[0];
        let mut oscillator = PolyVoice::with_adsr(
            &(),
            oscp.attack.var(),
            oscp.decay.var(),
            oscp.sustain.var(),
            oscp.release.var(),
        );
        oscillator.set_samplerate(config.sample_rate as f32);

        use fundsp::prelude32::*;

        // let mono_filter = || (pass() | params.filters[0].cutoff.var() | dc(0.5f32)) >> lowpass();
        // let mut filter = mono_filter() | mono_filter();

        // AudioUnit::set_sample_rate(&mut filter, config.sample_rate);

        Self {
            oscillators: [oscillator],
            wavetables: [wt.try_into().unwrap()],
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _ctx: ProcessContext,
        _events: &InputEvents,
        params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        buffers.main().for_each_stereo_frame(|frame| {
            frame.fill(0.);

            for ((osc, oscp), wavetable) in self
                .oscillators
                .iter_mut()
                .zip(&params.oscillators)
                .zip(&self.wavetables)
            {
                let wt_pos = oscp.wt_position.value();
                let wt = ((wavetable.len() - 1) as f32 * wt_pos.clamp(0.0, 1.0)) as usize;
                let table = &wavetable[wt];

                let pan = oscp.pan.value();
                let gain = oscp.gain.value();

                osc.tick(|voice, adsr_gain, velocity| {
                    let (value, hint) =
                        table.read(voice.table_hint(), voice.frequency(), voice.next_phase());
                    voice.set_table_hint(hint);

                    const MAX_VELOCITY: f32 = u8::MAX as f32;
                    let v = adsr_gain * gain * value * (velocity as f32 / MAX_VELOCITY);

                    frame[0] += v * (0.5 - pan).clamp(0f32, 1f32);
                    frame[1] += v * (0.5 + pan).clamp(0f32, 1f32);
                });
            }
        });

        // self.synth.process(size, input, output);

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, midi: MidiMessage, _: &Self::Params, _: &Self::SharedData) {
        for osc in &mut self.oscillators {
            osc.event(midi);
        }
    }

    fn gui(params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        vizia_gui(480, 320, move |cx, _| {
            VStack::new(cx, |cx| {
                Panel::new(cx, "Osc1", |cx| {
                    param_knob(cx, &params.oscillators[0].wt_position);
                    param_knob(cx, &params.oscillators[0].gain);
                    param_knob(cx, &params.oscillators[0].pan);
                });
                Panel::new(cx, "Adsr", |cx| {
                    param_knob(cx, &params.oscillators[0].attack);
                    param_knob(cx, &params.oscillators[0].decay);
                    param_knob(cx, &params.oscillators[0].sustain);
                    param_knob(cx, &params.oscillators[0].release);
                });
            });
        })
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
