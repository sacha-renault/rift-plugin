use std::sync::Arc;

use clack_plugin::host::HostAudioProcessorHandle;

use crate::wrapper::hosts_messages::HostsMessages;

pub struct ProcessContext<'a> {
    host: &'a HostAudioProcessorHandle<'a>,
    host_messages: Arc<HostsMessages>,
}

impl<'a> ProcessContext<'a> {
    pub(crate) fn new(
        host: &'a HostAudioProcessorHandle<'a>,
        host_messages: Arc<HostsMessages>,
    ) -> Self {
        Self {
            host,
            host_messages,
        }
    }

    pub fn request_latency_update(&self, latency: u32) {
        self.host_messages.set_latency(latency);
        self.host.request_callback();
    }
}
