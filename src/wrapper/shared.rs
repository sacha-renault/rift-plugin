use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::{
    _sealed::__ParamsInitializer,
    wrapper::{ClapPlugin, shared_states::SharedQueues},
};

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user
    pub(crate) params: Arc<P::ParamType>,
    /// Any shared data, defined also by the user
    pub(crate) other: Arc<P::SharedType>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) states: Arc<SharedQueues>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            other: Arc::clone(&self.other),
            states: Arc::clone(&self.states),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        let mut params = P::ParamType::default();
        params.__initialize();

        Self {
            params: Arc::new(params),
            other: Arc::new(P::SharedType::default()),
            states: Arc::new(SharedQueues::default()),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
