//! Audio and event buffer views handed to the plugin's `process`.

mod buffer;
mod buffers;
pub mod event_handling;
pub mod frame;
mod zip_event;

pub use buffer::Buffer;
pub use buffers::Buffers;
pub use event_handling::ZipEventConfig;
pub use frame::{Frame, SampleFrames};
