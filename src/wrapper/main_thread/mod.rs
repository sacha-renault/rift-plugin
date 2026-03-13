use std::sync::Arc;

use clack_extensions::context_menu::{ContextMenuTarget, HostContextMenu};
use clack_extensions::latency::HostLatency;
use clack_plugin::prelude::*;

use rift_plugin_shared::gui::ClapGui;

use crate::context::MainThreadTask;
use crate::prelude::*;
use crate::wrapper::shared_states::PluginSharedState;
use crate::wrapper::{ClapPlugin, shared::WrapperShared};

mod context_menu;
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

    fn notify_latency_changed(&mut self) {
        if let Some(ext) = self.host.get_extension::<HostLatency>() {
            ext.changed(&mut self.host);
        } else {
            log::error!("Error when requesting latency change")
        }
    }

    fn open_context_menu(&mut self, param_ctx_menu: ParamContextMenu) {
        if let Some(ext) = self.host.get_extension::<HostContextMenu>() {
            let ParamContextMenu {
                param_id,
                x,
                y,
                screen,
            } = param_ctx_menu;

            if !ext.can_popup(&mut self.host) {
                log::error!("Couldn't open popup");
                return;
            }

            let target = ContextMenuTarget::Param(param_id);
            if let Err(err) = ext.popup(&mut self.host, target, screen, x, y) {
                log::error!("Couldn't open popup {err}")
            }
        } else {
            log::error!("Error when requesting popup open")
        }
    }
}

impl<'a, P: ClapPlugin> PluginMainThread<'a, WrapperShared<P>> for WrapperMainThread<'a, P> {
    fn on_main_thread(&mut self) {
        let states = self.states();
        while let Some(task) = states.pop_main_thread_tasks() {
            use MainThreadTask::*;

            match task {
                ChangeLatency(new_latency) => {
                    states.set_latency(new_latency);
                    self.notify_latency_changed();
                }
                RequestRestart => self.host.request_restart(),
                ParamContextMenu(ctx) => self.open_context_menu(ctx),
            }
        }
    }
}
