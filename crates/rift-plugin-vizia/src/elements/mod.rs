//! A collection of event that can be use on Vizia GUI.

mod consumer_dispatch;
mod control_points;
mod dropdown;
mod grid;
mod param_button;
mod param_knob;
mod param_slider;
mod param_xy;
mod peak_viewer;
mod plot_xy;
mod popup;

mod gui_prelude {
    //! this is an internal helper that gather
    //! everything needed for ui elements
    pub use rift_plugin_shared::params::ClapParam;

    pub use rift_plugin_derive::{HandleExtension, ParamViewBuilder};
    pub use vizia::prelude::*;

    pub(crate) use crate::utils::gui_events::ContextMenuEvent;
    pub use crate::utils::*;
}

pub use consumer_dispatch::{AudioConsumerDispatch, AudioConsumerDispatchExt};
pub use control_points::ControlPoints;
pub use dropdown::{DropdownItem, DropdownStyled};
pub use grid::{GridScale, PlotGrid, PlotGridExt};
pub use gui_prelude::RedrawOnExt;
pub use param_button::ParamButton;
pub use param_knob::ParamKnob;
pub use param_slider::ParamSlider;
pub use param_xy::ParamPadXY;
pub use peak_viewer::PeaksViewer;
pub use plot_xy::{PlotXY, PlotXYExt};
pub use popup::PopupExt;
