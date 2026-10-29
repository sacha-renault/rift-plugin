//! Audio and event buffer views handed to the plugin's `process`.

mod buffer;
mod buffers;
pub mod event_handling;
mod event_iterator;
pub mod frame;

pub use buffer::Buffer;
pub use buffers::Buffers;
pub use event_handling::InputEvent;
pub use event_iterator::{EventsCursor, FrameEventsIter};
pub use frame::{Frame, SampleFrames};
