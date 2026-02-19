use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::wrapper::{ClapPlugin, hosts_messages::HostsMessages};

pub struct WrapperShared<P: ClapPlugin> {
    /// Params of the plugin, defined by the user
    pub(crate) params: Arc<P::ParamType>,
    /// Any shared data, defined also by the user
    pub(crate) other: Arc<P::SharedType>,
    /// Internal messaging system between Audio and Main(GUI) thread
    pub(crate) host_messages: Arc<HostsMessages>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            other: Arc::clone(&self.other),
            host_messages: Arc::clone(&self.host_messages),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        Self {
            params: Arc::new(P::ParamType::default()),
            other: Arc::new(P::SharedType::default()),
            host_messages: Arc::new(HostsMessages::default()),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
