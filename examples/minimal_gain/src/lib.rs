//! Minimal CLAP plugin: a stereo gain with no GUI.
//!
//! The wrapper always builds a GUI factory when the main thread starts, even
//! though the `PluginGui` extension is not registered (see
//! `crates/rift-plugin-wrapper/src/factory.rs`). So we hand it [`NoGuiFactory`],
//! which builds a [`NoGui`] doing nothing.

use std::ffi::CStr;
use std::sync::Arc;

use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::*;
use rift_plugin_accumulator::prelude::AudioAccumulator;
use rift_plugin_vizia::analysers::AccumulatorProxy;
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::{DialModifiers, param_knob};

#[derive(Params)]
pub struct MinimalGainParams {
    #[param(name = "Gain", range = linear(0, 2), default = 1.0)]
    pub gain: FloatParam,
}

/// Shared between the audio thread and the main/GUI thread.
struct SharedData {
    acc: Arc<AudioAccumulator>,
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            acc: Arc::new(AudioAccumulator::new(2, 44100)),
        }
    }
}

struct MinimalGain {}

impl ClapPlugin for MinimalGain {
    type Params = MinimalGainParams;
    type SharedData = SharedData;

    const EVENT_PRE_PROCESS: EventPreProcess =
        EventPreProcess::PARAM_APPLY_CHANGE.union(EventPreProcess::PARAM_NOTIFY_CHANGE);

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
        context: ProcessContext,
        _input_events: &InputEvents,
        params: &Self::Params,
        data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        let gain = params.gain.value();

        let mut main = buffers.main();
        for frame in main.iter_samples() {
            for sample in frame {
                *sample *= gain;
            }
        }

        data.acc
            .audio_thread_writes(&buffers.main(), context.block_info());

        if gain > 1.9 {
            let _ = context.background_task::<_, ()>(|| todo!());
        }

        Ok(ProcessStatus::Continue)
    }

    fn gui(params: Arc<Self::Params>, data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        vizia_gui(160, 200, move |cx, _| {
            let proxy = AccumulatorProxy::install(cx, data.acc.clone());

            HStack::new(cx, |cx| {
                param_knob(cx, &params.gain)
                    .large()
                    .height(Units::Stretch(1.))
                    .width(Units::Stretch(8.));

                HStack::new(cx, |cx| {
                    proxy.meter(cx, 0);
                    proxy.meter(cx, 1);
                })
                .height(Units::Percentage(80.))
                .width(Units::Stretch(2.));
            })
            .height(Units::Stretch(1.))
            .width(Units::Stretch(1.));
        })
    }

    const ID: &str = "com.rift.minimal-gain";
    const NAME: &str = "Minimal Gain";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&CStr] = &[features::AUDIO_EFFECT, features::STEREO];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::InputOutput(2);
}

export_clap_plugin!(MinimalGain);
