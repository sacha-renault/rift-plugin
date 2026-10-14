//! Small standalone utilities: bounded vectors, interpolation, channel routing,
//! dequeue buffers, note helpers, spaces and a background task executor.

pub mod background_thread;
pub mod bounded_vec;
pub mod conversion;
pub mod dequeue_buffer;
pub mod interpo;
pub mod multi_channel;
pub mod notes;
pub mod spaces;

mod test_macros;

use std::{cell::RefCell, rc::Rc};

/// Type alias of [`Rc<RefCell<T>>`]
pub type ConsumerCell<T> = Rc<RefCell<T>>;

pub use background_thread::BackgroundExecutor;
pub use bounded_vec::BoundedVec;
pub use multi_channel::MultiChannel;

pub const DEFAULT_SAMPLERATE: f64 = 48000.;
