pub mod gui;
pub mod params;
pub mod transport;
pub mod utils;

pub mod prelude {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    /// Type alias of [`Rc<RefCell<T>>`]
    pub type ConsumerCell<T> = Rc<RefCell<T>>;
    pub use transport::{BlockInfo, BlockTime, ChannelsInfo};

    pub use utils::atomic_f32::AtomicF32;
}

// Remove the cfg test so it can be used anywhere
// not optimal but so small it's not so much a pb
mod test_macros;
