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
    ArrowSelect,
}

/// A selector for an option of a short list (a waveform, a filter type...).
pub struct Selector {
    value: Signal<f32>,
    count: usize,
    callbacks: ControlCallbacks,
    style: Signal<SelectorStyle>,
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
        }
        .build(cx, |cx| {
            style.set_or_bind(cx, move |cx, style| {
                let style = style.get();

                match style {
                    SelectorStyle::Segmented => {
                        for (index, option) in options.iter().enumerate() {
                            let selected = value.map(move |v| index_of(*v, count) == index);
                            Label::new(cx, option.clone())
                                .class("segment")
                                .toggle_class("selected", selected)
                                .on_press(move |cx| cx.emit(SelectorEvent::Pick(index)));
                        }
                    }
                    SelectorStyle::ArrowSelect => {
                        ZStack::new(cx, |cx| {
                            Svg::new(cx, ICON_CHEVRON_LEFT).hoverable(false);
                        })
                        .class("step")
                        .on_press(|cx| cx.emit(SelectorEvent::Step(-1)));

                        let options = options.clone();
                        let current = value.map(move |v| options[index_of(*v, count)].clone());
                        Label::new(cx, current).class("current").hoverable(false);

                        ZStack::new(cx, |cx| {
                            Svg::new(cx, ICON_CHEVRON_RIGHT).hoverable(false);
                        })
                        .class("step")
                        .on_press(|cx| cx.emit(SelectorEvent::Step(1)));
                    }
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

            WindowEvent::MouseScroll(_, y) if *y != 0.0 && !cx.is_disabled() => {
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
}
