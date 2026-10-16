use clack_extensions::params::*;
use clack_plugin::prelude::*;

use crate::ClapPlugin;

use super::WrapperProcessor;

impl<'a, P: ClapPlugin> PluginAudioProcessorParams for WrapperProcessor<'a, P> {
    fn flush(&mut self, inputs: &InputEvents, outputs: &mut OutputEvents) {
        self.handle_audio_thread_tasks(outputs);
        self.handle_input_events(inputs);
    }
}
