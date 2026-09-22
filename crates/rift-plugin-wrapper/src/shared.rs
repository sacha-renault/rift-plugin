use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::ClapPlugin;
use rift_plugin_context::SharedQueues;

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user
    pub(crate) params: Arc<P::Params>,
    /// Shared data, defined by the user.
    pub(crate) data: Arc<P::SharedData>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) states: Arc<SharedQueues>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            data: Arc::clone(&self.data),
            states: Arc::clone(&self.states),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        let params = P::Params::default();
        let data = P::SharedData::default();

        Self {
            params: Arc::new(params),
            data: Arc::new(data),
            states: Arc::new(SharedQueues::new(P::TASKS_CAPACITY)),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
