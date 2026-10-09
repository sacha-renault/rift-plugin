use vizia::vg::Point;

use crate::dev_prelude::*;

const THUMB_RADIUS: f32 = 8.0;
const LINE_THICKNESS: f32 = 4.0;

/// A slider for a normalized value, horizontal or vertical.
pub struct Fader {
    value: Signal<f32>,
    default: f32,
    drag: Option<Drag>,
    thumb_scaling: Signal<f32>,
    line_scaling: Signal<f32>,
    vertical: Signal<bool>,
    callbacks: ControlCallbacks,
}

impl Fader {
    /// A bare track, without text.
    pub fn new(cx: &mut Context, value: Signal<f32>) -> Handle<'_, Self> {
        Self::build_fader(cx, value, None)
    }

    /// A fader with its `label` on top left and its `readout` (the current
    /// value as text) on top right.
    pub fn labeled(
        cx: &mut Context,
        label: impl Res<String> + 'static,
        readout: impl Res<String> + 'static,
        value: Signal<f32>,
    ) -> Handle<'_, Self> {
        let label = label.to_signal(cx);
        let readout = readout.to_signal(cx);
        Self::build_fader(cx, value, Some((label, readout)))
    }

    fn build_fader(
        cx: &mut Context,
        value: Signal<f32>,
        texts: Option<(Signal<String>, Signal<String>)>,
    ) -> Handle<'_, Self> {
        let line_scaling = Signal::new(1.);
        let thumb_scaling = Signal::new(1.);
        let vertical = Signal::new(true);

        Self {
            value,
            default: value.get_untracked(),
            drag: None,
            line_scaling,
            thumb_scaling,
            vertical,
            callbacks: ControlCallbacks::default(),
        }
        .build(cx, |cx| {
            if let Some((label, readout)) = texts {
                VStack::new(cx, |cx| {
                    Label::new(cx, label).class("fader-label").hoverable(false);
                    Label::new(cx, readout)
                        .class("fader-readout")
                        .hoverable(false);
                })
                .class("fader-header")
                .hoverable(false);
            }

            FaderTrack::new(cx, value, thumb_scaling, line_scaling, vertical);
        })
        .role(Role::Slider)
        .numeric_value(value.map(|v| (*v as f64 * 100.0).round()))
        .navigable(true)
    }

    fn axis(&self, cx: &EventContext) -> Axis {
        let bounds = if let Some(&track) = cx.get_entities_by_class("track").first() {
            cx.transformed_bounds(track)
        } else {
            debug_assert!(false, "fader has no .track child");
            cx.bounds() // Fallback to self in release build ...
        };

        Axis::new(
            bounds,
            THUMB_RADIUS * cx.scale_factor(),
            self.vertical.get_untracked(),
        )
    }

    fn drag_to(&mut self, cx: &mut EventContext, x: f32, y: f32) {
        let Some(mut drag) = self.drag else {
            return;
        };
        let axis = self.axis(cx);

        if cx.modifiers().shift() {
            let along = axis.along(x, y);
            drag.value += (along - drag.last_y) / axis.length().max(1.0) * FINE;
            drag.last_y = along;
        } else {
            drag.value = axis.value_at(x, y);
        }
        drag.value = drag.value.clamp(0.0, 1.0);

        self.drag = Some(drag);
        self.callbacks.change(cx, drag.value);
    }

    fn step(&self, cx: &mut EventContext, amount: f32) {
        let fine = if cx.modifiers().shift() { FINE } else { 1.0 };
        let value = (self.value.get_untracked() + amount * fine).clamp(0.0, 1.0);
        self.callbacks.single_edit(cx, value);
    }
}

#[modifiers(for Handle<'_, Fader>)]
pub trait FaderModifers {
    #[concrete]
    /// Make the thumb bigger
    fn thumb_scaling(self, scale: f32) -> Self {
        self.modify(|fader| fader.thumb_scaling.set(scale.max(0.01)))
    }

    #[concrete]
    /// Make the line thicker
    fn line_scaling(self, scale: f32) -> Self {
        self.modify(|fader| fader.line_scaling.set(scale.max(0.01)))
    }

    #[concrete]
    /// Slider becomes horizontal
    fn horizontal(self) -> Self {
        self.class("horizontal")
            .modify(|fader| fader.vertical.set(false))
    }
}

impl Control for Fader {
    fn callbacks_mut(&mut self) -> &mut ControlCallbacks {
        &mut self.callbacks
    }

    fn set_default(&mut self, normalized: f32) {
        self.default = normalized;
    }
}

impl View for Fader {
    fn element(&self) -> Option<&'static str> {
        Some("fader")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) if !cx.is_disabled() => {
                if cx.modifiers().ctrl() || cx.modifiers().logo() {
                    self.callbacks.single_edit(cx, self.default);
                } else {
                    let (x, y) = (cx.mouse().cursor_x, cx.mouse().cursor_y);
                    let axis = self.axis(cx);

                    self.drag = Some(Drag {
                        last_y: axis.along(x, y),
                        value: self.value.get_untracked(),
                    });
                    cx.capture();
                    cx.focus_with_visibility(false);
                    cx.toggle_class("dragging", true);
                    self.callbacks.begin(cx);
                    // A plain click jumps to the cursor.
                    self.drag_to(cx, x, y);
                }
                meta.consume();
            }

            WindowEvent::MouseMove(x, y) => self.drag_to(cx, *x, *y),

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag.take().is_some() {
                    cx.release();
                    cx.toggle_class("dragging", false);
                    self.callbacks.end(cx);
                }
            }

            WindowEvent::MouseDown(MouseButton::Right) if !cx.is_disabled() => {
                self.callbacks.context_menu(cx);
                meta.consume();
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left) if !cx.is_disabled() => {
                match self.drag.as_mut() {
                    Some(drag) => {
                        drag.value = self.default;
                        self.callbacks.change(cx, self.default);
                    }
                    None => self.callbacks.single_edit(cx, self.default),
                }
                meta.consume();
            }

            WindowEvent::MouseScroll(_, y) if *y != 0.0 && !cx.is_disabled() => {
                self.step(cx, y.clamp(-MAX_NOTCHES, MAX_NOTCHES) * WHEEL_STEP);
                meta.consume();
            }

            WindowEvent::KeyDown(Code::Home, _) if !cx.is_disabled() => {
                self.callbacks.single_edit(cx, 0.0);
            }

            WindowEvent::KeyDown(Code::End, _) if !cx.is_disabled() => {
                self.callbacks.single_edit(cx, 1.0);
            }

            _ => {}
        });
    }
}

struct FaderTrack {
    value: Signal<f32>,
    thumb_scaling: Signal<f32>,
    line_scaling: Signal<f32>,
    vertical: Signal<bool>,
}

impl FaderTrack {
    fn new(
        cx: &mut Context,
        value: Signal<f32>,
        thumb_scaling: Signal<f32>,
        line_scaling: Signal<f32>,
        vertical: Signal<bool>,
    ) -> Handle<'_, Self> {
        Self {
            value,
            thumb_scaling,
            line_scaling,
            vertical,
        }
        .build(cx, |_| {})
        .class("track")
        .bind(value, |mut handle| handle.needs_redraw())
        .hoverable(false)
    }
}

impl View for FaderTrack {
    fn element(&self) -> Option<&'static str> {
        Some("fader-track")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        if bounds.w < 1.0 || bounds.h < 1.0 {
            return;
        }

        let slots = Slots::read(cx);
        let scale = cx.scale_factor();
        let thickness = LINE_THICKNESS * scale * self.line_scaling.get_untracked();
        let radius = THUMB_RADIUS * scale * self.thumb_scaling.get_untracked();
        let axis = Axis::new(bounds, radius, self.vertical.get_untracked());

        // Behind the track (uncovered values)
        let mut groove = stroke_paint(thickness);
        groove.set_color(slots.track);
        canvas.draw_line(axis.start, axis.end, &groove);

        // Where the fill starts, marked by a tick unless it is the start.
        let value = self.value.get_untracked().clamp(0.0, 1.0);
        let (from, to) = (axis.at(0.), axis.at(value));

        if slots.glow.a() > 0 {
            let mut glow = stroke_paint(thickness * 2.2);
            glow.set_color(slots.glow);
            canvas.draw_line(from, to, &blurred(glow, thickness));
        }

        let mut fill = stroke_paint(thickness);
        match linear(axis.start, axis.end, slots.accent, slots.accent_end) {
            Some(shader) => {
                fill.set_shader(shader);
            }
            None => {
                fill.set_color(slots.accent);
            }
        }
        canvas.draw_line(from, to, &fill);

        // Thumb: a soft shadow, the body and a small accent dot.
        let shadow = blurred(fill_paint(Color::rgba(0, 0, 0, 170)), radius * 0.35);
        canvas.draw_circle(
            Point::new(to.x, to.y + radius * 0.25),
            radius * 0.95,
            &shadow,
        );

        canvas.draw_circle(to, radius, &fill_paint(slots.rim));

        let dot = mix(slots.accent, slots.accent_end, value);
        canvas.draw_circle(to, radius * 0.38, &fill_paint(dot));
    }
}

/// Helper to get value on the fader
struct Axis {
    start: Point,
    end: Point,
    vertical: bool,
}

impl Axis {
    fn new(bounds: BoundingBox, inset: f32, vertical: bool) -> Self {
        let cx = bounds.x + bounds.w / 2.0;
        let cy = bounds.y + bounds.h / 2.0;

        if vertical {
            Self {
                start: Point::new(cx, bounds.bottom() - inset),
                end: Point::new(cx, bounds.top() + inset),
                vertical: true,
            }
        } else {
            Self {
                start: Point::new(bounds.left() + inset, cy),
                end: Point::new(bounds.right() - inset, cy),
                vertical: false,
            }
        }
    }

    fn at(&self, value: f32) -> Point {
        Point::new(
            self.start.x + (self.end.x - self.start.x) * value,
            self.start.y + (self.end.y - self.start.y) * value,
        )
    }

    fn length(&self) -> f32 {
        (self.end.x - self.start.x).abs() + (self.end.y - self.start.y).abs()
    }

    fn value_at(&self, x: f32, y: f32) -> f32 {
        let length = self.length().max(1.0);
        if self.vertical {
            (self.start.y - y) / length
        } else {
            (x - self.start.x) / length
        }
    }

    fn along(&self, x: f32, y: f32) -> f32 {
        if self.vertical { -y } else { x }
    }
}
