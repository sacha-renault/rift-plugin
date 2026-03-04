//! Contains utility structs and functions. Should be used in
//! [`super::elements`]

mod audio_peak;
mod denormalizer;
mod draw_utils;
mod peak_bucket;
mod windowed_buffer;

pub use audio_peak::AudioPeaks;
pub use denormalizer::Denormalizer;
pub use draw_utils::{clip_bounds, make_closed_strokepath, make_open_strokepath};
pub use peak_bucket::PeakBucket;
pub use windowed_buffer::WindowBufferAvg;
