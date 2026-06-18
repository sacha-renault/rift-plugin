use std::sync::Arc;
use std::sync::atomic::Ordering;

use clack_plugin::plugin::features;
use clack_plugin::prelude::*;

use rift_plugin::prelude::clack_extensions::note_ports::{NoteDialect, NoteDialects};
use rift_plugin::prelude::*;

use rift_plugin_accumulator::prelude::*;
use rift_plugin_core::utils::lfo::{Lfo, LfoFreq, LfoMode};
use rift_plugin_dsp::filters::biquads::*;

use crate::params::PluginParams;
use crate::shared::Shared;

pub struct Plugin {
    params: Arc<PluginParams>,
    shared: Arc<Shared>,
    lfo1: Lfo,
    filter: MultiChannel<BiquadCascade>,
}

impl ClapPlugin for Plugin {
    type ParamType = PluginParams;
    type SharedType = Shared;

    const PARAM_EVENT_AUTO_HANDLING: bool = true;
    const MIDI_EVENT_AUTO_HANDLING: bool = false;

    fn create(
        params: Arc<Self::ParamType>,
        shared: Arc<Self::SharedType>,
        config: PluginAudioConfiguration,
        mut context: InitContext,
    ) -> Self {
        let samplerate = config.sample_rate as f32;
        context.set_latency(0);
        let lfo1 = Lfo::new(
            LfoMode::Classic,
            LfoFreq::Beats(params.lfo_time.value()),
            samplerate,
            params.lfo.value().clone(),
        );

        let filter = MultiChannel::new(2, || {
            let mut cascade = BiquadCascade::new(samplerate);
            let mode = FilterMode::HighPass {
                cutoff: params.cutoff.value(),
                order: FilterOrder::Eight,
            };
            cascade.set_mode(mode);
            cascade
        });

        Plugin {
            params,
            shared,
            lfo1,
            filter,
        }
    }

    fn on_midi_message(&mut self, _: MidiMessage) {}

    fn param_changed(&mut self, id: ClapId, _: EventSource) {
        use crate::params::param_ids::*;

        if id == LFO_TIME_ID {
            let lfo_time = self.params.lfo_time.value();
            self.lfo1.set_frequency(LfoFreq::Beats(lfo_time));
        } else if id == LFO_ID {
            self.lfo1.set_control_points(self.params.lfo.value());
        } else if id == cutoff_ID {
            self.filter.apply_all(|filter| {
                filter.set_mode(FilterMode::HighPass {
                    cutoff: self.params.cutoff.value(),
                    order: FilterOrder::Eight, // this is just placeholder, put a dropdown ?
                })
            });
        }
    }

    fn process(
        &mut self,
        mut buffers: Buffers,
        ctx: ProcessContext<Self>,
        events: &InputEvents,
    ) -> Result<ProcessStatus, PluginError> {
        self.process_start(&ctx);

        if self.params.skip.value() {
            return Ok(ProcessStatus::Continue);
        }

        let clip = self.params.clip.value();
        let gain = self.params.gain.value();

        let mut main = buffers.main();
        for (events, frame) in main.iter_samples().zip_events::<Self>(events) {
            for event in events {
                self.handle_timed_events(event)
            }

            let lfo_value = self.lfo1.get_next();

            for (idx, sample) in frame.enumerate() {
                let x = sample.clamp(-clip, clip) * gain * lfo_value;
                *sample = self.filter.with_channel_mut(idx, |b| b.process(x))
            }
        }

        let _ = self.process_end(&mut buffers, ctx);
        Ok(ProcessStatus::Continue)
    }

    fn gui(params: Arc<Self::ParamType>, shared: Arc<Self::SharedType>) -> Box<dyn GuiFactory> {
        crate::gui::create_gui(params, shared)
    }

    const ID: &str = "com.test.basicplugin";
    const NAME: &str = "Basic Plugin";
    const VERSION: &str = "0.1.0";
    const FEATURES: &[&std::ffi::CStr] = &[features::AUDIO_EFFECT];

    const MAIN_AUDIO_PORTS: MainAudioPort = MainAudioPort::InputOutput(2);
    const AUX_AUDIO_PORTS: &[AudioPort<'_>] = &[];
    const MIDI_PORTS: &[MidiPort<'_>] = &[MidiPort::input(b"midi input")
        .supported_dialects(NoteDialects::MIDI)
        .preferred_dialect(NoteDialect::Midi)];
}

impl Plugin {
    // ... Rest of ClapPlugin implementation
    fn process_start(&mut self, ctx: &ProcessContext<Self>) {
        self.lfo1.set_block_info(ctx.block_info());
    }

    fn process_end(
        &mut self,
        buffers: &mut Buffers,
        ctx: ProcessContext<Self>,
    ) -> Result<ProcessStatus, PluginError> {
        let main = buffers.main();

        self.shared
            .post_acc
            .push_slices(&mut main.iter_channels(), ctx.block_info());

        // Update lfo positions
        self.shared
            .lfo_position
            .store(self.lfo1.get_position(), Ordering::Relaxed);

        Ok(ProcessStatus::Continue)
    }

    fn handle_timed_events(&mut self, event: InputEvent) {
        match event {
            InputEvent::MidiEvent(msg) if matches!(msg.kind, MidiMessageKind::NoteOn { .. }) => {
                self.lfo1.retrigger();
            }
            _ => unreachable!("Param event are set to auto handled"),
        }
    }
}
