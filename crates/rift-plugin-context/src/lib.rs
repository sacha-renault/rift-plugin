//! Contexts handed to the plugin at init/process time, plus the internal
//! cross-thread task queues and shared state they rely on.

mod gui_context;
mod init_context;
mod menu_context;
mod process_context;
mod shared_states;
mod tasks;

pub use gui_context::GuiContextImpl;
pub use init_context::InitContext;
pub use menu_context::MenuContext;
pub use process_context::ProcessContext;
pub use shared_states::PluginSharedState;

pub use tasks::ParamContextMenu;
pub use tasks::{AudioThreadTask, MainThreadTask};
