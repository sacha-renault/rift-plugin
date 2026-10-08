mod data;
mod factory;
mod gui;

pub mod widgets;

pub(crate) mod dev_prelude {
    pub use super::data::*;
    pub use super::prelude::*;
}

pub mod prelude {
    pub use super::factory::{ViziaFactory, vizia_gui};
    pub use rift_plugin_gui::GuiFactory;
    pub use vizia::prelude::*;
}
