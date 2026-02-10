use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::wrapper::ClapPlugin;

pub struct WrapperShared<P: ClapPlugin> {
    pub(crate) params: Arc<P::ParamType>,
    pub(crate) other: Arc<P::Shared>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
            other: Arc::clone(&self.other),
        }
    }
}

impl<P: ClapPlugin> Default for WrapperShared<P> {
    fn default() -> Self {
        Self {
            params: Arc::new(P::ParamType::default()),
            other: Arc::new(P::Shared::default()),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
