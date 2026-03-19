use std::cell::UnsafeCell;

use clack_plugin::plugin::PluginError;
use crossbeam_queue::ArrayQueue;
use serde::{Serialize, de::DeserializeOwned};

pub use super::Persistent;

pub trait ParamQueueType: Clone {
    type EventType;

    fn handle_event(&mut self, event: Self::EventType);
}

pub struct ParamQueue<T: ParamQueueType> {
    cache: UnsafeCell<T>,
    queue: ArrayQueue<T::EventType>,
}

/// SAFETY: Access to the inner [`UnsafeCell<T>`] is governed by these rules:
///
/// - The **audio thread** is the sole owner of `value()` during processing.
/// - The **UI thread** may call `value()` once each time it is (re)opened,
///   to snapshot the current state. At this point no events have been pushed
///   yet, so there is no writer.
/// - After that initial read, the UI thread only interacts via `queue.push()`
///   and must not call `value()` again until the next UI open.
unsafe impl<T: ParamQueueType + Send> Send for ParamQueue<T> {}
unsafe impl<T: ParamQueueType + Send> Sync for ParamQueue<T> {}

impl<T: ParamQueueType> ParamQueue<T> {
    /// Drain pending events and return a reference to the current state.
    ///
    /// # Safety
    ///
    /// Must only be called from the audio thread. Calling from an other
    /// thread is undefined behavior.
    pub fn value(&self) -> &T {
        let value = unsafe { &mut *self.cache.get() };
        while let Some(event) = self.queue.pop() {
            value.handle_event(event);
        }
        value
    }

    /// Read-only snapshot of the current state for the UI thread.
    ///
    /// The returned value is a clone — the UI thread must not hold
    /// a reference into the cache. Safe to call while the audio
    /// thread is processing.
    ///
    /// # Safety
    ///
    /// This must be called ONCE per UI initialization BEFORE
    /// any event is pushed into the internal queue. Anything
    /// beyond that is undefined behavior
    pub fn snapshot(&self) -> T {
        // This shouldn't be poping anything as
        // UI is the only one to write events, and audio thread
        // should have drain all events already, but still.
        while let Some(_) = self.queue.pop() {}
        unsafe { (*self.cache.get()).clone() }
    }

    pub fn push_event(&self, event: <T as ParamQueueType>::EventType) {
        self.queue.force_push(event);
    }
}

impl<T: ParamQueueType + Serialize + DeserializeOwned> Persistent for ParamQueue<T> {
    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        let value = unsafe { &*self.cache.get() };
        serde_json::to_writer(writer, value).map_err(|_| PluginError::Message("serialize error"))
    }

    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: T = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;

        // Drain any pending events — they're stale now
        while self.queue.pop().is_some() {}
        unsafe { *self.cache.get() = value }
        Ok(())
    }
}
