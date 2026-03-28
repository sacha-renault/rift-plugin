use std::sync::atomic::Ordering;

use clack_extensions::params::*;
use clack_plugin::events::event_types::{
    MidiEvent, ParamValueEvent, TransportEvent, TransportFlags,
};
use clack_plugin::prelude::*;

use rift_plugin_core::gui::{GuiParamEvent, GuiParamEventKind};
use rift_plugin_core::params::Params;
use rift_plugin_core::transport::BlockIndex;

use crate::context::{AudioThreadTask, InitContext, ProcessContext};
use crate::prelude::Buffers;
use crate::ty::EventSource;
use crate::wrapper::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared};

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
    samplerate: f64,
    block_index: BlockIndex,
}

impl<'a, P: ClapPlugin> WrapperProcessor<'a, P> {
    fn handle_audio_thread_tasks(&mut self, outputs: &mut OutputEvents) {
        while let Some(task) = self.shared.states.pop_audio_thread_tasks() {
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

    fn handle_event_auto(&mut self, events: &InputEvents) {
        if !P::MIDI_EVENT_AUTO_HANDLING && !P::PARAM_EVENT_AUTO_HANDLING {
            return;
        }

        for event in events.iter() {
            if let Some(event) = event.as_event::<ParamValueEvent>() {
                if P::PARAM_EVENT_AUTO_HANDLING
                    && let Some(id) = event.param_id()
                {
                    let value = event.value();
                    self.shared.params.set_value(id, value);
                    self.plugin.param_changed(id, EventSource::Host);
                }
            } else if let Some(event) = event.as_event::<MidiEvent>() {
                if P::MIDI_EVENT_AUTO_HANDLING {
                    self.plugin.on_midi_message((*event).into());
                }
            } else if let Some(event) = event.as_event::<TransportEvent>() {
                log::info!("{event:?}");
            }
        }
    }

    #[inline]
    fn handle_gui_param_change(&mut self, event: GuiParamEvent, outputs: &mut OutputEvents) {
        if let Some(raw_event) = event.maybe_to_raw() {
            if let err @ Err(..) = outputs.try_push(raw_event) {
                log::error!("There was an error push event {err:?}")
            }
        }

        match event.kind {
            GuiParamEventKind::GestureBegin | GuiParamEventKind::GestureEnd => self.request_flush(),
            GuiParamEventKind::Value(_) => {
                self.plugin.param_changed(event.param_id, EventSource::GUI);
            }
            GuiParamEventKind::ValueLess => {
                self.plugin.param_changed(event.param_id, EventSource::GUI)
            }
        }
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<'a, P> {
    fn flush(&mut self, inputs: &InputEvents, outputs: &mut OutputEvents) {
        self.handle_audio_thread_tasks(outputs);
        self.handle_event_auto(inputs);
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
        let plugin = P::create(
            shared.params.clone(),
            shared.other.clone(),
            audio_config,
            init_context,
        );

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
        self.flush(events.input, events.output);
        let buffers = Buffers::new(audio, P::MAIN_AUDIO_PORTS);

        if let Some(flags) = process.transport.map(|tr| tr.flags) {
            self.shared.states.is_playing.store(
                flags.contains(TransportFlags::IS_PLAYING),
                Ordering::Relaxed,
            );
        }

        let context = ProcessContext {
            host: &self.host,
            states: self.shared.states.clone(),
            shared: self.shared.other.clone(),
            process,
            samplerate: self.samplerate,
            num_events: 0,
            outputs_events: events.output,
            block_index: self.block_index.increment(),
        };

        self.plugin.process(buffers, context, events.input)
    }
}
