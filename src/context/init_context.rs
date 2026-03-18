use std::sync::Arc;

use clack_plugin::host::HostMainThreadHandle;

use crate::wrapper::shared_states::SharedQueues;

/// Context for initializing plugin state during host callback requests.
pub struct InitContext<'a> {
    host: &'a HostMainThreadHandle<'a>,
    states: Arc<SharedQueues>,
    num_events: usize,
}

impl<'a> InitContext<'a> {
    pub(crate) fn new(host: &'a HostMainThreadHandle<'a>, states: Arc<SharedQueues>) -> Self {
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

    fn states(&self) -> Arc<SharedQueues> {
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
