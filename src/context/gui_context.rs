use std::sync::Arc;

use rift_plugin_shared::gui::{GuiContext, GuiParamEvent};

use crate::context::AudioThreadTask;
use crate::prelude::Params;
use crate::wrapper::shared_states::PluginSharedState;

pub struct GuiContextImpl {
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) params: Arc<dyn Params>,
}

impl GuiContext for GuiContextImpl {
    /// Queues a GUI parameter event to be processed by the audio thread.
    ///
    /// The event is wrapped in [`AudioThreadTask`] and pushed to the audio thread's internal task queue.
    /// If the queue is full, an error is logged but the event is dropped.
    fn param_event(&self, event: GuiParamEvent) {
        let task = AudioThreadTask::GuiParamEvent(event);
        if self.states.push_audio_thread_task(task).is_err() {
            log::error!("Couldn't push new param event from gui");
        }
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
}
