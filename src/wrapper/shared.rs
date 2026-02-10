use std::sync::Arc;

use clack_plugin::plugin::PluginShared;

use crate::wrapper::ClapPlugin;

#[derive(Default)]
pub struct WrapperShared<P: ClapPlugin> {
    params: Arc<P::ParamType>,
}

impl<P: ClapPlugin> Clone for WrapperShared<P> {
    fn clone(&self) -> Self {
        Self {
            params: Arc::clone(&self.params),
        }
    }
}

impl<P: ClapPlugin> PluginShared<'_> for WrapperShared<P> {}
