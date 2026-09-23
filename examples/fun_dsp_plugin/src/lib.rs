use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::clack_extensions::gui::{GuiSize, Window};
use rift_plugin::prelude::clack_plugin::plugin::features;
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
    generator: Box<dyn AudioUnit>,
}

impl ClapPlugin for FunDspPlugin {
    type Params = Parameters;
    type SharedData = ();

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = false;

    fn create(
        params: &Self::Params,
        _config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        let mono = || var(&params.frequency) >> sine();
        let stereo = mono() | mono();

        Self {
            generator: Box::new(stereo) as Box<dyn AudioUnit>,
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _context: ProcessContext,
        _input_events: &InputEvents,
        _params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        buffers.main().for_each_stereo_frame(|[left, right]| {
            (*left, *right) = self.generator.get_stereo();
        });

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, _midi: MidiMessage) {}

    fn param_changed(&mut self, _id: ClapId, _source: EventSource) {}

    fn gui(_params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        Box::new(NoGuiFactory)
    }

    const ID: &str = "com.rift.fun-dsp-generator";
    const NAME: &str = "Fun Dsp Generator";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&CStr] = &[features::SYNTHESIZER, features::STEREO];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::OutputOnly(2);
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
