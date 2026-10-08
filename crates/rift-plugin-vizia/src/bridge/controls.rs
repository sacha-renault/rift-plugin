//! Interaction plumbing shared by every value widget.
//!
//! A *control* edits one normalized value (`0.0..=1.0`) and reports edits
//! through three callbacks that mirror a CLAP automation gesture:
//!
//! 1. `on_begin`: the user grabbed the control,
//! 2. `on_change`: the value moved (any number of times),
//! 3. `on_end`: the user let go.
//!
//! Controls know nothing about the host: [`ParamBinding`](crate::ParamBinding)
//! (or your own closures) turn those callbacks into actions.

use vizia::prelude::*;

/// Drag precision multiplier when Shift is held.
pub(crate) const FINE: f32 = 0.1;
/// Range covered by one notch of the mouse wheel.
pub(crate) const WHEEL_STEP: f32 = 0.02;
/// A single wheel event never counts for more notches than this.
pub(crate) const MAX_NOTCHES: f32 = 5.0;

type Action = Box<dyn Fn(&mut EventContext)>;
type ValueAction = Box<dyn Fn(&mut EventContext, f32)>;

/// The callbacks a control reports edits to.
#[derive(Default)]
pub struct ControlCallbacks {
    begin: Option<Action>,
    change: Option<ValueAction>,
    end: Option<Action>,
    context_menu: Option<Action>,
}

impl ControlCallbacks {
    pub(crate) fn begin(&self, cx: &mut EventContext) {
        if let Some(f) = &self.begin {
            f(cx);
        }
    }

    pub(crate) fn change(&self, cx: &mut EventContext, value: f32) {
        if let Some(f) = &self.change {
            f(cx, value);
        }
    }

    pub(crate) fn end(&self, cx: &mut EventContext) {
        if let Some(f) = &self.end {
            f(cx);
        }
    }

    pub(crate) fn context_menu(&self, cx: &mut EventContext) {
        if let Some(f) = &self.context_menu {
            f(cx);
        }
    }

    /// An edit that isn't a drag: wheel, keyboard, click, reset...
    pub(crate) fn single_edit(&self, cx: &mut EventContext, value: f32) {
        self.begin(cx);
        self.change(cx, value);
        self.end(cx);
    }
}

/// Implemented by the widgets that edit a normalized value.
///
/// It unlocks the [`ControlExt`] modifiers on their handle.
pub trait Control: View {
    #[doc(hidden)]
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks;

    #[doc(hidden)]
    fn set_default(&mut self, normalized: f32);
}

/// Modifiers available on every control (dial, fader, toggle, selector...).
pub trait ControlExt: Sized {
    /// Called when the user grabs the control.
    fn on_begin(self, callback: impl Fn(&mut EventContext) + 'static) -> Self;

    /// Called each time the user moves the control, with the new normalized
    /// value in `[0, 1]`.
    fn on_change(self, callback: impl Fn(&mut EventContext, f32) + 'static) -> Self;

    /// Called when the user lets go of the control.
    fn on_end(self, callback: impl Fn(&mut EventContext) + 'static) -> Self;

    /// Called when the user right-clicks the control.
    fn on_context_menu(self, callback: impl Fn(&mut EventContext) + 'static) -> Self;

    /// The normalized value the control jumps to on double-click.
    fn default_normalized(self, normalized: f32) -> Self;
}

impl<V: Control> ControlExt for Handle<'_, V> {
    fn on_begin(self, callback: impl Fn(&mut EventContext) + 'static) -> Self {
        self.modify(|view| view.callbacks_mut().begin = Some(Box::new(callback)))
    }

    fn on_change(self, callback: impl Fn(&mut EventContext, f32) + 'static) -> Self {
        self.modify(|view| view.callbacks_mut().change = Some(Box::new(callback)))
    }

    fn on_end(self, callback: impl Fn(&mut EventContext) + 'static) -> Self {
        self.modify(|view| view.callbacks_mut().end = Some(Box::new(callback)))
    }

    fn on_context_menu(self, callback: impl Fn(&mut EventContext) + 'static) -> Self {
        self.modify(|view| view.callbacks_mut().context_menu = Some(Box::new(callback)))
    }

    fn default_normalized(self, normalized: f32) -> Self {
        self.modify(|view| view.set_default(normalized.clamp(0.0, 1.0)))
    }
}
