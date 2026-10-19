use crate::dev_prelude::*;

#[modifiers(for EventContext<'_>)]
pub trait ContextExt {
    /// Notify the audio thread that a persitent param has changed.
    #[concrete]
    fn notify_persistent_changed(&mut self, id: ClapId) {
        self.emit_to(Entity::root(), GuiParamEvent::persistent(id));
    }
}
