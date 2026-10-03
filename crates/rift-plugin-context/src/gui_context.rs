use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use rift_plugin_gui::{GuiContext, GuiParamEvent};

use crate::PluginSharedState;
use crate::{AudioThreadTask, MainThreadTask, ParamContextMenu};
use rift_plugin_params::ParamCollection;

pub struct GuiContextImpl {
    pub states: Arc<PluginSharedState>,
    pub params: Arc<dyn ParamCollection>,
}

impl GuiContext for GuiContextImpl {
    /// Queues a GUI parameter event to be processed by the audio thread.
    ///
    /// The event is wrapped in [`AudioThreadTask`] and pushed to the audio thread's internal task queue.
    /// If the queue is full, an error is logged but the event is dropped.
    fn param_event(&self, event: GuiParamEvent) {
        let task = AudioThreadTask::GuiParamEvent(event);
        if self.states.post_to_audio(task).is_err() {
            log::error!("Couldn't push new param event from gui");
        }
    }

    fn params(&self) -> Arc<dyn ParamCollection> {
        self.params.clone()
    }

    fn param_context_menu(
        &self,
        param_id: clack_plugin::prelude::ClapId,
        x: i32,
        y: i32,
        screen: i32,
    ) {
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
        if self
            .states
            .post_to_audio(AudioThreadTask::RequestCallback)
            .is_err()
        {
            log::error!("Couldn't push callback request");
        }
    }

    fn is_playing(&self) -> Arc<AtomicBool> {
        self.states.is_playing.clone()
    }

    fn pop_in_gui(&self) -> Option<rift_plugin_gui::GuiTasks> {
        self.states.pop_in_gui()
    }
}
