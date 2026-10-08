use rift_plugin_params::ClapId;

/// A request to open the host's context menu (automation, MIDI learn, ...) on
/// a parameter. Handled by the GUI, which forwards it to the host.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParamContextMenuRequest {
    pub id: ClapId,
    pub x: i32,
    pub y: i32,
    pub screen: i32,
}
