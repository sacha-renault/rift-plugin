//! Contains utility structs and functions. Should be used in
//! [`super::elements`]

pub(crate) mod active_element;
pub(crate) mod cached_texture;
pub(crate) mod draw_utils;
pub(crate) mod gui_events;
pub(crate) mod handle_generic_extensions;
pub(crate) mod lens;
pub(crate) mod viewport_transform;

pub use active_element::{ActiveElementData, ActiveElementEvent};
pub use cached_texture::CachedTexture;
pub use draw_utils::{change_color_opacity, clip_bounds, make_strokepath};
pub use gui_events::{gesture_end, gesture_start, set_value, set_value_normalized};
pub use handle_generic_extensions::{
    DestructThenBuildView, FView, RedrawLensEvent, RedrawOnExt, ViewApplyModifiers,
};
pub use lens::make_lens;
pub use viewport_transform::ViewportTransform;

pub fn apply_transform_opt(func: Option<fn(f32) -> f32>, v: f32) -> f32 {
    if let Some(func) = func { func(v) } else { v }
}
