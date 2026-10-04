/// How param updates reach the GUI.
///
/// Retained GUIs need the audio thread to push every change (they hold widget
/// state that would otherwise go stale). Immediate-mode GUIs re-read the params
/// each frame and don't want the pushes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamSync {
    /// Audio thread pushes changes into the GUI queue.
    Push,
    /// GUI pulls param values itself; nothing is pushed.
    Pull,
}
