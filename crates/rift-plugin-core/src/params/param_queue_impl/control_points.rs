use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{params::ParamQueueType, utils::bounded_vec::BoundedVec};

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq)]
pub struct ControlPoint {
    pub x: f32,
    pub y: f32,
    pub tension: f32,
}

/// Event pushed from the UI thread to mutate control points.
#[derive(Clone, Copy)]
pub enum ControlPointEvent {
    DeletePoint {
        idx: usize,
    },
    ModifyPoint {
        idx: usize,
        x: f32,
        y: f32,
        tension: f32,
    },
    AddPointBefore {
        idx: usize,
        x: f32,
        y: f32,
        tension: f32,
    },
}

/// A bounded list of control points, safe for use in the audio thread.
///
/// Capacity is fixed at construction. Insertions beyond capacity are
/// silently dropped, and all operations are bounds-checked to avoid panics.
/// This guarantees no heap allocation after `new()`.
#[derive(Clone, Deserialize, Serialize)]
pub struct ControlPoints {
    points: BoundedVec<ControlPoint>,
    max_tension: f32,
}

impl ControlPoints {
    pub fn new(capacity: usize) -> Self {
        Self {
            points: BoundedVec::new(capacity),
            max_tension: 10.,
        }
    }

    pub fn with_value(values: Vec<ControlPoint>, capacity: usize) -> Self {
        let mut points = BoundedVec::new(capacity);
        points.extend_from_slice(&values);
        Self {
            points,
            max_tension: 10.,
        }
    }

    pub fn copy_from(&mut self, other: &ControlPoints) {
        self.points.clear();

        // This will panic if other.points.len() > self.points.capacity()
        // but it will NEVER reallocate.
        self.points.copy_from_slice(&other.points);
        self.max_tension = other.max_tension;
    }

    pub fn capacity(&self) -> usize {
        self.points.capacity()
    }

    pub fn get_value(&self, position: f32) -> f32 {
        let Some(right_idx) = self.points.iter().position(|p| p.x >= position) else {
            // Past all points - hold last value
            return self.points.last().map(|p| p.y).unwrap_or_default();
        };

        let right = &self.points[right_idx];

        if right_idx == 0 {
            right.y
        } else {
            let left = &self.points[right_idx - 1];
            let fract = (position - left.x) / (right.x - left.x);
            let (_, y) = pow_interpolation(left, right, fract);
            y
        }
    }

    fn can_add_point(&self) -> bool {
        !self.points.is_full()
    }
}

impl Deref for ControlPoints {
    type Target = [ControlPoint];

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl ParamQueueType for ControlPoints {
    type EventType = ControlPointEvent;

    fn handle_event(&mut self, event: Self::EventType) {
        use ControlPointEvent::*;

        match event {
            DeletePoint { idx } => {
                if idx < self.points.len() {
                    self.points.remove(idx);
                }
            }
            ModifyPoint { idx, x, y, tension } => {
                if let Some(point) = self.points.get_mut(idx) {
                    *point = ControlPoint {
                        x,
                        y,
                        tension: tension.clamp(-self.max_tension, self.max_tension),
                    }
                }
            }
            AddPointBefore { idx, x, y, tension } if self.can_add_point() => {
                self.points.insert(
                    idx,
                    ControlPoint {
                        x,
                        y,
                        tension: tension.clamp(-self.max_tension, self.max_tension),
                    },
                );
            }
            _ => {}
        }
    }

    fn snapshot(&self) -> Self {
        // Snapshot should provide the full array with same capacity
        self.clone()
    }
}

fn pow_interpolation(p1: &ControlPoint, p2: &ControlPoint, t: f32) -> (f32, f32) {
    let x = p1.x + (p2.x - p1.x) * t;
    let y = p1.y + (p2.y - p1.y) * shape(t, p1.tension);
    (x, y)
}

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
