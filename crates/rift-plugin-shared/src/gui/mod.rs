//! Simply define the GUI trait here so it can be used by any crate

mod events;
mod gui_traits;

pub use events::{GuiParamEvent, GuiParamEventKind};
pub use gui_traits::{ClapGui, GuiContext, GuiFactory};
