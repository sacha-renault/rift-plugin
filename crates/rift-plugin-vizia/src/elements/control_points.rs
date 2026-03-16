use super::gui_prelude::*;

/// Internal event, ControlPoints sends to a child that
/// it should update his position
struct MovePoint {
    x: f32,
    y: f32,
}

/// The child points sends to the control that the point at idx
/// started the drag action
struct BeginDrag {
    child_idx: usize,
}

/// Struct containing the initial value of the point and it's idx in the array
/// (todo!() define type of array)
struct DraggablePoint {
    init_x: f32,
    init_y: f32,
    idx: usize,
}

impl View for DraggablePoint {
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

impl DestructThenBuildView for DraggablePoint {
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

#[derive(HandleExtension)]
pub struct ControlPoints {
    dragging: Option<(usize, Entity)>,
    points: Vec<(f32, f32)>,

    #[extension(ext)]
    on_change: Option<Box<dyn Fn(&mut EventContext, (f32, f32), usize)>>,
}

impl ControlPoints {
    pub fn new(cx: &mut Context, points: Vec<(f32, f32)>) -> Handle<'_, ControlPoints> {
        let intial_values = points.clone();
        Self {
            dragging: None,
            points,
            on_change: None,
        }
        .build(cx, move |cx| {
            for (idx, &(init_x, init_y)) in intial_values.iter().enumerate() {
                DraggablePoint {
                    init_x,
                    init_y,
                    idx,
                }
                .build_view(cx);
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
                if self.points[idx] != (x, y) {
                    self.points[idx] = (x, y);
                    cx.emit_to(entity, MovePoint { x, y });

                    if let Some(on_change) = &self.on_change {
                        on_change(cx, (x, y), idx)
                    }
                }
            }
            _ => {}
        });
    }
}
