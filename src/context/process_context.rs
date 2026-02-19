use std::sync::Arc;

use clack_plugin::host::HostAudioProcessorHandle;

use crate::wrapper::shared_states::PluginSharedState;

pub struct ProcessContext<'a> {
    host: &'a HostAudioProcessorHandle<'a>,
    states: Arc<PluginSharedState>,
}

impl<'a> ProcessContext<'a> {
    pub(crate) fn new(
        host: &'a HostAudioProcessorHandle<'a>,
        host_messages: Arc<PluginSharedState>,
    ) -> Self {
        Self {
            host,
            states: host_messages,
        }
    }

    pub fn request_restart(&self) {
        self.states.request_restart();
        self.host.request_callback();
    }
}
