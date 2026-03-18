pub mod gui;
pub mod params;
pub mod transport;
pub mod utils;

pub mod prelude {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    pub type ConsumerCell<T> = Rc<RefCell<T>>;
    pub use transport::{BlockInfo, BlockTime, ChannelsInfo};
}

// Remove the cfg test so it can be used anywhere
// not optimal but so small it's not so much a pb
mod test_macros;
