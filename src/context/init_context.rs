use std::sync::Arc;

use clack_plugin::host::HostMainThreadHandle;

use crate::wrapper::shared_states::PluginSharedState;

pub struct InitContext<'a> {
    host: HostMainThreadHandle<'a>,
    states: Arc<PluginSharedState>,
}

impl<'a> InitContext<'a> {
    pub(crate) fn new(host: HostMainThreadHandle<'a>, states: Arc<PluginSharedState>) -> Self {
        Self { host, states }
    }
}
