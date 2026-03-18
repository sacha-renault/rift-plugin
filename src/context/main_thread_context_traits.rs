use std::sync::Arc;

use crate::wrapper::shared_states::SharedQueues;

pub(crate) trait HostStatesGetter {
    /// Exposes the cloned Arc to external code requiring the shared state.
    fn states(&self) -> Arc<SharedQueues>;

    /// Increments the pending event count. Used to track interactions before drop.
    ///
    /// Any type that implement this must also implement request callback on drop.
    fn increment_event_count(&mut self);
}
