use std::sync::Arc;

use clack_extensions::latency::HostLatency;
use clack_plugin::prelude::*;

use rift_plugin_gui::ClapGui;

use crate::{ClapPlugin, shared::WrapperShared};
use rift_plugin_context::{MainThreadTask, PluginSharedState};

pub(crate) mod context_menu;
mod latency;
mod midi;
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

    fn notify_latency_changed(&mut self, latency: u32) {
        self.states().set_latency(latency);
        if let Some(ext) = self.host.get_extension::<HostLatency>() {
            ext.changed(&mut self.host);
        } else {
            log::error!("Error when requesting latency change")
        }
    }
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<'a, P> {
    fn on_main_thread(&mut self) {
        let states = self.states();
        while let Some(task) = states.pop_in_main() {
            use MainThreadTask::*;

            match task {
                ChangeLatency(new_latency) => self.notify_latency_changed(new_latency),
                RequestRestart => self.host.request_restart(),
                ParamContextMenu(ctx) => self.open_param_menu(ctx),
            }
        }
    }
}
