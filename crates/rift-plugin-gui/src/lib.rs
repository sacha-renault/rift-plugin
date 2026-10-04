//! Simply define the GUI trait here so it can be used by any crate

mod events;
mod gui_traits;
mod gui_type;

pub use events::{GuiParamEvent, GuiParamEventKind, GuiTasks};
pub use gui_traits::{ClapGui, GuiContext, GuiFactory};

// Reexport some clack stuff
pub use clack_extensions::gui::{GuiConfiguration, GuiSize, Window};
pub use clack_plugin::plugin::PluginError;
pub use gui_type::GuiType;
