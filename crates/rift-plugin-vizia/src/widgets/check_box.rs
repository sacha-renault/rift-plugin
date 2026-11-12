//! A tick box shown as a small LED in a recessed bezel.

use crate::dev_prelude::*;

/// A tick box, optionally with a label.
pub struct CheckBox {
    value: Signal<f32>,
    callbacks: ControlCallbacks,
}

impl CheckBox {
    /// A bare tick box.
    pub fn new(cx: &mut Context, value: Signal<f32>) -> Handle<'_, Self> {
        Self::build_check_box(cx, value, None)
    }

    /// A tick box with a `label` next to it.
    pub fn labeled(
        cx: &mut Context,
        label: impl Res<String> + 'static,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let label = label.to_signal(cx);
        Self::build_check_box(cx, value, Some(label))
    }

    fn build_check_box(
        cx: &mut Context,
        value: Signal<f32>,
        label: Option<Signal<String>>,
    ) -> Handle<'_, Self> {
        let on = value.map(|v| *v >= 0.5);

        Self {
            value,
            callbacks: ControlCallbacks::default(),
        }
        .build(cx, move |cx| {
            ZStack::new(cx, |cx| {
                Element::new(cx).class("check-box-led").hoverable(false);
            })
            .class("check-box-frame")
            .hoverable(false);

            if let Some(label) = label {
                Label::new(cx, label)
                    .class("check-box-label")
                    .hoverable(false);
            }
        })
        .checked(on)
        .role(Role::CheckBox)
        .cursor(CursorIcon::Hand)
        .navigable(true)
    }

    fn flip(&self, cx: &mut EventContext) {
        let next = if self.value.get_untracked() >= 0.5 {
            0.0
        } else {
            1.0
        };
        self.callbacks.single_edit(cx, next);
    }
}

impl Control for CheckBox {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, _normalized: f32) {}
}

impl View for CheckBox {
    fn element(&self) -> Option<&'static str> {
        // Not "checkbox": vizia's default theme already styles that element.
        Some("check-box")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::Press { mouse } => {
                let over = if *mouse {
                    cx.mouse().left.pressed
                } else {
                    cx.focused()
                };
                if over == cx.current() && !cx.is_disabled() {
                    self.flip(cx);
                }
            }

            WindowEvent::MouseDown(MouseButton::Right) if !cx.is_disabled() => {
                self.callbacks.context_menu(cx);
                meta.consume();
            }

            WindowEvent::ActionRequest(action)
                if matches!(action.action, Action::Click) && !cx.is_disabled() =>
            {
                self.flip(cx);
            }

            _ => {}
        });
    }
}
