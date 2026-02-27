use crate::gui::GuiParamEvent;

pub enum MainThreadTask {
    ChangeLatency(u32),
    RequestRestart,
}

pub enum AudioThreadTask {
    GuiParamEvent(GuiParamEvent),
}
