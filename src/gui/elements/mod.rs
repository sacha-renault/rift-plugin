//! A collection of event that can be use on Vizia GUI.

mod consumer_dispatch;
mod grid;
mod oscilloscope;
mod param_button;
mod param_knob;
mod param_slider;

mod gui_prelude {
    pub use crate::prelude::ClapParam;
    pub use hug_derive::{HandleExtension, ParamViewBuilder};
    pub use vizia::prelude::*;

    pub use super::super::utils::*;
}

pub use consumer_dispatch::{AudioConsumerDispatch, AudioConsumerDispatchExt};
pub use grid::{GridScale, PlotGrid, PlotGridExt};
pub use gui_prelude::RedrawOnExt;
pub use oscilloscope::{Oscilloscope, OscilloscopeExt};
pub use param_button::ParamButton;
pub use param_knob::ParamKnob;
pub use param_slider::ParamSlider;
