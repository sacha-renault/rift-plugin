//! Contains utility structs and functions. Should be used in
//! [`super::elements`]

mod draw_utils;
mod gui_events;
mod handle_generic_extensions;
mod lens;
mod viewport_transform;

pub use draw_utils::{clip_bounds, make_strokepath};
pub use gui_events::{gesture_end, gesture_start, set_value, set_value_normalized};
pub use handle_generic_extensions::{FView, RedrawOnExt, ViewApplyModifiers};
pub use lens::make_lens;
pub use viewport_transform::ViewportTransform;

pub fn apply_transform_opt(func: Option<fn(f32) -> f32>, v: f32) -> f32 {
    if let Some(func) = func { func(v) } else { v }
}
