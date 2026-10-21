use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use rift_plugin_gui::{GuiContext, GuiParamEvent, GuiParamEventKind, HostSharedHandle};

use crate::ParamContextMenu;
use crate::PluginSharedState;
use rift_plugin_params::{ClapId, ParamCollection};

#[derive(bon::Builder)]
pub struct GuiContextImpl {
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

        if self.states.audio_tasks().try_post(event).is_some() {
            log::error!("Couldn't push new param event from gui");
            return;
        }

        self.request_flush();
    }

    fn params(&self) -> Arc<dyn ParamCollection> {
        self.params.clone()
    }

    fn is_playing(&self) -> Arc<AtomicBool> {
        self.states.is_playing.clone()
    }

    fn pop_in_gui(&self) -> Option<rift_plugin_gui::GuiEvent> {
        self.states.gui_tasks().pop()
    }

    fn is_overflow(&self) -> bool {
        self.states.gui_tasks().take_overflow()
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
}

impl GuiContextImpl {
    fn context_menu(&self, param_id: Option<ClapId>, x: i32, y: i32, screen: i32) {
        let task = ParamContextMenu {
            param_id,
            x,
            y,
            screen,
        };
        if self.states.main_tasks().try_post(task).is_some() {
            log::error!("Couldn't push new param event from gui");
            return;
        }
        // FIX: use the shared handle for the callbacks...
        self.handle.request_callback();
    }
}
