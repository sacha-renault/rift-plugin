use clack_extensions::params::*;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::prelude::*;

use crate::{
    params::param_trait::Params,
    prelude::Buffers,
    wrapper::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared},
};

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
    output_scratch: Vec<&'a mut [f32]>,
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
            if let err @ Err(..) = outputs.try_push(event) {
                log::error!("There was an error push event {err:?}")
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
        // Count number of port for
        let total_output = P::AUDIO_PORTS
            .iter()
            .filter(|port| !port.is_input)
            .fold(0, |acc, port| acc + port.channel_count);

        log::debug!("PluginAudioProcessor::activate with {total_output} ports");

        // Create the plugin instance & activate right away
        let mut plugin = P::create(shared.params.clone(), shared.other.clone());
        plugin.activate(audio_config);

        // Allocate a scratch buffer ONCE
        Ok(Self {
            shared: shared.clone(),
            plugin,
            host,
            output_scratch: Vec::with_capacity(total_output as usize),
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        // TODO
        // Get more than only main port_pair
        // This requires ClapPlugin::process change of interface
        // with a new Option<AuxInput> / Option<AuxOutput> ?
        let mut port_pair = audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;
        let mut output_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        // Clear but buffer will not have to reallocate :)
        self.output_scratch.clear();

        // Extract the buffer slices that we need, while making sure they are paired correctly and
        // check for either in-place or separate buffers.
        for pair in output_channels.iter_mut() {
            let slice = match pair {
                ChannelPair::InputOnly(_) => {
                    panic!("TODO: plugin doesn't expect any InputOnly")
                }
                ChannelPair::OutputOnly(o) => o,
                ChannelPair::InPlace(b) => b,
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    o
                }
            };

            // We lie to the rust compiler because that's simpler for now ...
            // TODO
            let wrong_lifetime_slice: &'a mut [f32] = unsafe { std::mem::transmute(slice) };
            self.output_scratch.push(wrong_lifetime_slice);
        }

        self.flush(events.input, events.output);

        let buffers = Buffers::new(&self.output_scratch);
        let output_status = self.plugin.process(buffers);

        // MANDATORY
        // WITH transmute CALL WE HAVE TO CLEAR output_scratch
        // OTHERWISE UNDEF BEHAVIOR !!!!
        self.output_scratch.clear();
        output_status
    }
}
