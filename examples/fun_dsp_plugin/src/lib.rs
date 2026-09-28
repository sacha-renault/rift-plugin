use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::clack_extensions::gui::{GuiSize, Window};
use rift_plugin::prelude::clack_extensions::note_ports::{NoteDialect, NoteDialects};
use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::utils::notes::midi_to_frequency;
use rift_plugin::prelude::*;
use rift_plugin_gui::{ClapGui, GuiContext, GuiFactory};

pub mod oscillator;

#[derive(Params)]
pub struct OscillatorParams {
    #[param(name = "Wt Position", range = 0..255, default = 0)]
    pub wt_position: IntParam,
}

#[derive(Params)]
pub struct FilterParams {
    #[param(
        name = "Cutoff",
        range = 20..2000,
        default = 440.0,
    )]
    pub cutoff: SharedFloatParam,
}

#[derive(Params)]
pub struct FunDspParams {
    #[nested]
    pub filters: [FilterParams; 1],

    #[nested]
    pub oscillators: [OscillatorParams; 1],
}

struct FunDspPlugin {
    oscillator: crate::oscillator::Oscillator,
}

impl ClapPlugin for FunDspPlugin {
    type Params = FunDspParams;
    type SharedData = ();

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = true;

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
                let phase = |_i: u32| 0.0;

                // A sine is just the fundamental; a square adds odd harmonics at 1/i.
                let amplitude = move |_pitch: f64, i: u32| {
                    if i % 2 == 0 {
                        0.0
                    } else {
                        (1.0 - morph) * if i == 1 { 1.0 } else { 0.0 } + morph * (1.0 / i as f64)
                    }
                };

                Wavetable::new(MIN_PITCH, MAX_PITCH, TABLES_PER_OCTAVE, &phase, &amplitude)
            })
            .collect();

        let mut oscillator = crate::oscillator::Oscillator::new(config.sample_rate, 256, 256);
        oscillator.set_wavetables(wt);

        // let mono_filter = || (pass() | var(&params.filters[0].cutoff) | dc(0.5f32)) >> lowpass();
        // let mut filter = mono_filter() | mono_filter();

        // AudioUnit::set_sample_rate(&mut filter, config.sample_rate);

        Self { oscillator }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _context: ProcessContext,
        events: &InputEvents,
        params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        for (events, frame) in buffers.main().iter_samples().zip_events::<Self>(events) {
            for _event in events {}

            let mut fr = [0f32; 2];

            for oscillator in &params.oscillators {
                let wt_pos = oscillator.wt_position.value();
                let [l, r] = self.oscillator.tick(wt_pos as usize);

                fr[0] += l;
                fr[1] += r;
            }

            frame.fill(fr);
        }

        // self.synth.process(size, input, output);

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, midi: MidiMessage) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, .. } => {
                self.oscillator
                    .trigger(note, midi_to_frequency(note), || 0f32);
            }
            MidiMessageKind::NoteOff { note, .. } => self.oscillator.deactivate(note),
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
