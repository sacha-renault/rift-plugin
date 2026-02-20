use std::sync::Arc;

use clack_plugin::host::HostAudioProcessorHandle;

use crate::wrapper::shared_states::PluginSharedState;

pub struct ProcessContext<'a> {
    host: &'a HostAudioProcessorHandle<'a>,
    states: Arc<PluginSharedState>,
    num_events: usize,
}

impl<'a> ProcessContext<'a> {
    pub(crate) fn new(
        host: &'a HostAudioProcessorHandle<'a>,
        host_messages: Arc<PluginSharedState>,
    ) -> Self {
        Self {
            host,
            states: host_messages,
            num_events: 0,
        }
    }
}

impl<'a> super::HostStatesGetter for ProcessContext<'a> {
    #[inline]
    fn increment_event_count(&mut self) {
        self.num_events += 1;
    }

    #[inline]
    fn states(&self) -> Arc<PluginSharedState> {
        self.states.clone()
    }
}

impl<'a> Drop for ProcessContext<'a> {
    fn drop(&mut self) {
        if self.num_events > 0 {
            self.host.request_callback();
        }
    }
}
