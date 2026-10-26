use std::sync::Arc;

use rift_plugin_gui::{
    GuiContext, GuiParamEvent, GuiParamEventKind, HostParamChange, HostSharedHandle,
};
use rift_plugin_params::{ClapId, ParamCollection};

use crate::shared::{ParamContextMenu, PluginSharedState};

#[derive(bon::Builder)]
pub(crate) struct GuiContextImpl {
    states: Arc<PluginSharedState>,
    params: Arc<dyn ParamCollection>,
    handle: HostSharedHandle<'static>,
}

impl GuiContext for GuiContextImpl {
    /// Queues a GUI parameter event for the audio thread.
    fn param_event(&self, event: GuiParamEvent) {
        if let GuiParamEventKind::Value(value) = event.kind {
            self.params.set_value(event.param_id, value);
        }

        if self.states.param_events().try_post(event).is_some() {
            log::error!("Couldn't push new param event from gui");
        }

        self.request_flush();
    }

    fn params(&self) -> Arc<dyn ParamCollection> {
        self.params.clone()
    }

    fn pop_in_gui(&self) -> Option<HostParamChange> {
        self.states.gui_param_changes().pop()
    }

    fn take_refresh_request(&self) -> bool {
        let rescan = self.states.take_param_rescan();
        let overflow = self.states.gui_param_changes().take_overflow();
        rescan || overflow
    }

    fn global_context_menu(&self, x: i32, y: i32, screen: i32) {
        self.context_menu(None, x, y, screen);
    }

    fn param_context_menu(&self, param_id: ClapId, x: i32, y: i32, screen: i32) {
        self.context_menu(Some(param_id), x, y, screen);
    }

    fn handle(&self) -> Option<HostSharedHandle<'static>> {
        Some(self.handle)
    }

    fn latest_transport_info(&self) -> Option<rift_plugin_types::TransportInfo> {
        self.states.latest_transport_info()
    }
}

impl GuiContextImpl {
    fn context_menu(&self, param_id: Option<ClapId>, x: i32, y: i32, screen: i32) {
        let task = ParamContextMenu {
            param_id,
            x,
            y,
            screen,
        };
        if self.states.context_menu_requests().try_post(task).is_some() {
            log::error!("Couldn't push new param event from gui");
            return;
        }
        // FIX: use the shared handle for the callbacks...
        self.handle.request_callback();
    }
}
