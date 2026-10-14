use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use rift_plugin::prelude::clack_plugin::plugin::features;
use rift_plugin::prelude::*;
use rift_plugin_accumulator::prelude::AudioAccumulator;
use rift_plugin_vizia::analysers::AccumulatorProxy;
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::*;

const DEFAULT_LOW_CUT_HZ: f32 = 120.0;
const DEFAULT_HIGH_CUT_HZ: f32 = 8000.0;
const DEFAULT_REVERB_MIX: f32 = 0.25;
const DEFAULT_TIME_S: f32 = 3.0;
const FILTER_Q: f32 = 0.707;
const ROOM_SIZE: f32 = 10.0;
const DAMPING: f32 = 0.5;

#[derive(Params)]
pub struct ReverbParams {
    #[param(name = "Low Cut", range = skew(20, 20000, 5), default = DEFAULT_LOW_CUT_HZ)]
    pub low_cut: SharedFloatParam,

    #[param(name = "High Cut", range = skew(20, 20000, 5), default = DEFAULT_HIGH_CUT_HZ)]
    pub high_cut: SharedFloatParam,

    #[param(name = "Mix", range = linear(0, 1), default = DEFAULT_REVERB_MIX)]
    pub mix: SharedFloatParam,
}

struct SharedData {
    acc: Arc<AudioAccumulator>,
}

impl FromParams<ReverbParams> for SharedData {
    fn create(_: Arc<ReverbParams>) -> Self {
        Self {
            acc: Arc::new(AudioAccumulator::new(2, 44100)),
        }
    }
}

struct FunDspReverb {
    graph: Box<dyn AudioUnit>,
}

fn build_graph(params: &ReverbParams, sample_rate: f64, time_s: f32) -> Box<dyn AudioUnit> {
    // fundsp SVFs take inputs positionally as `(signal, cutoff, Q)`;
    // every input is `f32` so the type system can't catch a swap.
    let hp_l = (pass() | params.low_cut.var() | dc(FILTER_Q)) >> highpass();
    let hp_r = (pass() | params.low_cut.var() | dc(FILTER_Q)) >> highpass();
    let lp_l = (pass() | params.high_cut.var() | dc(FILTER_Q)) >> lowpass();
    let lp_r = (pass() | params.high_cut.var() | dc(FILTER_Q)) >> lowpass();

    let filters_stereo = (hp_l | hp_r) >> (lp_l | lp_r);
    let wet = filters_stereo >> reverb_stereo(ROOM_SIZE, time_s, DAMPING);
    let dry = multipass::<U2>();

    // `var()` is mono; broadcast to stereo by stacking two reads
    // so `*` matches the (stereo) wet/dry counts.
    let mix_stereo = || params.mix.var() | params.mix.var();
    let inv_mix_stereo = || (dc(1.0) - params.mix.var()) | (dc(1.0) - params.mix.var());

    // `&` is Bus: dry + wet share the input and sum their outputs.
    let mut graph: Box<dyn AudioUnit> = Box::new((dry * inv_mix_stereo()) & (wet * mix_stereo()));
    graph.set_sample_rate(sample_rate);
    graph.allocate();

    graph
}

impl ClapPlugin for FunDspReverb {
    type Params = ReverbParams;
    type SharedData = SharedData;

    const EVENT_PRE_PROCESS: EventPreProcess = EventPreProcess::all();

    fn create(
        params: &Self::Params,
        config: PluginAudioConfiguration,
        _context: InitContext,
    ) -> Self {
        Self {
            graph: build_graph(params, config.sample_rate, DEFAULT_TIME_S),
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _ctx: ProcessContext,
        _events: &InputEvents,
        _params: &Self::Params,
        _data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        buffers.main().for_each_stereo_frame(|frame| {
            let mut output = [0.; 2];
            self.graph.tick(frame, &mut output);
            frame.copy_from_slice(&output);
        });

        Ok(ProcessStatus::Continue)
    }

    fn gui(params: Arc<Self::Params>, data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        vizia_gui(320, 240, move |cx, _| {
            AccumulatorProxy::install(cx, data.acc.clone());

            HStack::new(cx, |cx| {
                param_knob(cx, &params.low_cut);
                param_knob(cx, &params.high_cut);
            });
        })
    }

    const ID: &str = "com.rift.minimal-gain";
    const NAME: &str = "Minimal Gain";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&CStr] = &[features::AUDIO_EFFECT, features::STEREO];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::InputOutput(2);
}

export_clap_plugin!(FunDspReverb);
