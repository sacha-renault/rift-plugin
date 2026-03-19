use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::params::ParamQueueType;

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
    points: Vec<ControlPoint>,
    capacity: usize,
}

impl ControlPoints {
    pub fn new(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn with_value(values: Vec<ControlPoint>, capacity: usize) -> Self {
        assert!(values.len() <= capacity);
        let mut points = Vec::with_capacity(capacity);
        points.extend_from_slice(&values);
        Self { points, capacity }
    }
}

impl Deref for ControlPoints {
    type Target = Vec<ControlPoint>;

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl ControlPoints {
    fn can_add_point_at(&self, idx: usize) -> bool {
        self.points.len() < self.capacity && idx <= self.points.len()
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
                    *point = ControlPoint { x, y, tension }
                }
            }
            AddPointBefore { idx, x, y, tension } if self.can_add_point_at(idx) => {
                self.points.insert(idx, ControlPoint { x, y, tension });
            }
            _ => {}
        }
    }

    fn snapshot(&self) -> Self {
        // Snapshot should provide the full array with same capacity
        let mut clone = Vec::with_capacity(self.capacity);
        clone.extend_from_slice(&self.points);
        Self {
            points: clone,
            capacity: self.capacity,
        }
    }
}
