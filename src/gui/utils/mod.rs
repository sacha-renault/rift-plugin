//! Contains utility structs and functions. Should be used in
//! [`super::elements`]

mod audio_peak;
mod denormalizer;
mod draw_utils;
mod gui_events;
mod handle_generic_extensions;
mod lens;
mod peak_bucket;
mod windowed_buffer;

pub use audio_peak::AudioPeaks;
pub use denormalizer::Denormalizer;
pub use draw_utils::{clip_bounds, make_closed_strokepath, make_open_strokepath};
pub use gui_events::{gesture_end, gesture_start, set_value, set_value_normalized};
pub use handle_generic_extensions::{FView, RedrawOnExt, ViewApplyModifiers};
pub use lens::make_lens;
pub use peak_bucket::PeakBucket;
pub use windowed_buffer::{WindowBuffer, WindowBufferMode};
