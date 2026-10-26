//! Contexts handed to the plugin at init/process time, and to menu callbacks.

mod gui_context;
mod init_context;
mod menu_context;
mod process_context;

pub(crate) use gui_context::GuiContextImpl;
pub use init_context::InitContext;
pub use menu_context::MenuContext;
pub use process_context::ProcessContext;
