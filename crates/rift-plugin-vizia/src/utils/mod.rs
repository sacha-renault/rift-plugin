mod paint;

pub(crate) use paint::*;

#[derive(Clone, Copy)]
pub struct Drag {
    /// Vertical position of the cursor at the previous event (physical px).
    pub last_y: f32,
    /// Value being edited. Kept apart from the signal so that the value is not
    /// quantized by the host between two mouse moves.
    pub value: f32,
}

/// The index of the option a normalized value stands for.
pub fn index_of(value: f32, count: usize) -> usize {
    ((value.clamp(0.0, 1.0) * (count - 1) as f32).round() as usize).min(count - 1)
}

/// The normalized value of the option at `index`.
pub fn value_of(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.0
    } else {
        index as f32 / (count - 1) as f32
    }
}
