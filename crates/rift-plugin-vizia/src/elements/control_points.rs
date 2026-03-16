use super::gui_prelude::*;

struct MovePoint {
    x: f32,
    y: f32,
}

struct BeginDrag {
    child_idx: usize,
}

struct Breakpoint {
    init_x: f32,
    init_y: f32,
    idx: usize,
}

impl View for Breakpoint {
    fn element(&self) -> Option<&'static str> {
        Some("point")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|point_event, _| match point_event {
            &MovePoint { x, y } => {
                cx.set_left(Percentage(x * 100.));
                cx.set_bottom(Percentage(y * 100.));
            }
        });

        event.map(|window_event, _| match window_event {
            &WindowEvent::MouseDown(btn) if btn == MouseButton::Left && !cx.is_disabled() => {
                cx.toggle_class("dragging", true);
                cx.emit(BeginDrag {
                    child_idx: self.idx,
                });
            }
            &WindowEvent::MouseUp(btn) if btn == MouseButton::Left => {
                cx.toggle_class("dragging", false);
            }
            _ => {}
        });
    }
}

impl DestructThenBuildView for Breakpoint {
    fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let (init_x, init_y) = (self.init_x, self.init_y);

        self.build(cx, |_| {})
            .corner_radius(Percentage(100.))
            .border_color(Color::azure())
            .position_type(PositionType::Absolute)
            .transform(vec![Transform::Translate((
                LengthOrPercentage::Percentage(-50.),
                LengthOrPercentage::Percentage(50.),
            ))])
            .left(Percentage(init_x * 100.))
            .bottom(Percentage(init_y * 100.))
    }
}

impl Breakpoint {
    pub fn new(init_x: f32, init_y: f32, idx: usize) -> Self {
        Self {
            init_x,
            init_y,
            idx,
        }
    }
}

pub struct ControlPoints {
    dragging: Option<(usize, Entity)>,
    points: Vec<(f32, f32)>,
}

impl ControlPoints {
    pub fn new(cx: &mut Context, points: Vec<(f32, f32)>) -> Handle<'_, ControlPoints> {
        let intial_values = points.clone();
        Self {
            dragging: None,
            points,
        }
        .build(cx, move |cx| {
            for (idx, &(x, y)) in intial_values.iter().enumerate() {
                Breakpoint::new(x, y, idx).build_view(cx);
            }
        })
        .overflow(Overflow::Hidden)
    }
}

impl View for ControlPoints {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|drag_event, meta| match *drag_event {
            BeginDrag { child_idx } => {
                self.dragging = Some((child_idx, meta.origin));
                meta.consume();
                cx.capture();
            }
        });

        event.map(|window_event, _| match window_event {
            &WindowEvent::MouseUp(btn) if btn == MouseButton::Left => {
                cx.release();

                // We resend an event to be sure points receives it
                if let Some((_, entity)) = self.dragging.take() {
                    cx.emit_to(entity, WindowEvent::MouseUp(MouseButton::Left));
                }
            }
            &WindowEvent::MouseMove(x, y) => {
                let Some((idx, entity)) = self.dragging else {
                    return;
                };

                let previous_x = if idx > 0 { self.points[idx - 1].0 } else { 0. };

                // todo!()
                // Use some custom validation rules
                let next_x = if idx < self.points.len() - 1 {
                    self.points[idx + 1].0
                } else {
                    1.
                };

                let pbounds = cx.bounds();
                let x = ((x - pbounds.x) / pbounds.w).clamp(previous_x, next_x);
                let y = ((pbounds.y + pbounds.h - y) / pbounds.h).clamp(0.0, 1.0);

                // validates some rules here, and send
                // Update internal repr
                self.points[idx] = (x, y);
                cx.emit_to(entity, MovePoint { x, y });
            }
            _ => {}
        });
    }
}
