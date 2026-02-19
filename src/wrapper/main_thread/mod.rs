use std::sync::Arc;

use clack_plugin::prelude::*;

use crate::prelude::*;
use crate::wrapper::shared_states::PluginSharedState;
use crate::wrapper::{ClapPlugin, shared::WrapperShared};

mod latency;
mod params;
mod ports;
mod state;
mod wrapper_gui;

pub struct WrapperMainThread<'a, P: ClapPlugin> {
    pub(crate) shared: WrapperShared<P>,
    pub(crate) gui: Box<dyn ClapGui>, // TODO an actual gui thing here
    pub(crate) host: HostMainThreadHandle<'a>,
}

impl<'a, P: ClapPlugin> WrapperMainThread<'a, P> {
    #[inline]
    fn states(&self) -> Arc<PluginSharedState> {
        self.shared.states.clone()
    }
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<'a, P> {
    fn on_main_thread(&mut self) {
        // Case latency has changed
        // if self.messages().take_latency_changed() {
        //     if let Some(ext) = self.host.get_extension::<HostLatency>() {
        //         ext.changed(&mut self.host);
        //     } else {
        //         log::error!("Error when requesting latency change")
        //     }
        // }
        if self.states().take_request_restart() {
            self.host.request_restart();
        }
    }
}
