use clack_extensions::params::*;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::prelude::*;

use crate::context::{AudioThreadTask, InitContext, ProcessContext};
use crate::gui::GuiParamEventKind;
use crate::{
    gui::GuiParamEvent,
    params::param_trait::Params,
    prelude::Buffers,
    wrapper::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared},
};

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
}

impl<'a, P: ClapPlugin> WrapperProcessor<'a, P> {
    fn request_flush(&self) {
        if let Some(ext) = self.host.get_extension::<HostParams>() {
            ext.request_flush(self.host.as_shared());
        } else {
            log::error!("Flush failed")
        }
    }

    #[inline]
    fn handle_gui_param_change(&self, event: GuiParamEvent, outputs: &mut OutputEvents) {
        if let err @ Err(..) = outputs.try_push(event.to_raw()) {
            log::error!("There was an error push event {err:?}")
        }

        match event.kind {
            GuiParamEventKind::GestureBegin | GuiParamEventKind::GestureEnd => self.request_flush(),
            GuiParamEventKind::Value(_) => {}
        }
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<'a, P> {
    fn flush(&mut self, inputs: &InputEvents, outputs: &mut OutputEvents) {
        for event in inputs.iter() {
            if let Some(param_event) = event.as_event::<ParamValueEvent>() {
                let Some(id) = param_event.param_id() else {
                    continue;
                };
                let value = param_event.value();
                self.shared.params.set_value(id, value);
            };

            // todo!() ?
            // maybe handle other kind of events ?
            // if let Some(some_event) = event.as_event::<SomeEvent>() { ... }
        }

        while let Some(task) = self.shared.states.pop_audio_thread_tasks() {
            use AudioThreadTask::*;

            match task {
                GuiParamEvent(event) => self.handle_gui_param_change(event, outputs),
            }
        }
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
        let mut plugin = P::create(shared.params.clone(), shared.other.clone());
        let init_context = InitContext::new(&main_thread.host, shared.states.clone());
        plugin.activate(audio_config, init_context);

        // Allocate a scratch buffer ONCE
        Ok(Self {
            shared: shared.clone(),
            plugin,
            host,
        })
    }

    fn process(
        &mut self,
        _process: Process,
        audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        self.flush(events.input, events.output);
        let buffers = Buffers::new(audio, P::MAIN_AUDIO_PORTS);
        let context = ProcessContext::new(
            &self.host,
            self.shared.states.clone(),
            self.shared.other.clone(),
        );
        self.plugin.process(buffers, context)
    }
}
