//! An on/off switch.

use crate::dev_prelude::*;

/// An on/off switch, optionally with a label.
pub struct Toggle {
    value: Signal<f32>,
    callbacks: ControlCallbacks,
}

impl Toggle {
    /// A bare switch.
    pub fn new(cx: &mut Context, value: Signal<f32>) -> Handle<'_, Self> {
        Self::build_toggle(cx, value, None)
    }

    /// A switch with a `label` next to it.
    pub fn labeled(
        cx: &mut Context,
        label: impl Res<String> + 'static,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let label = label.to_signal(cx);
        Self::build_toggle(cx, value, Some(label))
    }

    fn build_toggle(
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
            HStack::new(cx, |cx| {
                Element::new(cx).class("toggle-thumb").hoverable(false);
            })
            .class("toggle-track")
            .hoverable(false);

            if let Some(label) = label {
                Label::new(cx, label).class("toggle-label").hoverable(false);
            }
        })
        .checked(on)
        .role(Role::Switch)
        .navigable(true)
    }
}

impl Control for Toggle {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, _normalized: f32) {}
}

impl Toggle {
    fn flip(&self, cx: &mut EventContext) {
        let next = if self.value.get_untracked() >= 0.5 {
            0.0
        } else {
            1.0
        };
        self.callbacks.single_edit(cx, next);
    }
}

impl View for Toggle {
    fn element(&self) -> Option<&'static str> {
        Some("toggle")
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
