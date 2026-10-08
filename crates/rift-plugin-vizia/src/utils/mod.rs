#[derive(Clone, Copy)]
pub(crate) struct Drag {
    /// Vertical position of the cursor at the previous event (physical px).
    pub last_y: f32,
    /// Value being edited. Kept apart from the signal so that the value is not
    /// quantized by the host between two mouse moves.
    pub value: f32,
}
