use rift_plugin_core::params::{
    ParamQueue, ParamQueueType,
    param_queue_impl::{ControlPoint, ControlPointEvent, ControlPoints},
};
use vizia::{
    events::EventMeta,
    vg::{Paint, PaintCap, PaintStyle, Path},
};

use super::gui_prelude::*;

/// Internal event, ControlPoints sends to a child that
/// it should update his position
enum EditorToChild {
    MovePoint { x: f32, y: f32 },
    SetVisible,
    SetInvisible,
}

/// The child points sends to the control that the point at idx
/// started the drag action
enum ChildToEditor {
    BeginDrag,
    InitializePoint {
        idx: usize,
        visible: bool,
        entity: Entity,
    },
    // InitializeTension {
    //     idx: usize,
    //     visible: bool,
    //     entity: Entity,
    // },
}

/// Struct containing the initial value of the point and it's idx in the array
/// (todo!() define type of array)
struct DraggablePoint {
    init_x: f32,
    init_y: f32,
}

impl View for DraggablePoint {
    fn element(&self) -> Option<&'static str> {
        Some("point")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        use ChildToEditor::*;
        use EditorToChild::*;

        event.map(|point_event, _| match point_event {
            &MovePoint { x, y } => {
                cx.set_left(Percentage(x * 100.));
                cx.set_bottom(Percentage(y * 100.));
            }
            SetVisible => cx.set_display(Display::Flex),
            SetInvisible => cx.set_display(Display::None),
        });

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseDown(btn) if btn == MouseButton::Left && !cx.is_disabled() => {
                cx.toggle_class("dragging", true);
                cx.emit(BeginDrag);
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
pub struct ControlPointsEditor {
    dragging: Option<(usize, Entity)>,
    param: ParamQueue<ControlPoints>,
    points: ControlPoints,
    point_entities: Vec<(bool, Entity)>,
    // tension_entities: Vec<(bool, Entity)>,
    rule: fn(usize, ControlPoint, &ControlPoints) -> ControlPoint,
    last_mouse_pos: (f32, f32),

    #[extension(ext)]
    on_change: Option<Box<dyn Fn(&mut EventContext, ControlPoint, usize)>>,
}

impl ControlPointsEditor {
    pub fn new(
        cx: &mut Context,
        param: ParamQueue<ControlPoints>,
        rule: fn(usize, ControlPoint, &ControlPoints) -> ControlPoint,
    ) -> Handle<'_, ControlPointsEditor> {
        let points = param.snapshot();
        let capacity = points.capacity();
        let initial_values = points.clone();

        Self {
            param,
            rule,
            points,

            dragging: None,
            on_change: None,
            last_mouse_pos: (0., 0.),
            point_entities: vec![(false, Entity::null()); capacity],
            // tension_entities: vec![(false, Entity::null()); capacity],
        }
        .build(cx, move |cx| {
            for idx in 0..capacity {
                let (x, y, visible) = if let Some(p) = initial_values.get(idx) {
                    (p.x, p.y, true)
                } else {
                    (0.0, 0.0, false)
                };

                let entity = DraggablePoint {
                    init_x: x,
                    init_y: y,
                }
                .build_view(cx)
                .display(if visible {
                    Display::Flex
                } else {
                    Display::None
                })
                .class("point-handle")
                .entity();

                cx.emit(ChildToEditor::InitializePoint {
                    idx,
                    visible,
                    entity,
                });
            }

            // for idx in 0..(capacity - 1) {
            //     let (x, y, visible) = if let (Some(p1), Some(p2)) =
            //         (initial_values.get(idx), initial_values.get(idx + 1))
            //     {
            //         let (x, y) = segment_handle(p1, p2);
            //         (x, y, true)
            //     } else {
            //         (0.0, 0.0, false)
            //     };

            //     let entity = DraggablePoint {
            //         init_x: x,
            //         init_y: y,
            //     }
            //     .build_view(cx)
            //     .display(if visible {
            //         Display::Flex
            //     } else {
            //         Display::None
            //     })
            //     .class("tension-handle")
            //     .entity();

            //     cx.emit(ChildToEditor::InitializeTension {
            //         idx,
            //         visible,
            //         entity,
            //     });
            // }
        })
    }

    fn idx_by_point_entity(&self, entity: Entity) -> Option<usize> {
        self.point_entities.iter().position(|e| e.1 == entity)
    }

    // fn idx_by_tension_entity(&self, entity: Entity) -> Option<usize> {
    //     self.tension_entities.iter().position(|e| e.1 == entity)
    // }

    fn normalize_mouse_input(&self, cx: &mut EventContext, x: f32, y: f32) -> (f32, f32) {
        let pbounds = cx.bounds();
        let x = ((x - pbounds.x) / pbounds.w).clamp(0.0, 1.0);
        let y = ((pbounds.y + pbounds.h - y) / pbounds.h).clamp(0.0, 1.0);
        (x, y)
    }

    fn on_begin_drag(&mut self, cx: &mut EventContext, meta: &mut EventMeta) {
        let entity = meta.origin;
        if let Some(point_idx) = self.idx_by_point_entity(entity) {
            self.dragging = Some((point_idx, entity));
            cx.capture();
        }
        meta.consume();
    }

    fn on_mouse_up(&mut self, cx: &mut EventContext) {
        cx.release();
        if let Some((_, entity)) = self.dragging.take() {
            cx.emit_to(entity, WindowEvent::MouseUp(MouseButton::Left));
        }
    }

    fn move_point(&mut self, cx: &mut EventContext, x: f32, y: f32) {
        let (x, y) = self.normalize_mouse_input(cx, x, y);
        self.last_mouse_pos = (x, y);

        let Some((idx, entity)) = self.dragging else {
            return;
        };

        let tension = self.points[idx].tension;
        let point = (self.rule)(idx, ControlPoint { x, y, tension }, &self.points);
        let event = ControlPointEvent::ModifyPoint {
            idx,
            x: point.x,
            y: point.y,
            tension,
        };

        if self.points[idx] != point && self.param.push_event(event).is_ok() {
            self.points.handle_event(event);
            cx.emit_to(
                entity,
                EditorToChild::MovePoint {
                    x: point.x,
                    y: point.y,
                },
            );

            if let Some(on_change) = &self.on_change {
                on_change(cx, point, idx);
            }
        }
    }

    fn add_point(&mut self, cx: &mut EventContext) {
        if self.points.len() >= self.points.capacity() {
            log::error!("Max number of control point reached");
            return;
        }

        let (x, y) = self.last_mouse_pos;

        let Some(idx) = self.points.iter().position(|p| p.x >= x) else {
            return;
        };

        let event = ControlPointEvent::AddPointBefore {
            idx,
            x,
            y,
            tension: 0.,
        };
        if !self.param.push_event(event).is_ok() {
            return;
        }
        self.points.handle_event(event);

        let Some(hidden_idx) = self.point_entities.iter().position(|p| !p.0) else {
            log::error!("Cannot add a new point ...");
            return;
        };

        let (_, entity) = self.point_entities.remove(hidden_idx);
        self.point_entities.insert(idx, (true, entity));
        cx.emit_to(entity, EditorToChild::MovePoint { x, y });
        cx.emit_to(entity, EditorToChild::SetVisible);
    }

    fn remove_point(&mut self, cx: &mut EventContext) {
        let (x, y) = self.last_mouse_pos;

        // Find the closest point
        let Some((idx, _)) = self.points.iter().enumerate().min_by(|(_, a), (_, b)| {
            let da = (a.x - x).hypot(a.y - y);
            let db = (b.x - x).hypot(b.y - y);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            return;
        };

        let event = ControlPointEvent::DeletePoint { idx };
        if !self.param.push_event(event).is_ok() {
            return;
        }
        self.points.handle_event(event);

        let (_, entity) = self.point_entities.remove(idx);
        self.point_entities.push((false, entity));
        cx.emit_to(entity, EditorToChild::SetInvisible);
        cx.needs_redraw();
    }
}

impl View for ControlPointsEditor {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|child_event, meta| match *child_event {
            ChildToEditor::BeginDrag => self.on_begin_drag(cx, meta),
            ChildToEditor::InitializePoint {
                entity,
                visible,
                idx,
            } => self.point_entities[idx] = (visible, entity),
            // ChildToEditor::InitializeTension {
            //     idx,
            //     visible,
            //     entity,
            // } => self.point_entities[idx] = (visible, entity),
        });

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseUp(btn) if btn == MouseButton::Left => self.on_mouse_up(cx),
            WindowEvent::MouseUp(btn) if btn == MouseButton::Middle => self.remove_point(cx),
            WindowEvent::MouseMove(x, y) => self.move_point(cx, x, y),
            WindowEvent::MouseDoubleClick(btn) if btn == MouseButton::Left => self.add_point(cx),
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

        let mut path = Path::new();

        draw_curved(&mut path, &self.points, viewport_transform);
        canvas.draw_path(&path, &paint);
    }
}

fn draw_curved(path: &mut Path, points: &[ControlPoint], vt: ViewportTransform) {
    if points.len() < 2 {
        return;
    }

    path.move_to(vt.transform(points[0].x, points[0].y));

    for i in 0..points.len() - 1 {
        let p1 = &points[i];
        let p2 = &points[i + 1];

        let (pixel_width, _) = vt.transform(p2.x - p1.x, 0.0);
        let steps = (pixel_width.ceil() as usize).max(1);

        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let shaped_t = shape(t, p1.tension);

            let x = p1.x + (p2.x - p1.x) * t;
            let y = p1.y + (p2.y - p1.y) * shaped_t;

            path.line_to(vt.transform(x, y));
        }
    }
}

/// curve_amount: 0.0 = linear, negative = log (fast start), positive = exp (slow start)
/// Common range: roughly -8.0 to 8.0
fn shape(t: f32, curve_amount: f32) -> f32 {
    if curve_amount.abs() < 1e-6 {
        t // linear
    } else {
        let k = curve_amount;
        // classic audio tension formula:
        // attempt to map the tension symmetrically around linear
        let exp = (k).exp2();
        if k > 0.0 {
            t.powf(exp) // exponential: slow start, fast end
        } else {
            1.0 - (1.0 - t).powf(exp.recip()) // logarithmic: fast start, slow end
        }
    }
}

// fn tension_from_drag(p1: &ControlPoint, p2: &ControlPoint, mouse_y: f32) -> f32 {
//     let mid_y = (p1.y + p2.y) / 2.0; // linear midpoint
//     let range = (p2.y - p1.y).abs().max(1e-6);

//     // how far the handle is from the straight line, normalized to -1..1
//     let offset = (mouse_y - mid_y) / (range / 2.0);

//     // map to your tension range
//     (offset * 8.0).clamp(-8.0, 8.0)
// }

// fn segment_handle(p1: &ControlPoint, p2: &ControlPoint) -> (f32, f32) {
//     const T: f32 = 0.5;
//     let x = p1.x + (p2.x - p1.x) * T;
//     let y = p1.y + (p2.y - p1.y) * shape(T, p1.tension);
//     (x, y)
// }
