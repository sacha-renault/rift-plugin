use clack_extensions::{audio_ports::*, gui::*, params::*, state::PluginStateImpl};
use clack_plugin::prelude::*;

use crate::wrapper::{ClapPlugin, main_thread::WrapperMainThread, shared::WrapperShared};

pub struct WrapperProcessor<P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
}

impl<P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<P> {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    ) {
        todo!()
    }
}

impl<'a, P: ClapPlugin> PluginAudioProcessor<'a, WrapperShared<P>, WrapperMainThread<P>>
    for WrapperProcessor<P>
{
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut WrapperMainThread<P>,
        shared: &'a WrapperShared<P>,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // Create the plugin instance here
        let mut plugin = P::default();
        plugin.activate(audio_config);

        Ok(Self {
            shared: shared.clone(),
            plugin,
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        todo!()
    }
}
