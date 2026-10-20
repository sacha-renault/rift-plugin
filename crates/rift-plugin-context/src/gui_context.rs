use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use rift_plugin_gui::{GuiContext, GuiParamEvent, GuiParamEventKind, HostSharedHandle};

use crate::PluginSharedState;
use crate::{AudioThreadTask, MainThreadTask, ParamContextMenu};
use rift_plugin_params::{ClapId, ParamCollection};

pub struct GuiContextImpl {
    pub states: Arc<PluginSharedState>,
    pub params: Arc<dyn ParamCollection>,
    pub handle: HostSharedHandle<'static>,
}

impl GuiContext for GuiContextImpl {
    /// Queues a GUI parameter event to be processed by the audio thread.
    ///
    /// The event is wrapped in [`AudioThreadTask`] and pushed to the audio thread's internal task queue.
    /// If the queue is full, an error is logged but the event is dropped.
    fn param_event(&self, event: GuiParamEvent) {
        if let GuiParamEventKind::Value(value) = event.kind {
            self.params.set_value(event.param_id, value);
        }

        let task = AudioThreadTask::GuiParamEvent(event);
        if self.states.post_to_audio(task).is_err() {
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

    fn pop_in_gui(&self) -> Option<rift_plugin_gui::GuiTask> {
        self.states.pop_in_gui()
    }

    fn is_overflow(&self) -> bool {
        self.states.overflows().take_gui_overflow()
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
        let ctx = ParamContextMenu {
            param_id,
            x,
            y,
            screen,
        };
        let task = MainThreadTask::ParamContextMenu(ctx);
        if self.states.post_to_main(task).is_err() {
            log::error!("Couldn't push new param event from gui");
            return;
        }
        // FIX: use the shared handle for the callbacks...
        self.handle.request_callback();
    }
}
