//! Picks one option out of a short list.
use crate::dev_prelude::*;

use vizia::icons::{ICON_CHEVRON_LEFT, ICON_CHEVRON_RIGHT};
use vizia::prelude::*;

/// Internal events; never leaks outside
enum SelectorEvent {
    Pick(usize), // Pick by index
    Step(isize), // Pick next / previous
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SelectorStyle {
    #[default]
    Segmented,
    VerticalSegmented,
    ArrowSelect,
    DropDown,
}

/// A selector for an option of a short list (a waveform, a filter type...).
pub struct Selector {
    value: Signal<f32>,
    count: usize,
    callbacks: ControlCallbacks,
    style: Signal<SelectorStyle>,
    scroll_enabled: bool,
}

impl Selector {
    /// A selector over `options`.
    ///
    /// # Panics
    /// If `options` is empty.
    pub fn new<S: Into<String>>(
        cx: &mut Context,
        options: impl IntoIterator<Item = S>,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let options: Vec<String> = options.into_iter().map(Into::into).collect();
        assert!(!options.is_empty(), "a Selector needs at least one option");

        let count = options.len();
        let style = Signal::new(SelectorStyle::default());

        Self {
            value,
            count,
            callbacks: ControlCallbacks::default(),
            style,
            scroll_enabled: true,
        }
        .build(cx, |cx| {
            style.set_or_bind(cx, move |cx, style| {
                let style = style.get();

                let options = options.clone();
                match style {
                    SelectorStyle::Segmented | SelectorStyle::VerticalSegmented => {
                        segmented(cx, value, options, false)
                    }
                    SelectorStyle::ArrowSelect => arrow_selected(cx, value, options),
                    SelectorStyle::DropDown => drop_down(cx, value, options),
                }
            });
        })
        .role(Role::RadioGroup)
        .navigable(true)
    }
}

/// The option a normalized value stands for.
fn index_of(value: f32, count: usize) -> usize {
    ((value.clamp(0.0, 1.0) * (count - 1) as f32).round() as usize).min(count - 1)
}

/// The normalized value of option `index`.
fn value_of(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.0
    } else {
        index as f32 / (count - 1) as f32
    }
}

fn segmented(cx: &mut Context, value: Signal<f32>, options: Vec<String>, is_popover: bool) {
    let count = options.len();

    for (index, option) in options.into_iter().enumerate() {
        let selected = value.map(move |v| index_of(*v, count) == index);
        Label::new(cx, option)
            .class("segment")
            .toggle_class("selected", selected)
            .on_press(move |cx| {
                cx.emit(SelectorEvent::Pick(index));
                if is_popover {
                    cx.emit(PopupEvent::Close);
                }
            });
    }
}

fn arrow_selected(cx: &mut Context, value: Signal<f32>, options: Vec<String>) {
    let count = options.len();
    ZStack::new(cx, |cx| {
        Svg::new(cx, ICON_CHEVRON_LEFT).hoverable(false);
    })
    .class("step")
    .disabled(value.map(move |v| index_of(*v, count) == 0))
    .on_press(|cx| cx.emit(SelectorEvent::Step(-1)));

    let current = value.map(move |v| options[index_of(*v, count)].clone());
    Label::new(cx, current).class("current").hoverable(false);

    ZStack::new(cx, |cx| {
        Svg::new(cx, ICON_CHEVRON_RIGHT).hoverable(false);
    })
    .class("step")
    .disabled(value.map(move |v| index_of(*v, count) == count - 1))
    .on_press(|cx| cx.emit(SelectorEvent::Step(1)));
}

fn drop_down(cx: &mut Context, value: Signal<f32>, options: Vec<String>) {
    let count = options.len();
    let options_for_dropdown = options.clone();
    let selected_text = value.map(move |v| options[index_of(*v, count)].clone());

    Dropdown::new(
        cx,
        move |cx| {
            Label::new(cx, selected_text)
                .on_mouse_down(|cx, mb| {
                    if matches!(mb, MouseButton::Left) {
                        cx.emit(PopupEvent::Open);
                    }
                })
                .class("dropdown-trigger");
        },
        move |cx| {
            segmented(cx, value, options_for_dropdown.clone(), true);
        },
    );
}

impl Control for Selector {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, _normalized: f32) {}
}

impl Selector {
    fn current(&self) -> usize {
        index_of(self.value.get_untracked(), self.count)
    }

    /// Moves to the option `by` places away, if there is one.
    fn step(&self, cx: &mut EventContext, by: isize) {
        let target = (self.current() as isize + by).clamp(0, self.count as isize - 1) as usize;
        if target != self.current() {
            self.callbacks.single_edit(cx, value_of(target, self.count));
        }
    }
}

impl View for Selector {
    fn element(&self) -> Option<&'static str> {
        Some("selector")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|selector_event, meta| match selector_event {
            SelectorEvent::Pick(index) if !cx.is_disabled() => {
                let index = (*index).min(self.count - 1);
                if index != self.current() {
                    self.callbacks.single_edit(cx, value_of(index, self.count));
                }
                meta.consume();
            }
            SelectorEvent::Step(by) if !cx.is_disabled() => {
                self.step(cx, *by);
                meta.consume();
            }
            _ => {}
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Right) if !cx.is_disabled() => {
                self.callbacks.context_menu(cx);
                meta.consume();
            }

            WindowEvent::MouseScroll(_, y)
                if self.scroll_enabled && *y != 0.0 && !cx.is_disabled() =>
            {
                self.step(cx, if *y > 0.0 { -1 } else { 1 });
                meta.consume();
            }

            WindowEvent::KeyDown(Code::ArrowLeft | Code::ArrowUp, _) if !cx.is_disabled() => {
                self.step(cx, -1);
            }

            WindowEvent::KeyDown(Code::ArrowRight | Code::ArrowDown, _) if !cx.is_disabled() => {
                self.step(cx, 1);
            }

            _ => {}
        });
    }
}

#[modifiers(for Handle<'_, Selector>)]
pub trait SelectorModifiers {
    #[concrete]
    fn arrow_select(self) -> Self {
        self.class("stepper")
            .modify(|selector| selector.style.set(SelectorStyle::ArrowSelect))
    }

    #[concrete]
    fn vertical(self) -> Self {
        self.class("vertical")
            .modify(|selector| selector.style.set(SelectorStyle::VerticalSegmented))
    }

    #[concrete]
    fn dropdown(self) -> Self {
        self.class("vertical")
            .modify(|selector| selector.style.set(SelectorStyle::DropDown))
    }

    #[concrete]
    /// If this function is called, this will prevent to change the value on scroll
    fn disable_scroll(self) -> Self {
        self.modify(|selector| selector.scroll_enabled = false)
    }
}
