//! Minimal CLAP plugin: a stereo gain with no GUI.
//!
//! The wrapper always builds a GUI factory when the main thread starts, even
//! though the `PluginGui` extension is not registered (see
//! `crates/rift-plugin-wrapper/src/factory.rs`). So we hand it [`NoGuiFactory`],
//! which builds a [`NoGui`] doing nothing.

use std::ffi::CStr;
use std::marker::PhantomData;
use std::sync::Arc;

use rift_plugin::prelude::clack_extensions::gui::{GuiSize, Window};
use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::*;
use rift_plugin_gui::{ClapGui, GuiContext, GuiFactory};

params! {
    params {
        param gain: FloatParam {
            default: 1f32,
            min: 0f32,
            max: 2f32,
        },
    }
}

/// Shared between the audio thread and the main/GUI thread.
struct SharedData<P: ClapPlugin> {
    _channels: usize,
    _p: PhantomData<P>,
}

impl<P: ClapPlugin> Default for SharedData<P> {
    fn default() -> Self {
        let channels = P::MAIN_AUDIO_PORTS.capacity()
            + P::AUX_AUDIO_PORTS
                .iter()
                .map(AudioPort::channels)
                .sum::<u32>();

        Self {
            _channels: channels as usize,
            _p: PhantomData,
        }
    }
}

struct MinimalGain {}

impl ClapPlugin for MinimalGain {
    type Params = Parameters;
    type SharedData = SharedData<Self>;

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = false;

    fn create(
        _params: &Self::Params,
        _config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        Self {}
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _context: ProcessContext,
        _input_events: &InputEvents,
        params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        let gain = params.gain.value();

        let mut main = buffers.main();
        for frame in main.iter_samples() {
            for sample in frame {
                *sample *= gain;
            }
        }

        Ok(ProcessStatus::Continue)
    }

    fn on_midi_message(&mut self, _midi: MidiMessage) {}

    fn param_changed(&mut self, _id: ClapId, _source: EventSource) {}

    fn gui(_params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        Box::new(NoGuiFactory)
    }

    const ID: &str = "com.rift.minimal-gain";
    const NAME: &str = "Minimal Gain";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&CStr] = &[features::AUDIO_EFFECT, features::STEREO];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::InputOutput(2);
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

export_clap_plugin!(MinimalGain);
