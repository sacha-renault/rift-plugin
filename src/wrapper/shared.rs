use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::wrapper::{ClapPlugin, shared_states::PluginSharedState};

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user
    pub(crate) params: Arc<P::ParamType>,
    /// Any shared data, defined also by the user
    pub(crate) other: Arc<P::SharedType>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) states: Arc<PluginSharedState>,
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
        let mut states = PluginSharedState::new();
        for acc in P::accumulators() {
            states = states.add_accumulator(acc);
        }

        Self {
            params: Arc::new(P::ParamType::default()),
            other: Arc::new(P::SharedType::default()),
            states: Arc::new(states),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
