//! Audio and event buffer views handed to the plugin's `process`.

mod buffer;
mod buffers;
pub mod frame;

pub use buffer::Buffer;
pub use buffers::Buffers;
pub use frame::{Frame, SampleFrames};
