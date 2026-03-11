use std::{cell::RefCell, rc::Rc};

pub mod gui;
pub mod params;
pub mod transport;
pub mod utils;

pub type RcCell<T> = Rc<RefCell<T>>;

// Remove the cfg test so it can be used anywhere
// not optimal but so small it's not so much a pb
mod test_macros;
