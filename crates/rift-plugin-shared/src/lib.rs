use std::{cell::RefCell, rc::Rc};

pub mod gui;
pub mod params;
pub mod transport;
pub mod utils;

pub type RcCell<T> = Rc<RefCell<T>>;
