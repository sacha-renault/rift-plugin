use crate::dev_prelude::*;

#[modifiers(for T where T: EmitContext)]
pub trait ContextExt {
    /// Notify the audio thread that a persitent param has changed.
    #[concrete]
    fn notify_persistent_changed(&mut self, id: ClapId) {
        self.emit_to(Entity::root(), GuiParamEvent::persistent(id));
    }

    #[concrete]
    fn notify_param_changed(&mut self, id: ClapId, plain: f32) {
        self.emit_to(Entity::root(), GuiParamEvent::value(id, plain));
    }

    #[concrete]
    fn notify_param_gesture_start(&mut self, id: ClapId) {
        self.emit_to(Entity::root(), GuiParamEvent::gesture_start(id));
    }

    #[concrete]
    fn notify_param_gesture_end(&mut self, id: ClapId) {
        self.emit_to(Entity::root(), GuiParamEvent::gesture_end(id));
    }
}
