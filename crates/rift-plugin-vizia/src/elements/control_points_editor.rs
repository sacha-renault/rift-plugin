use rift_plugin_core::params::{
    ParamQueue, ParamQueueType,
    param_queue_impl::{ControlPoint, ControlPointEvent, ControlPoints},
};
use vizia::{
    events::EventMeta,
    vg::{Paint, PaintCap, PaintStyle, Path},
};

use crate::utils::draw_utils::close_path;

use super::gui_prelude::*;

const DELTA_DRAG_FACTOR: f32 = 1.45;

// Internal events

/// Editor → child: reposition or toggle visibility.
enum EditorToChild {
    MovePoint { x: f32, y: f32 },
    SetVisible,
    SetInvisible,
}

/// Child → editor: registration and drag start.
enum ChildToEditor {
    BeginDrag,
    InitializePoint {
        idx: usize,
        visible: bool,
        entity: Entity,
    },
    InitializeTension {
        idx: usize,
        visible: bool,
        entity: Entity,
    },
}

// DraggablePoint (shared by point handles and tension handles)

struct DraggablePoint {
    init_x: f32,
    init_y: f32,
}

impl View for DraggablePoint {
    fn element(&self) -> Option<&'static str> {
        Some("point")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|msg, _| match msg {
            &EditorToChild::MovePoint { x, y } => {
                cx.set_left(Percentage(x * 100.));
                cx.set_bottom(Percentage(y * 100.));
            }
            EditorToChild::SetVisible => cx.set_display(Display::Flex),
            EditorToChild::SetInvisible => cx.set_display(Display::None),
        });

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseDown(MouseButton::Left) if !cx.is_disabled() => {
                cx.toggle_class("dragging", true);
                cx.emit(ChildToEditor::BeginDrag);
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
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
            .position_type(PositionType::Absolute)
            .transform(vec![Transform::Translate((
                LengthOrPercentage::Percentage(-50.),
                LengthOrPercentage::Percentage(50.),
            ))])
            .left(Percentage(init_x * 100.))
            .bottom(Percentage(init_y * 100.))
    }
}

// ControlPointsEditor

#[derive(HandleExtension)]
pub struct ControlPointsEditor {
    param: ParamQueue<ControlPoints>,
    points: ControlPoints,
    rule: fn(usize, ControlPoint, &ControlPoints) -> ControlPoint,

    dragging: Option<(usize, Entity)>,
    last_mouse_pos: (f32, f32),

    point_entities: Vec<(bool, Entity)>,
    tension_entities: Vec<(bool, Entity)>,

    #[extension(ext)]
    on_change: Option<Box<dyn Fn(&mut EventContext, ControlPoint, usize)>>,

    #[extension(ext, set = true)]
    filled: bool,

    #[extension(ext)]
    fill_opacity: u8,
}

impl ControlPointsEditor {
    pub fn new(
        cx: &mut Context,
        param: ParamQueue<ControlPoints>,
        rule: fn(usize, ControlPoint, &ControlPoints) -> ControlPoint,
    ) -> Handle<'_, ControlPointsEditor> {
        let points = unsafe { param.snapshot() };
        let capacity = points.capacity();
        let initial = points.clone();

        Self {
            param,
            rule,
            points,
            dragging: None,
            on_change: None,
            last_mouse_pos: (0., 0.),
            point_entities: vec![(false, Entity::null()); capacity],
            tension_entities: vec![(false, Entity::null()); capacity],
            filled: false,
            fill_opacity: 100,
        }
        .build(cx, move |cx| {
            Self::build_tension_handles(cx, &initial, capacity);
            Self::build_point_handles(cx, &initial, capacity);
        })
    }

    fn build_point_handles(cx: &mut Context, initial: &ControlPoints, capacity: usize) {
        for idx in 0..capacity {
            let (x, y, visible) = match initial.get(idx) {
                Some(p) => (p.x, p.y, true),
                None => (0.0, 0.0, false),
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
    }

    fn build_tension_handles(cx: &mut Context, initial: &ControlPoints, capacity: usize) {
        for idx in 0..(capacity - 1) {
            let (x, y, visible) = match (initial.get(idx), initial.get(idx + 1)) {
                (Some(p1), Some(p2)) => {
                    let (x, y) = segment_handle(p1, p2);
                    (x, y, true)
                }
                _ => (0.0, 0.0, false),
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
            .class("tension-handle")
            .entity();

            cx.emit(ChildToEditor::InitializeTension {
                idx,
                visible,
                entity,
            });
        }
    }

    // Lookups

    fn idx_by_point_entity(&self, entity: Entity) -> Option<usize> {
        self.point_entities.iter().position(|e| e.1 == entity)
    }

    fn idx_by_tension_entity(&self, entity: Entity) -> Option<usize> {
        self.tension_entities.iter().position(|e| e.1 == entity)
    }

    fn normalize_mouse(&self, cx: &mut EventContext, x: f32, y: f32) -> (f32, f32) {
        let b = cx.bounds();
        let nx = ((x - b.x) / b.w).clamp(0.0, 1.0);
        let ny = ((b.y + b.h - y) / b.h).clamp(0.0, 1.0);
        (nx, ny)
    }

    // Event dispatch helpers

    /// Push an event to the param queue *and* apply it locally.
    /// Returns `true` on success.
    fn try_push_event(&mut self, cx: &mut EventContext, event: ControlPointEvent) -> bool {
        if self.param.push_event(event).is_err() {
            return false;
        }
        self.points.handle_event(event);
        cx.needs_redraw();
        true
    }

    fn emit_move(cx: &mut EventContext, entity: Entity, x: f32, y: f32) {
        cx.emit_to(entity, EditorToChild::MovePoint { x, y });
    }

    // Dragging

    fn on_begin_drag(&mut self, cx: &mut EventContext, meta: &mut EventMeta) {
        let entity = meta.origin;

        let idx = self
            .idx_by_point_entity(entity)
            .or_else(|| self.idx_by_tension_entity(entity));

        if let Some(idx) = idx {
            self.dragging = Some((idx, entity));
            cx.capture();
            meta.consume();
        }
    }

    fn on_mouse_up(&mut self, cx: &mut EventContext) {
        cx.release();
        if let Some((_, entity)) = self.dragging.take() {
            cx.emit_to(entity, WindowEvent::MouseUp(MouseButton::Left));
        }
    }

    fn move_point(&mut self, cx: &mut EventContext, x: f32, y: f32) {
        let Some((idx, entity)) = self.dragging else {
            return;
        };

        let tension = self.points[idx].tension;
        let point = (self.rule)(idx, ControlPoint { x, y, tension }, &self.points);

        if self.points[idx] == point {
            return;
        }

        let event = ControlPointEvent::ModifyPoint {
            idx,
            x: point.x,
            y: point.y,
            tension,
        };

        if !self.try_push_event(cx, event) {
            return;
        }

        // Update point handle
        Self::emit_move(cx, entity, point.x, point.y);

        // Update adjacent tension handles
        self.update_tension_handle_after(cx, idx);
        if idx > 0 {
            self.update_tension_handle_after(cx, idx - 1);
        }

        if let Some(on_change) = &self.on_change {
            on_change(cx, point, idx);
        }
    }

    fn move_tension(&mut self, cx: &mut EventContext, _x: f32, y: f32) {
        let Some((idx, entity)) = self.dragging else {
            return;
        };

        let p1 = &self.points[idx];
        let p2 = match self.points.get(idx + 1) {
            Some(p) => p,
            None => return,
        };

        let tension = tension_from_drag(p1, p2, y);
        let event = ControlPointEvent::ModifyPoint {
            idx,
            x: p1.x,
            y: p1.y,
            tension,
        };

        if !self.try_push_event(cx, event) {
            return;
        }

        // Reposition the tension handle on the new curve
        let (hx, hy) = segment_handle(&self.points[idx], &self.points[idx + 1]);
        Self::emit_move(cx, entity, hx, hy);

        if let Some(on_change) = &self.on_change {
            on_change(cx, self.points[idx], idx);
        }
    }

    fn get_points_with_handle(
        &self,
        idx: usize,
    ) -> Option<(bool, Entity, &ControlPoint, &ControlPoint)> {
        let (act, te) = <[_]>::get(&self.tension_entities, idx).copied()?;
        let p1 = <[_]>::get(&self.points, idx)?;
        let p2 = <[_]>::get(&self.points, idx + 1)?;
        Some((act, te, p1, p2))
    }

    /// Re-sync the tension handles that depend on point `idx` (i.e. idx-1 and idx).
    fn update_tension_handle_after(&self, cx: &mut EventContext, idx: usize) {
        // Tension handle *before* this point (segment idx-1 → idx)
        if let Some((true, entity, p1, p2)) = self.get_points_with_handle(idx) {
            let (hx, hy) = segment_handle(p1, p2);
            Self::emit_move(cx, entity, hx, hy);
        }
    }

    fn reset_tension(&mut self, cx: &mut EventContext, idx: usize) {
        // Idx of tension is same idx has its point.
        let ControlPoint { x, y, .. } = self.points[idx];
        let event = ControlPointEvent::ModifyPoint {
            idx,
            x,
            y,
            tension: 0f32,
        };

        if self.try_push_event(cx, event) {
            self.update_tension_handle_after(cx, idx);
        }
    }

    fn add_point(&mut self, cx: &mut EventContext) {
        if self.points.len() >= self.points.capacity() {
            log::error!(
                "Max number of control points reached {} >= {}",
                self.points.capacity(),
                self.points.len()
            );
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
        if !self.try_push_event(cx, event) {
            return;
        }

        // Recycle a hidden point entity
        let Some(hidden) = self.point_entities.iter().position(|p| !p.0) else {
            log::error!("No spare point entity to activate");
            debug_assert!(false, "No spare point entity to activate");
            return;
        };

        let (_, entity) = self.point_entities.remove(hidden);
        self.point_entities.insert(idx, (true, entity));
        Self::emit_move(cx, entity, x, y);
        cx.emit_to(entity, EditorToChild::SetVisible);

        // Recycle a hidden tension entity - inserting a point splits one segment into two
        let Some(hidden_t) = self.tension_entities.iter().position(|t| !t.0) else {
            log::error!("No spare tension entity to activate");
            debug_assert!(false, "No spare tension entity to activate");
            return;
        };

        let (_, t_entity) = self.tension_entities.remove(hidden_t);
        self.tension_entities.insert(idx, (true, t_entity));
        cx.emit_to(t_entity, EditorToChild::SetVisible);

        // Reposition the two tension handles on either side of the new point
        self.update_tension_handle_after(cx, idx);
        if idx > 0 {
            self.update_tension_handle_after(cx, idx - 1);
        }
    }

    fn remove_point(&mut self, cx: &mut EventContext, idx: usize) {
        let event = ControlPointEvent::DeletePoint { idx };
        if !self.try_push_event(cx, event) {
            return;
        }

        let (_, entity) = self.point_entities.remove(idx);
        self.point_entities.push((false, entity));
        cx.emit_to(entity, EditorToChild::SetInvisible);

        // Hide one tension handle - removing a point merges two segments into one.
        // Pick the tension entity at `idx`, or `idx - 1` if idx was the last visible segment.
        let visible_count = self.tension_entities.iter().filter(|t| t.0).count();
        let t_idx = idx.min(visible_count.saturating_sub(1));

        let (_, t_entity) = self.tension_entities.remove(t_idx);
        self.tension_entities.push((false, t_entity));
        cx.emit_to(t_entity, EditorToChild::SetInvisible);

        // Reposition the surviving tension handle that now spans the merged segment
        if idx > 0
            && let Some((true, entity, p1, p2)) = self.get_points_with_handle(idx - 1)
        {
            let (hx, hy) = segment_handle(p1, p2);
            Self::emit_move(cx, entity, hx, hy);
        }
    }
}

// View imp

impl View for ControlPointsEditor {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|child_event, meta| match *child_event {
            ChildToEditor::BeginDrag => self.on_begin_drag(cx, meta),
            ChildToEditor::InitializePoint {
                idx,
                visible,
                entity,
            } => {
                self.point_entities[idx] = (visible, entity);
            }
            ChildToEditor::InitializeTension {
                idx,
                visible,
                entity,
            } => {
                self.tension_entities[idx] = (visible, entity); // was point_entities - bug!
            }
        });

        event.map(|window_event, _| match *window_event {
            WindowEvent::MouseUp(MouseButton::Left) => self.on_mouse_up(cx),
            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                if let Some(idx) = self.idx_by_point_entity(cx.hovered()) {
                    self.remove_point(cx, idx);
                } else {
                    self.add_point(cx)
                }
            }
            WindowEvent::MouseMove(x, y) => {
                let (x, y) = self.normalize_mouse(cx, x, y);
                self.last_mouse_pos = (x, y);

                let Some((_, entity)) = self.dragging else {
                    return;
                };

                if self.idx_by_point_entity(entity).is_some() {
                    self.move_point(cx, x, y);
                } else if self.idx_by_tension_entity(entity).is_some() {
                    self.move_tension(cx, x, y);
                }
            }
            WindowEvent::MouseDown(MouseButton::Right) => {
                if let Some(idx) = self.idx_by_tension_entity(cx.hovered()) {
                    self.reset_tension(cx, idx);
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let vt = ViewportTransform::new(cx.bounds());

        let mut paint = Paint::default();
        paint
            .set_color(cx.font_color())
            .set_stroke_width(cx.border_width())
            .set_stroke_cap(PaintCap::Round)
            .set_style(PaintStyle::Stroke)
            .set_anti_alias(true);

        let mut path = Path::new();
        draw_curved(&mut path, &self.points, &vt);
        canvas.draw_path(&path, &paint);

        if self.filled {
            close_path(&mut path, &vt, 0.);
            paint
                .set_style(PaintStyle::Fill)
                .set_anti_alias(false)
                .set_color(change_color_opacity(cx.font_color(), self.fill_opacity));
            canvas.draw_path(&path, &paint);
        }
    }
}

// Curve mat

fn draw_curved(path: &mut Path, points: &[ControlPoint], vt: &ViewportTransform) {
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

/// Attempt to map the tension symmetrically around linear.
///
/// `curve_amount`:  0.0 = linear,  negative = log (fast start),  positive = exp (slow start).
fn shape(t: f32, curve_amount: f32) -> f32 {
    if curve_amount.abs() < 1e-6 {
        return t;
    }

    let exp = curve_amount.exp2();
    if curve_amount > 0.0 {
        t.powf(exp)
    } else {
        1.0 - (1.0 - t).powf(exp.recip())
    }
}

fn tension_from_drag(p1: &ControlPoint, p2: &ControlPoint, mouse_y: f32) -> f32 {
    let (_, current_handle_y) = segment_handle(p1, p2);
    let delta_y = (mouse_y - current_handle_y) * DELTA_DRAG_FACTOR;
    if p1.y <= p2.y {
        p1.tension - delta_y
    } else {
        p1.tension + delta_y
    }
}

fn segment_handle(p1: &ControlPoint, p2: &ControlPoint) -> (f32, f32) {
    const T: f32 = 0.5;
    let x = p1.x + (p2.x - p1.x) * T;
    let y = p1.y + (p2.y - p1.y) * shape(T, p1.tension);
    (x, y)
}
