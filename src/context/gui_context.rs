use std::sync::Arc;

use crate::context::AudioThreadTask;
use crate::gui::GuiParamEvent;
use crate::prelude::Params;
use crate::wrapper::shared_states::PluginSharedState;

pub struct GuiContext {
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) params: Arc<dyn Params>,
}

impl GuiContext {
    /// Queues a GUI parameter event to be processed by the audio thread.
    ///
    /// The event is wrapped in [`AudioThreadTask`] and pushed to the audio thread's internal task queue.
    /// If the queue is full, an error is logged but the event is dropped.
    pub fn param_event(&self, event: GuiParamEvent) {
        let task = AudioThreadTask::GuiParamEvent(event);
        if self.states.push_audio_thread_task(task).is_err() {
            log::error!("Couldn't push new param event from gui");
        }
    }
}
