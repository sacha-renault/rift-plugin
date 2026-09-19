use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::wrapper::{ClapPlugin, shared_states::SharedQueues};

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user
    pub(crate) params: Arc<P::ParamType>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) states: Arc<SharedQueues>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            states: Arc::clone(&self.states),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        let params = P::ParamType::default();

        Self {
            params: Arc::new(params),
            states: Arc::new(SharedQueues::new(P::TASKS_CAPACITY)),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
