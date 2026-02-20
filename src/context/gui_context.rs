use std::sync::Arc;

use crate::context::AudioThreadTask;
use crate::gui::GuiParamEvent;
use crate::wrapper::shared_states::PluginSharedState;

pub struct GuiContext {
    pub(crate) states: Arc<PluginSharedState>,
}

impl GuiContext {
    pub fn param_event(&self, event: GuiParamEvent) {
        let task = AudioThreadTask::GuiParamEvent(event);
        if self.states.push_audio_thread_task(task).is_err() {
            log::error!("Couldn't push new param event from gui");
        }
    }
}
