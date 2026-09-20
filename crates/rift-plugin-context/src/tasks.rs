//! Exaustive lists of task audio / main thread may have to perform. Those are sent in there respective queues
//! in [`crate::wrapper::shared_states::PluginSharedState`].

use clack_plugin::utils::ClapId;
use rift_plugin_core::gui::GuiParamEvent;

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

    /// This means an action has been done. It will not be processed
    /// by audio thread, but since he is the only one to always run
    /// this is useful. Might be good to implement proper callback
    /// so gui doesn't rely on audio thread to request a callback ...
    RequestCallback,
}
