use crate::dev_prelude::*;

/// Pixels of vertical travel needed to sweep the whole range.
const DRAG_RANGE: f32 = 220.0;

/// Alias to `Knob`, but with text.
pub struct DialModifiers {
    hot: Signal<bool>,
    drag: Option<Drag>,
    value: Signal<f32>,
    centered: Signal<bool>,
    default: f32,
    callbacks: ControlCallbacks,
}

impl View for DialModifiers {
    fn element(&self) -> Option<&'static str> {
        Some("dial")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseOver | WindowEvent::MouseOut | WindowEvent::MouseLeave => {
                self.update_hot(cx);
            }

            WindowEvent::MouseDown(MouseButton::Left) if !cx.is_disabled() => {
                if cx.modifiers().ctrl() || cx.modifiers().logo() {
                    self.callbacks.single_edit(cx, self.default);
                } else {
                    self.drag = Some(Drag {
                        last_y: cx.mouse().cursor_y,
                        value: self.value.get_untracked(),
                    });
                    cx.capture();
                    cx.focus_with_visibility(false);
                    cx.toggle_class("dragging", true);
                    self.update_hot(cx);
                    self.callbacks.begin(cx);
                }
                meta.consume();
            }

            WindowEvent::MouseMove(_, y) => self.drag_to(cx, *y),

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag.take().is_some() {
                    cx.release();
                    cx.toggle_class("dragging", false);
                    self.update_hot(cx);
                    self.callbacks.end(cx);
                }
            }

            WindowEvent::MouseDown(MouseButton::Right) if !cx.is_disabled() => {
                self.callbacks.context_menu(cx);
                meta.consume();
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left) if !cx.is_disabled() => {
                match self.drag.as_mut() {
                    // The double-click lands in the middle of a new drag:
                    // keep the gesture, restart it from the default.
                    Some(drag) => {
                        drag.value = self.default;
                        self.callbacks.change(cx, self.default);
                    }
                    None => self.callbacks.single_edit(cx, self.default),
                }
                meta.consume();
            }

            _ => {}
        });
    }
}

impl DialModifiers {
    pub fn labeled(
        cx: &mut Context,
        label: impl Res<String> + 'static,
        readout: impl Res<String> + 'static,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let label = label.to_signal(cx);
        let readout = readout.to_signal(cx);
        Self::build_dial(cx, value, Some((label, readout)), false)
    }

    fn build_dial(
        cx: &mut Context,
        value: Signal<f32>,
        texts: Option<(Signal<String>, Signal<String>)>,
        centered: bool,
    ) -> Handle<'_, Self> {
        let hot = Signal::new(false);
        let centered = Signal::new(false);

        Self {
            hot,
            drag: None,
            value,
            default: value.get_untracked(),
            centered,
            callbacks: ControlCallbacks::default(),
        }
        .build(cx, |cx| {
            ZStack::new(cx, move |cx| {
                ArcTrack::new(
                    cx,
                    centered.get(),
                    Percentage(100.0),
                    Percentage(15.0),
                    -240.,
                    60.,
                    KnobMode::Continuous,
                )
                .value(value)
                .class("dial-track");

                HStack::new(cx, |cx| {
                    Element::new(cx).class("dial-tick");
                })
                .bind(value, move |handle| {
                    let value = value.get();
                    handle.rotate(Angle::Deg(value * 300.0 - 150.0));
                })
                .class("dial-head");
            })
            .class("dial-face");

            if let Some((label, readout)) = texts {
                let text = Memo::new(move |_| {
                    if hot.get() {
                        readout.get()
                    } else {
                        label.get()
                    }
                });
                Label::new(cx, text).class("dial-label").hoverable(false);
            }
        })
    }

    fn update_hot(&self, cx: &EventContext) {
        self.hot.set(self.drag.is_some() || cx.is_over());
    }

    fn drag_to(&mut self, cx: &mut EventContext, y: f32) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };

        // Moving up (smaller y) increases the value.
        let delta = (drag.last_y - y) / (DRAG_RANGE * cx.scale_factor());
        let precision = if cx.modifiers().shift() { FINE } else { 1.0 };
        drag.last_y = y;
        drag.value = (drag.value + delta * precision).clamp(0.0, 1.0);

        let value = drag.value;
        self.callbacks.change(cx, value);
    }
}

impl Control for DialModifiers {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, normalized: f32) {
        self.default = normalized
    }
}

#[modifiers(for Handle<'_, DialModifiers>)]
pub trait DialExt {
    #[concrete]
    fn centered(self) -> Self {
        self.modify(|dial| dial.centered.set(true))
    }
}
