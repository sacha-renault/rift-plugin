mod bridge;
mod factory;
mod gui;
mod theme;
mod utils;

pub mod analysers;
pub mod widgets;

pub(crate) mod dev_prelude {
    pub(crate) use super::utils::*;

    pub use super::bridge::*;
    pub use super::prelude::*;

    pub use rift_plugin_gui::{GuiContext, GuiParamEvent};
    pub use rift_plugin_params::{ClapId, Param, ParamPtr};

    pub use vizia::prelude::*;
}

pub mod prelude {
    pub use super::bridge::register_param;
    pub use super::factory::{ViziaFactory, vizia_gui};

    pub use rift_plugin_derive::modifiers;
    pub use rift_plugin_gui::GuiFactory;
    pub use vizia::prelude::*;
}
