use std::sync::Arc;

use clack_plugin::host::HostMainThreadHandle;

use crate::wrapper::shared_states::PluginSharedState;

pub(crate) trait MainThreadContextGetter {
    fn host(&self) -> &HostMainThreadHandle<'_>;
    fn host_mut(&mut self) -> &mut HostMainThreadHandle<'_>;
    fn states(&self) -> Arc<PluginSharedState>;
}
