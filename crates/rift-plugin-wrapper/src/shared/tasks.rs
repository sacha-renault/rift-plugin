//! Payloads posted from the GUI to the plugin's main thread.

use clack_plugin::utils::ClapId;

/// A GUI request to open the host context menu for a parameter.
///
/// Posted by the GUI to the main thread.
pub(crate) struct ParamContextMenu {
    pub param_id: Option<ClapId>,
    pub x: i32,
    pub y: i32,
    pub screen: i32,
}
