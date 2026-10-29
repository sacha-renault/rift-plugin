//! Wrapper-owned state shared between the plugin's threads.
//!
//! `PluginSharedState` and the task queues it contains are deliberately
//! crate-private: plugins receive contexts that expose only the operations
//! they need, never the raw cross-thread state.

use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::ClapPlugin;
use rift_plugin_gui::HostParamChange;
use rift_plugin_params::UserParams;
use rift_plugin_params::params_wrapper::ParamsWrapper;
use rift_plugin_params::{ClapId, ParamCollection};

mod shared_states;
mod tasks;

pub(crate) use shared_states::PluginSharedState;
pub(crate) use tasks::ParamContextMenu;

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user.
    pub(crate) user_params: Arc<P::Params>,
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
            user_params: Arc::clone(&self.user_params),
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
        let data = P::SharedData::from_params(params.clone());

        Self {
            user_params: params,
            host_params,
            data: Arc::new(data),
            states: Arc::new(PluginSharedState::new(P::TASKS_CAPACITY)),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}

pub trait FromParams<P: UserParams> {
    fn from_params(params: Arc<P>) -> Self;
}

impl<T, P: UserParams> FromParams<P> for T
where
    T: Default,
{
    fn from_params(_: Arc<P>) -> Self {
        Self::default()
    }
}

/// Writes a host-driven parameter value and, when the GUI is open, queues it so
/// widgets follow automation.
///
/// `apply` is false when the plugin asked to own that value: the pre-block pass and
/// the per-frame one each apply only the half the other skipped.
#[inline]
pub(crate) fn apply_host_param(
    host_params: &ParamsWrapper,
    states: &PluginSharedState,
    apply: bool,
    notify_gui: bool,
    id: ClapId,
    value: f32,
) {
    if apply {
        host_params.set_value(id, value);
    }

    if notify_gui && states.is_gui_opened() {
        states
            .gui_param_changes()
            .post(HostParamChange { id, value });
    }
}
