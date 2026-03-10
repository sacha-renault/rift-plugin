use vizia::prelude::*;

use rift_plugin_shared::params::{ClapParam, ParamPtr};

use rift_plugin_shared::gui::GuiParamEvent;

/// Signals the start of a user gesture on a [`ClapParam`] (e.g., drag or wheel move began).
pub fn gesture_start(param_ptr: ParamPtr, cx: &mut EventContext) {
    cx.emit(GuiParamEvent::gesture_start(param_ptr.id()));
}

/// Signals the end of a user gesture on a [`ClapParam`].
pub fn gesture_end(param_ptr: ParamPtr, cx: &mut EventContext) {
    cx.emit(GuiParamEvent::gesture_end(param_ptr.id()));
}

/// Emits an event to update a [`ClapParam`] with the provided absolute value (in its original scale).
///
/// # Note
/// The value is not normalized; it must represent the parameter's native scale.
pub fn set_value(param_ptr: ParamPtr, cx: &mut EventContext, value: f64) {
    cx.emit(GuiParamEvent::value(param_ptr.id(), value));
}

/// Emits an event to update a [`ClapParam`] with the provided normalized value (0.0–1.0).
///
/// # Note
/// The `denormalize` method is automatically called internally to convert this to the parameter's native scale.
pub fn set_value_normalized(param_ptr: ParamPtr, cx: &mut EventContext, value: f64) {
    let normalized = param_ptr.denormalize(value);
    cx.emit(GuiParamEvent::value(param_ptr.id(), normalized));
}
