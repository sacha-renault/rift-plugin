use crate::{gui::GuiParamEvent, prelude::Accumulators};

pub enum MainThreadTask {
    ChangeLatency(u32),
    RequestRestart,
    SetAccumulators(Accumulators),
}

pub enum AudioThreadTask {
    GuiParamEvent(GuiParamEvent),
}
