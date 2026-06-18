use core::f32;
use std::f32::consts::TAU;
use std::sync::Arc;

use clack_plugin::plugin::features;
use clack_plugin::prelude::*;

use rift_plugin::prelude::clack_extensions::note_ports::{NoteDialect, NoteDialects};
use rift_plugin::prelude::*;
use rift_plugin_dsp::oscillator::Oscillator;
use rift_plugin_vizia::utils::set_value;
use rift_plugin_vizia::*;

#[derive(Default, Debug, DeriveEnumValues)]
enum OscillatorType {
    #[default]
    Sin,
    Square,
    Saw,
}

impl OscillatorType {
    fn func(&self) -> fn(f32) -> f32 {
        use OscillatorType::*;

        match self {
            Sin => |phase| (TAU * phase).sin(),
            Square => |phase| if phase < 0.5 { -1. } else { 1. },
            Saw => |phase| (phase - 0.5) * 2.,
        }
    }
}

#[derive(DeriveParams)]
pub struct PluginParams {
    #[param(name = "OSC_TYPE")]
    generator_type: EnumParam<OscillatorType>,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            generator_type: EnumParam::new(OscillatorType::Sin),
        }
    }
}

pub struct Plugin {
    #[allow(unused)]
    params: Arc<PluginParams>,

    #[allow(unused)]
    shared: Arc<()>,

    osc: Oscillator,
}

impl ClapPlugin for Plugin {
    type ParamType = PluginParams;
    type SharedType = ();

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = false;

    fn create(
        params: Arc<Self::ParamType>,
        shared: Arc<Self::SharedType>,
        config: PluginAudioConfiguration,
        mut context: InitContext,
    ) -> Self {
        context.set_latency(0);

        Plugin {
            params,
            shared,
            osc: Oscillator::new(config.sample_rate as f32),
        }
    }

    fn on_midi_message(&mut self, _: MidiMessage) {}

    fn param_changed(&mut self, _: ClapId, _: EventSource) {}

    fn process(
        &mut self,
        mut buffers: Buffers,
        _: ProcessContext<Self>,
        events: &InputEvents,
    ) -> Result<ProcessStatus, PluginError> {
        let mut main = buffers.main();
        let osc_generator = self.params.generator_type.value().func();

        for (events, frame) in main.iter_samples().zip_events(events) {
            self.handle_frame_events(events);

            for sample in frame {
                *sample = self.osc.get_next(osc_generator);
            }
        }

        Ok(ProcessStatus::Continue)
    }

    fn gui(params: Arc<Self::ParamType>, _: Arc<Self::SharedType>) -> Box<dyn GuiFactory> {
        #[derive(Lens)]
        struct AppData {
            params: Arc<PluginParams>,
        }

        impl Model for AppData {}

        let ptr = params.generator_type.as_ptr();

        ViziaGui::factory((600, 200), move |cx, _| {
            AppData {
                params: params.clone(),
            }
            .build(cx);

            HStack::new(cx, |cx| {
                for index in 0..OscillatorType::count() {
                    let btn_name = format!("{:?}", OscillatorType::from_index(index).unwrap());
                    Button::new(cx, |cx| Label::new(cx, &btn_name))
                        .on_press(move |cx| set_value(ptr, cx, index as f32))
                        .toggle_class(
                            "accent",
                            AppData::params
                                .map(move |p| p.generator_type.value().to_index() == index),
                        );
                }
            });
        })
    }

    const ID: &str = "com.test.basicplugin";
    const NAME: &str = "Basic Instrument";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&std::ffi::CStr] = &[features::INSTRUMENT];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::OutputOnly(2);
    const MIDI_PORTS: &[MidiPort<'_>] = &[MidiPort::input(b"midi input")
        .supported_dialects(NoteDialects::MIDI)
        .preferred_dialect(NoteDialect::Midi)];
}

impl Plugin {
    fn handle_frame_events(&mut self, events: FrameEvents<'_, Self>) {
        for event in events {
            match event {
                InputEvent::MidiEvent(msg) => match msg.kind {
                    MidiMessageKind::NoteOn { note, .. } => {
                        self.osc.trigger(note, || 0.);
                    }
                    MidiMessageKind::NoteOff { note, .. } => {
                        self.osc.deactivate(note);
                    }
                    _ => {}
                },
                _ => (),
            }
        }
    }
}

export_clap_plugin!(Plugin);
