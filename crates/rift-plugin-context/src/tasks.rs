//! Exhaustive list of tasks the audio / main thread may have to perform. Those are sent in their respective queues
//! in [`crate::PluginSharedState`].

use clack_plugin::utils::ClapId;
use rift_plugin_gui::GuiParamEvent;

pub struct ParamContextMenu {
    pub param_id: Option<ClapId>,
    pub x: i32,
    pub y: i32,
    pub screen: i32,
}

pub enum MainThreadTask {
    ParamContextMenu(ParamContextMenu),
}

pub enum AudioThreadTask {
    GuiParamEvent(GuiParamEvent),
}
