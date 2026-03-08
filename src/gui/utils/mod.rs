//! Contains utility structs and functions. Should be used in
//! [`super::elements`]

mod audio_peak;
mod draw_utils;
mod gui_events;
mod handle_generic_extensions;
mod lens;
mod viewport_transform;

pub use audio_peak::AudioPeaks;
pub use draw_utils::{clip_bounds, make_strokepath};
pub use gui_events::{gesture_end, gesture_start, set_value, set_value_normalized};
pub use handle_generic_extensions::{FView, RedrawOnExt, ViewApplyModifiers};
pub use lens::make_lens;
pub use viewport_transform::ViewportTransform;
