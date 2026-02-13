use clack_extensions::params::*;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::prelude::*;

use crate::{
    params::param_trait::Params,
    wrapper::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared},
};

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
}

impl<'a, P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<'a, P> {
    fn flush(&mut self, input_events: &InputEvents, _output_events: &mut OutputEvents) {
        for event in input_events.iter() {
            let Some(param_event) = event.as_event::<ParamValueEvent>() else {
                continue;
            };
            let Some(id) = param_event.param_id() else {
                continue;
            };
            let value = param_event.value();
            self.shared.params.set_value(id, value);
        }

        // todo!()
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
        // Create the plugin instance here
        let mut plugin = P::create(shared.params.clone(), shared.other.clone());
        plugin.activate(audio_config);

        Ok(Self {
            shared: shared.clone(),
            plugin,
            host,
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut port_pair = audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut output_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        // TODO do with no allocation
        let mut channel_buffers = [None, None];

        // Extract the buffer slices that we need, while making sure they are paired correctly and
        // check for either in-place or separate buffers.
        for (pair, buf) in output_channels.iter_mut().zip(&mut channel_buffers) {
            *buf = match pair {
                ChannelPair::InputOnly(_) => None,
                ChannelPair::OutputOnly(_) => None,
                ChannelPair::InPlace(b) => Some(b),
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    Some(o)
                }
            }
        }

        self.flush(events.input, events.output);

        // todo!()
        // self.plugin.process()
        Ok(ProcessStatus::Continue)
    }
}
