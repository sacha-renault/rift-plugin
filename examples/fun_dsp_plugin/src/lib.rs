use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::clack_extensions::gui::{GuiSize, Window};
use rift_plugin::prelude::clack_extensions::note_ports::{NoteDialect, NoteDialects};
use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::utils::notes::midi_to_frequency;
use rift_plugin::prelude::*;
use rift_plugin_dsp::oscillator::OscillatorPosition;
use rift_plugin_gui::{ClapGui, GuiContext, GuiFactory};

params! {
    params {
        filters: Array<1> {
            param cutoff: SharedFloatParam {
                default: 440f32,
                min: 20f32,
                max: 2000f32,
            },
        },

        param wt_position: IntParam {
            default: 0,
            min:0,
            max: 255,
        }
    }
}

struct FunDspPlugin {
    synths: [Box<dyn AudioUnit>; 256],
    phase: OscillatorPosition,
}

impl ClapPlugin for FunDspPlugin {
    type Params = Parameters;
    type SharedData = ();

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = true;

    fn create(
        params: &Self::Params,
        config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        let synth = |table| An(PhaseSynth::new(table));

        // Band-limited wavetable range shared by every morph position.
        const MIN_PITCH: f64 = 20.0;
        const MAX_PITCH: f64 = 20_000.0;
        const TABLES_PER_OCTAVE: f64 = 4.0;

        let synths = std::array::from_fn(|idx| {
            // `idx / 255` morphs 0 (pure sine) -> 1 (band-limited square).
            let morph = idx as f64 / 255.0;

            // All partials are sine-phased, keeping the wave odd and symmetric.
            let phase = |_i: u32| 0.0;

            // A sine is just the fundamental; a square adds odd harmonics at 1/i.
            let amplitude = move |_pitch: f64, i: u32| {
                if i % 2 == 0 {
                    0.0
                } else {
                    (1.0 - morph) * if i == 1 { 1.0 } else { 0.0 } + morph * (1.0 / i as f64)
                }
            };

            let table = Wavetable::new(MIN_PITCH, MAX_PITCH, TABLES_PER_OCTAVE, &phase, &amplitude);
            let mut mono = synth(Arc::new(table));
            AudioUnit::set_sample_rate(&mut mono, config.sample_rate);
            Box::new(mono) as Box<dyn AudioUnit>
        });

        let mono_filter = || (pass() | var(&params.filters[0].cutoff) | dc(0.5f32)) >> lowpass();
        let mut filter = mono_filter() | mono_filter();

        AudioUnit::set_sample_rate(&mut filter, config.sample_rate);

        Self {
            synths,
            phase: OscillatorPosition::new(config.sample_rate as f32),
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _context: ProcessContext,
        events: &InputEvents,
        params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        if !self.phase.is_active() {
            buffers.main().zero_fill();
            return Ok(ProcessStatus::Continue);
        }

        let wt_pos = params.wt_position.value() as usize;

        for (events, frame) in buffers.main().iter_samples().zip_events::<Self>(events) {
            for _event in events {}

            let value = self.synths[wt_pos].filter_mono(self.phase.get_next_phase());
            // TODO, filter stuff, later

            for sample in frame {
                *sample = value;
            }
        }

        // self.synth.process(size, input, output);

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, midi: MidiMessage) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, .. } => {
                self.phase.trigger(midi_to_frequency(note));
            }
            MidiMessageKind::NoteOff { .. } => self.phase.deactivate(),
            _ => {}
        }
    }

    fn param_changed(&mut self, _id: ClapId, _source: EventSource) {}

    fn gui(_params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        Box::new(NoGuiFactory)
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

/// No-op GUI. Every CLAP GUI callback succeeds and does nothing.
struct NoGui;

impl ClapGui for NoGui {
    fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
        Ok(())
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        None
    }

    fn can_resize(&mut self) -> bool {
        false
    }

    fn adjust_size(&mut self, _size: GuiSize) -> Option<GuiSize> {
        None
    }

    fn set_size(&mut self, _size: GuiSize) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_parent(&mut self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_transient(&mut self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn show(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

struct NoGuiFactory;

impl GuiFactory for NoGuiFactory {
    fn build(self: Box<Self>, _states: Arc<dyn GuiContext>) -> Box<dyn ClapGui> {
        Box::new(NoGui)
    }
}

export_clap_plugin!(FunDspPlugin);
