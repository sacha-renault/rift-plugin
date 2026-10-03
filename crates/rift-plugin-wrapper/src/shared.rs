use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::ClapPlugin;
use rift_plugin_context::PluginSharedState;
use rift_plugin_params::UserParams;
use rift_plugin_params::params_wrapper::ParamsWrapper;

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user.
    pub(crate) params: Arc<P::Params>,
    /// Host-facing view over the very same parameters, keyed by [`ClapId`] for
    /// fast lookups.
    ///
    /// [`ClapId`]: clack_plugin::utils::ClapId
    pub(crate) host_params: Arc<ParamsWrapper>,
    /// Shared data, defined by the user.
    pub(crate) data: Arc<P::SharedData>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) states: Arc<PluginSharedState>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            host_params: Arc::clone(&self.host_params),
            data: Arc::clone(&self.data),
            states: Arc::clone(&self.states),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        // Box the user params first: the `ParamPtr`s collected below point into
        // this allocation, so it must not move afterwards.
        let params = Arc::new(P::Params::default());
        let host_params = Arc::new(ParamsWrapper::new(params.all_params()));
        let data = P::SharedData::default();

        Self {
            params,
            host_params,
            data: Arc::new(data),
            states: Arc::new(PluginSharedState::new(P::TASKS_CAPACITY)),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
