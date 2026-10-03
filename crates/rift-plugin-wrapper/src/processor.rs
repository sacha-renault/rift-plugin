use clack_extensions::params::*;
use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent, TransportFlags};
use clack_plugin::prelude::*;

use rift_plugin_gui::{GuiParamEvent, GuiParamEventKind, GuiTasks};
use rift_plugin_params::ParamCollection;
use rift_plugin_types::transport::BlockIndex;

use crate::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared};
use rift_plugin_buffers::Buffers;
use rift_plugin_context::{AudioThreadTask, InitContext, ProcessContext};
use rift_plugin_types::{EventPreProcess, EventSource};

/// Runs the audio callback under `assert_no_alloc` when the debug-only
/// `assert-no-alloc` feature is enabled, and calls it directly otherwise.
#[cfg(all(feature = "assert-no-alloc", debug_assertions))]
#[inline]
fn audio_callback<R>(f: impl FnOnce() -> R) -> R {
    assert_no_alloc::assert_no_alloc(f)
}

#[cfg(not(all(feature = "assert-no-alloc", debug_assertions)))]
#[inline]
fn audio_callback<R>(f: impl FnOnce() -> R) -> R {
    f()
}

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
    samplerate: f64,
    block_index: BlockIndex,
}

impl<'a, P: ClapPlugin> WrapperProcessor<'a, P> {
    fn handle_audio_thread_tasks(&mut self, outputs: &mut OutputEvents) {
        while let Some(task) = self.shared.states.pop_in_audio() {
            use AudioThreadTask::*;

            match task {
                GuiParamEvent(event) => self.handle_gui_param_change(event, outputs),
                RequestCallback => self.host.request_callback(),
            }
        }
    }

    fn request_flush(&self) {
        if let Some(ext) = self.host.get_extension::<HostParams>() {
            ext.request_flush(self.host.as_shared());
        } else {
            log::error!("Flush failed")
        }
    }

    fn handle_input_events(&mut self, events: &InputEvents) {
        let flags = P::EVENT_PRE_PROCESS;

        for event in events.iter() {
            if let Some(event) = event.as_event::<ParamValueEvent>() {
                let Some(id) = event.param_id() else {
                    continue;
                };

                if flags.contains(EventPreProcess::PARAM_APPLY_CHANGE) {
                    let value = event.value();
                    self.shared.host_params.set_value(id, value as f32);
                }

                if flags.contains(EventPreProcess::PARAM_NOTIFY_CHANGE) {
                    self.plugin.on_param_change(
                        id,
                        &self.shared.params,
                        &self.shared.data,
                        EventSource::Host,
                    );
                }

                _ = self.shared.states.post_to_gui(GuiTasks::ParamChanged {
                    id,
                    value: event.value() as f32,
                });
            } else if flags.contains(EventPreProcess::MIDI_NOTIFY_EVENT)
                && let Some(event) = event.as_event::<MidiEvent>()
            {
                self.plugin.on_midi_message(
                    (*event).into(),
                    &self.shared.params,
                    &self.shared.data,
                );
            }
        }
    }

    #[inline]
    fn handle_gui_param_change(&mut self, event: GuiParamEvent, outputs: &mut OutputEvents) {
        if let Some(raw_event) = event.maybe_to_raw()
            && let err @ Err(..) = outputs.try_push(raw_event)
        {
            log::error!("There was an error push event {err:?}")
        }

        match event.kind {
            GuiParamEventKind::GestureBegin | GuiParamEventKind::GestureEnd => self.request_flush(),
            GuiParamEventKind::Value(_) => {
                self.plugin.on_param_change(
                    event.param_id,
                    &self.shared.params,
                    &self.shared.data,
                    EventSource::GUI,
                );
            }
            GuiParamEventKind::ValueLess => self.plugin.on_param_change(
                event.param_id,
                &self.shared.params,
                &self.shared.data,
                EventSource::GUI,
            ),
        }
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<'a, P> {
    fn flush(&mut self, inputs: &InputEvents, outputs: &mut OutputEvents) {
        self.handle_audio_thread_tasks(outputs);
        self.handle_input_events(inputs);
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessor<'a, WrapperShared<P>, WrapperMainThread<'a, P>>
    for WrapperProcessor<'a, P>
{
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut WrapperMainThread<P>,
        shared: &'a WrapperShared<P>,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // Create the plugin instance & activate right away
        let init_context = InitContext::new(&main_thread.host, shared.states.clone());
        let plugin = P::create(shared.params.as_ref(), audio_config, init_context);
        shared.states.set_samplerate(audio_config.sample_rate);

        // Allocate a scratch buffer ONCE
        Ok(Self {
            shared: shared.clone(),
            plugin,
            host,
            samplerate: audio_config.sample_rate,
            block_index: BlockIndex(-1),
        })
    }

    fn process(
        &mut self,
        process: Process,
        audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        audio_callback(|| {
            self.flush(events.input, events.output);
            let buffers = Buffers::new(audio, P::MAIN_AUDIO_PORTS);

            if let Some(flags) = process.transport.map(|tr| tr.flags) {
                self.shared
                    .states
                    .set_is_playing(flags.contains(TransportFlags::IS_PLAYING));
            }

            let context = ProcessContext {
                host: &self.host,
                states: self.shared.states.clone(),
                process,
                samplerate: self.samplerate,
                num_events: 0,
                outputs_events: events.output,
                block_index: self.block_index.increment(),
                host_params: self.shared.host_params.clone(),
            };

            self.plugin.process(
                buffers,
                context,
                events.input,
                self.shared.params.as_ref(),
                self.shared.data.as_ref(),
            )
        })
    }

    fn deactivate(self, _: &mut WrapperMainThread<'a, P>) {
        self.plugin.deactivate();
    }

    fn reset(&mut self) {
        self.plugin.reset();
    }

    fn start_processing(&mut self) -> Result<(), PluginError> {
        self.plugin.start_processing()
    }

    fn stop_processing(&mut self) {
        self.plugin.stop_processing();
    }
}
