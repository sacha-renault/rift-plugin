//! Exaustive lists of task audio / main thread may have to perform. Those are sent in there respective queues
//! in [`crate::wrapper::shared_states::PluginSharedState`].

use clack_plugin::utils::ClapId;
use rift_plugin_shared::gui::GuiParamEvent;

pub struct ParamContextMenu {
    pub param_id: ClapId,
    pub x: i32,
    pub y: i32,
    pub screen: i32,
}

pub enum MainThreadTask {
    ChangeLatency(u32),
    RequestRestart,
    ParamContextMenu(ParamContextMenu),
}

pub enum AudioThreadTask {
    GuiParamEvent(GuiParamEvent),
}
