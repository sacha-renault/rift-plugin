use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::Arc;

use fundsp::prelude32::*;
use fundsp::wavetable::Wavetable;
use rift_fun_dsp::{Blend, WaveShape, WavetableMorph};
use rift_plugin::prelude::utils::BoundedVec;
use rift_plugin::prelude::*;

use crate::graph::{Shared, build_net};
use crate::params::{FunDspParams, OSC_COUNT, ParamFlags};

const WAVETABLES_CAPACITY: usize = 256;

pub struct FunDspPlugin {
    oscillators: [PolyVoice<OscillatorVoice>; OSC_COUNT],
    wavetables: [BoundedVec<Arc<Wavetable>, WAVETABLES_CAPACITY>; OSC_COUNT],
    graph: Box<dyn AudioUnit>,

    config: PluginAudioConfiguration,
    need_rebuild: HashSet<ClapId>,
}

impl ClapPlugin for FunDspPlugin {
    type Params = FunDspParams;
    type SharedData = Shared;

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

        // `wt_position` morphs 0 (pure sine) -> 1 (band-limited square).
        let wt: Vec<_> = WavetableMorph::new(MIN_PITCH, MAX_PITCH, TABLES_PER_OCTAVE)
            .shapes([WaveShape::sine(), WaveShape::square()])
            .blend(Blend::Crossfade)
            .build(WAVETABLES_CAPACITY);

        let oscp = &params.oscillators[0];
        let mut oscillator = PolyVoice::with_adsr(
            &(),
            oscp.attack.var(),
            oscp.decay.var(),
            oscp.sustain.var(),
            oscp.release.var(),
        );
        oscillator.set_samplerate(config.sample_rate as f32);

        let graph = build_net(config.sample_rate, params);

        let need_rebuild = HashSet::from_iter(
            params.params_by_tags(|tags| tags.intersects(ParamFlags::NEED_REBUILD)),
        );

        Self {
            oscillators: [oscillator],
            wavetables: [wt.try_into().unwrap()],
            graph,
            config,
            need_rebuild,
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        _ctx: ProcessContext,
        _events: &InputEvents,
        params: &Self::Params,
        data: &Self::SharedData,
    ) -> Result<ProcessStatus, PluginError> {
        // First we ensure we don't have a pending graph...
        if let Some(graph) = data.tasks.try_recv() {
            utils::swap_in(&mut self.graph, graph);
        }

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

                osc.tick(|voice, adsr_gain, info| {
                    let (value, hint) =
                        table.read(voice.table_hint(), voice.frequency(), voice.next_phase());
                    voice.set_table_hint(hint);

                    let v = adsr_gain * gain * value * info.velocity_scaled();

                    frame[0] += v * (0.5 - pan).clamp(0f32, 1f32);
                    frame[1] += v * (0.5 + pan).clamp(0f32, 1f32);
                });
            }

            self.graph.fill_frame(frame);
        });

        Ok(ProcessStatus::Continue)
    }

    fn on_param_change(
        &mut self,
        id: ClapId,
        _params: &Self::Params,
        shared: &Self::SharedData,
        _source: EventSource,
    ) {
        if self.need_rebuild.contains(&id) {
            let _ = shared.tasks.post(self.config.sample_rate);
        }
    }

    fn on_midi_message(&mut self, midi: MidiMessage, _: &Self::Params, _: &Self::SharedData) {
        for osc in &mut self.oscillators {
            osc.event(midi);
        }
    }

    fn gui(params: Arc<Self::Params>, _data: Arc<Self::SharedData>) -> Box<dyn GuiFactory> {
        crate::gui::build(params)
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

    const PARAM_MENU_ITEMS: &[MenuItem] = &[
        MenuItem::title(c"USER MENU"),
        MenuItem::entry(c"CANT CLICK THOSE", |_| {}).enabled_when(|_| false),
        MenuItem::entry(c"PANIKKK", |_| panic!("PANIK")),
    ];
}
