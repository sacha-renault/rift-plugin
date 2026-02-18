use clack_extensions::params::*;
use clack_plugin::extensions::HostExtensionSide;
use clack_plugin::prelude::*;
use clack_plugin::{events::event_types::ParamValueEvent, extensions::Extension};

use crate::{
    gui::ParamGuiEvent,
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
    #[inline]
    fn with_extension<E, F, R>(&self, func: F) -> Option<R>
    where
        E: Extension<ExtensionSide = HostExtensionSide>,
        F: FnOnce(&E) -> R,
    {
        self.host.get_extension::<E>().map(|ext| func(&ext))
    }

    fn request_flush(&self) -> bool {
        self.with_extension::<HostParams, _, _>(|host| host.request_flush(self.host.as_shared()))
            .is_some()
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

        self.shared.params.process_event(|event| {
            if let err @ Err(..) = outputs.try_push(&event) {
                log::error!("There was an error push event {err:?}")
            }

            match event {
                ParamGuiEvent::GestureStart(_) | ParamGuiEvent::GestureEnd(_) => {
                    if !self.request_flush() {
                        log::error!("Flush failed");
                    }
                }
                ParamGuiEvent::ValueEvent(_) => {}
            }
        });
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessor<'a, WrapperShared<P>, WrapperMainThread<'a, P>>
    for WrapperProcessor<'a, P>
{
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut WrapperMainThread<P>,
        shared: &'a WrapperShared<P>,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // Create the plugin instance & activate right away
        let mut plugin = P::create(shared.params.clone(), shared.other.clone());
        plugin.activate(audio_config);

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
        self.plugin.process(buffers)
    }
}
