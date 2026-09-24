use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::clack_extensions::gui::{GuiSize, Window};
use rift_plugin::prelude::clack_extensions::note_ports::{NoteDialect, NoteDialects};
use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::utils::notes::midi_to_frequency;
use rift_plugin::prelude::*;
use rift_plugin_gui::{ClapGui, GuiContext, GuiFactory};

params! {
    params {
        param frequency: SharedFloatParam {
            default: 440f32,
            min: 20f32,
            max: 2000f32,
        },
    }
}

struct FunDspPlugin {
    synth: Box<dyn AudioUnit>,
    freq: Shared,
    is_playing: bool,
}

impl ClapPlugin for FunDspPlugin {
    type Params = Parameters;
    type SharedData = ();

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = true;

    fn create(
        _params: &Self::Params,
        config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        let freq = Shared::new(440f32);
        let table = Arc::new(AtomicTable::from_wavetable(triangle_table(), 0));

        let synth = || var(&freq) >> An(AtomicSynth::<f32>::new(table.clone()));
        let mut stereo = synth() | synth();
        AudioUnit::set_sample_rate(&mut stereo, config.sample_rate);

        Self {
            synth: Box::new(stereo) as Box<dyn AudioUnit>,
            freq,
            is_playing: false,
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _context: ProcessContext,
        events: &InputEvents,
        _params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        // if !self.is_playing {
        //     buffers.main().zero_fill();
        //     return Ok(ProcessStatus::Continue);
        // }

        for (events, frame) in buffers.main().iter_samples().zip_events::<Self>(events) {
            for _event in events {}

            if self.is_playing {
                self.synth.fill_frame::<2>(frame);
            }
        }

        // self.synth.process(size, input, output);

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, midi: MidiMessage) {
        match midi.kind {
            MidiMessageKind::NoteOn { note, .. } => {
                self.is_playing = true;
                self.freq.set(midi_to_frequency(note));
            }
            MidiMessageKind::NoteOff { .. } => self.is_playing = false,
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
