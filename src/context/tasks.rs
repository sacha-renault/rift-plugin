//! Exaustive lists of task audio / main thread may have to perform. Those are sent in there respective queues
//! in [`crate::wrapper::shared_states::PluginSharedState`].

use rift_plugin_shared::gui::GuiParamEvent;

pub enum MainThreadTask {
    ChangeLatency(u32),
    RequestRestart,
}

pub enum AudioThreadTask {
    GuiParamEvent(GuiParamEvent),
}
