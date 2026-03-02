mod oscilloscope;
mod param_button;
mod param_knob;

pub mod param_binding;

mod gui_prelude {
    pub use super::param_binding::*;
    pub use crate::prelude::ClapParam;
    pub use hug_derive::ParamViewBuilder;
    pub use vizia::prelude::*;

    pub use super::super::utils::*;
}

pub use oscilloscope::Oscilloscope;
pub use param_button::ParamButton;
pub use param_knob::ParamKnob;
