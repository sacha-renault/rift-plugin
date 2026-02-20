use std::sync::Arc;

use clack_plugin::host::HostMainThreadHandle;

use crate::wrapper::shared_states::PluginSharedState;

pub struct InitContext<'a> {
    host: &'a HostMainThreadHandle<'a>,
    states: Arc<PluginSharedState>,
    num_events: usize,
}

impl<'a> InitContext<'a> {
    pub(crate) fn new(host: &'a HostMainThreadHandle<'a>, states: Arc<PluginSharedState>) -> Self {
        Self {
            host,
            states,
            num_events: 0,
        }
    }
}

impl<'a> super::HostStatesGetter for InitContext<'a> {
    fn increment_event_count(&mut self) {
        self.num_events += 1;
    }

    fn states(&self) -> Arc<PluginSharedState> {
        self.states.clone()
    }
}

impl<'a> Drop for InitContext<'a> {
    fn drop(&mut self) {
        if self.num_events > 0 {
            self.host.request_callback();
        }
    }
}
