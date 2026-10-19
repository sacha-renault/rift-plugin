use std::sync::Arc;

use clack_plugin::host::HostMainThreadHandle;

use crate::PluginSharedState;

/// Context for initializing plugin state during host callback requests.
pub struct InitContext<'a> {
    host: &'a HostMainThreadHandle<'a>,
    states: Arc<PluginSharedState>,
}

impl<'a> InitContext<'a> {
    pub fn new(host: &'a HostMainThreadHandle<'a>, states: Arc<PluginSharedState>) -> Self {
        Self { host, states }
    }

    /// Asks the host to restart the plugin.
    pub fn request_restart(&mut self) {
        self.host.request_restart();
    }

    /// Reports a new processing latency, in samples.
    pub fn set_latency(&mut self, latency: u32) {
        // On init context. We don't have to do anything else.
        // Latency will be set be changed right after the activation
        self.states.set_latency(latency);
    }
}
