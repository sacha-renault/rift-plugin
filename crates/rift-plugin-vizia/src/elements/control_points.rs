use vizia::vg::{Paint, PaintCap, PaintStyle};

use super::gui_prelude::*;

/// Representation of a point
#[derive(Clone, Copy, PartialEq)]
pub struct ControlPoint {
    pub x: f32,
    pub y: f32,
}

impl From<ControlPoint> for MovePoint {
    fn from(ControlPoint { x, y }: ControlPoint) -> Self {
        MovePoint { x, y }
    }
}

impl From<(f32, f32)> for ControlPoint {
    fn from((x, y): (f32, f32)) -> Self {
        ControlPoint { x, y }
    }
}

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

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseDown(btn) if btn == MouseButton::Left && !cx.is_disabled() => {
                cx.toggle_class("dragging", true);
                cx.emit(BeginDrag {
                    child_idx: self.idx,
                });
            }
            WindowEvent::MouseUp(btn) if btn == MouseButton::Left => {
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
    points: Vec<ControlPoint>,
    rule: fn(usize, ControlPoint, &Vec<ControlPoint>) -> ControlPoint,

    #[extension(ext)]
    on_change: Option<Box<dyn Fn(&mut EventContext, ControlPoint, usize)>>,
}

impl ControlPoints {
    pub fn new(
        cx: &mut Context,
        points: impl Into<Vec<ControlPoint>>,
        rule: fn(usize, ControlPoint, &Vec<ControlPoint>) -> ControlPoint,
    ) -> Handle<'_, ControlPoints> {
        let points = points.into();
        let intial_values = points.clone();
        Self {
            dragging: None,
            points,
            on_change: None,
            rule,
        }
        .build(cx, move |cx| {
            for (idx, &ControlPoint { x, y }) in intial_values.iter().enumerate() {
                DraggablePoint {
                    init_x: x,
                    init_y: y,
                    idx,
                }
                .build_view(cx);
            }
        })
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

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseUp(btn) if btn == MouseButton::Left => {
                cx.release();

                // We resend an event to be sure points receives it
                if let Some((_, entity)) = self.dragging.take() {
                    cx.emit_to(entity, WindowEvent::MouseUp(MouseButton::Left));
                }
            }
            WindowEvent::MouseMove(x, y) => {
                let Some((idx, entity)) = self.dragging else {
                    return;
                };

                let pbounds = cx.bounds();
                let x = ((x - pbounds.x) / pbounds.w).clamp(0.0, 1.0);
                let y = ((pbounds.y + pbounds.h - y) / pbounds.h).clamp(0.0, 1.0);

                // validates some rules here, and send
                // Update internal repr
                let point = (self.rule)(idx, ControlPoint { x, y }, &self.points);

                if self.points[idx] != point {
                    self.points[idx] = point;
                    cx.emit_to(entity, MovePoint::from(point));

                    if let Some(on_change) = &self.on_change {
                        on_change(cx, point, idx)
                    }
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let viewport_transform = ViewportTransform::new(cx.bounds());
        let mut paint = Paint::default();
        paint
            .set_color(cx.font_color())
            .set_stroke_width(cx.border_width())
            .set_stroke_cap(PaintCap::Round)
            .set_style(PaintStyle::Stroke)
            .set_anti_alias(true);

        let Some(path_with_closing) = make_strokepath(
            self.points
                .iter()
                .copied()
                .map(|ControlPoint { x, y }| (x, y)),
            viewport_transform,
            0.,
        ) else {
            return;
        };

        canvas.draw_path(&path_with_closing.path, &paint);
    }
}
