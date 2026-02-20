use std::sync::Arc;

use crate::wrapper::shared_states::PluginSharedState;

pub(crate) trait HostStatesGetter {
    fn states(&self) -> Arc<PluginSharedState>;
    fn increment_event_count(&mut self);
}
