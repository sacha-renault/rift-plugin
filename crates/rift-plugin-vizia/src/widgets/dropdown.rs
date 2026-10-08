//! Picks one option out of a list shown in a popup.

use crate::dev_prelude::*;

use super::{index_of, value_of};
use vizia::icons::ICON_CHEVRONS_RIGHT;
use vizia::prelude::*;

/// Internal event; never leaks outside.
enum DropdownEvent {
    Pick(usize),
}

/// A dropdown for choosing one option out of a list.
///
/// It edits one normalized value (`0..=1`) like every other control, so it can
/// be wired to a parameter with [`ParamBinding::connect`](crate::ParamBinding).
///
/// # Notes:
/// Looks like label doesn't fill width by default. I could put the style
/// ```cs
/// dropdown .segment {
///     width: 1s;
/// }
/// ```
/// So it takes full width, but i am not sure what's the best default.
/// I prefer let this like that, anyone can override anyway ...
pub struct Dropdown {
    value: Signal<f32>,
    count: usize,
    is_open: Signal<bool>,
    callbacks: ControlCallbacks,
    placement: Signal<Placement>,
}

impl Dropdown {
    /// A dropdown over `options`.
    ///
    /// # Panics
    /// If `options` is empty.
    pub fn new<S: Into<String>>(
        cx: &mut Context,
        options: impl IntoIterator<Item = S>,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let options: Vec<String> = options.into_iter().map(Into::into).collect();
        assert!(!options.is_empty(), "a Dropdown needs at least one option");

        let count = options.len();
        let is_open = Signal::new(false);
        let placement = Signal::new(Placement::Bottom);

        Self {
            value,
            count,
            is_open,
            placement,
            callbacks: ControlCallbacks::default(),
        }
        .build(cx, |cx| {
            let trigger_options = options.clone();
            let selected = value.map(move |v| trigger_options[index_of(*v, count)].clone());

            HStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    Svg::new(cx, ICON_CHEVRONS_RIGHT).hoverable(false);
                })
                .class("icon");

                Label::new(cx, selected).class("current");
            })
            .class("dropdown-trigger")
            .toggle_class("open", is_open)
            .on_mouse_down(|cx, mb| {
                if matches!(mb, MouseButton::Left) {
                    cx.emit(PopupEvent::Open);
                }
            });

            Binding::new(cx, is_open, move |cx| {
                if is_open.get() {
                    let options = options.clone();
                    Popover::new(cx, |cx| {
                        for (index, option) in options.into_iter().enumerate() {
                            let selected = value.map(move |v| index_of(*v, count) == index);
                            Label::new(cx, option)
                                .class("segment")
                                .toggle_class("selected", selected)
                                .on_press(move |cx| {
                                    cx.emit(DropdownEvent::Pick(index));
                                    cx.emit(PopupEvent::Close);
                                });
                        }
                    })
                    .placement(placement)
                    .show_arrow(false)
                    .on_blur(|cx| cx.emit(PopupEvent::Close));
                }
            });
        })
        .role(Role::ComboBox)
        .navigable(true)
    }

    /// The index of the current option.
    fn current(&self) -> usize {
        index_of(self.value.get_untracked(), self.count)
    }
}

impl Control for Dropdown {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, _normalized: f32) {}
}

impl View for Dropdown {
    fn element(&self) -> Option<&'static str> {
        Some("dropdown")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|popup_event, meta| match popup_event {
            PopupEvent::Open => {
                self.is_open.set(true);
                meta.consume();
            }
            PopupEvent::Close => {
                self.is_open.set(false);
                meta.consume();
            }
            PopupEvent::Switch => {
                self.is_open.set(!self.is_open.get());
                meta.consume();
            }
        });

        event.map(|dropdown_event, meta| match dropdown_event {
            DropdownEvent::Pick(index) if !cx.is_disabled() => {
                let index = (*index).min(self.count - 1);
                if index != self.current() {
                    self.callbacks.single_edit(cx, value_of(index, self.count));
                }
                meta.consume();
            }
            _ => {}
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Right) if !cx.is_disabled() => {
                self.callbacks.context_menu(cx);
                meta.consume();
            }
            _ => {}
        });
    }
}

#[modifiers(for Handle<'_, Dropdown>)]
pub trait DropdownModifiers {
    #[concrete]
    fn placement(self, placement: Placement) -> Self {
        self.modify(|dropdown| dropdown.placement.set(placement))
    }
}
