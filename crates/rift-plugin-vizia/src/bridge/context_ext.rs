use rift_plugin_params::{PersistValue, PersistentParam, SwapParam};

use crate::dev_prelude::*;

#[modifiers(for T where T: EmitContext)]
pub trait ContextExt {
    #[concrete]
    fn set_swap_param<V: PersistValue>(&mut self, param: &SwapParam<V>, value: impl Into<V>) {
        self.emit_to(Entity::root(), rift_plugin_gui::set_swap(param, value));
    }

    /// Publishes a value to a persistent parameter, then tells the audio thread it
    /// changed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// cx.set_persistent(&params.sample, |p| p.set_value(path));
    /// ```
    #[concrete]
    fn set_persistent<P: PersistentParam, F: FnOnce(&P)>(&mut self, p: &P, set: F) {
        self.emit_to(Entity::root(), rift_plugin_gui::set_persistent(p, set));
    }

    /// Tells the audio thread that a persistent parameter changed, without touching
    /// its value.
    ///
    /// For the cases where the value is already published, e.g. after loading state.
    /// The write must have happened on this thread before this call, otherwise the
    /// audio thread can re-read the old value; prefer [`Self::set_persistent`] when
    /// you are the one changing it.
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
