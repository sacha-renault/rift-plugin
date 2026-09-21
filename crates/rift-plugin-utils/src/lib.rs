//! Small standalone utilities: bounded vectors, interpolation, channel routing,
//! dequeue buffers, note helpers and spaces.

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

pub use multi_channel::MultiChannel;
