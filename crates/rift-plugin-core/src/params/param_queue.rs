use std::cell::UnsafeCell;

use crossbeam_queue::ArrayQueue;

pub use super::{ClapParam, TypedParamRef};

pub trait ParamQueueType {
    type EventType;

    fn handle_event(&self, event: Self::EventType);
}

pub struct ParamQueue<T: ParamQueueType> {
    cache: UnsafeCell<T>,
    queue: ArrayQueue<T::EventType>,
}

impl<T: ParamQueueType> TypedParamRef for ParamQueue<T> {
    type ValueType = T;

    fn value(&self) -> &Self::ValueType {
        unsafe { &*self.cache.get() }
    }

    fn set_value(&self, value: Self::ValueType) {
        // when we set we must empty the queue
        while let Some(_) = self.queue.pop() {}
        unsafe { *self.cache.get() = value }
    }
}
