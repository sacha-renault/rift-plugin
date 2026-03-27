use std::{cell::UnsafeCell, sync::Arc};

use clack_plugin::{plugin::PluginError, utils::ClapId};
use crossbeam_queue::ArrayQueue;
use serde::{Serialize, de::DeserializeOwned};

use crate::params::NamedParam;

pub use super::Persistent;

pub trait ParamQueueType {
    type EventType;

    fn handle_event(&mut self, event: Self::EventType);
    fn snapshot(&self) -> Self;
}

/// Shared handle to a parameter queue.
///
/// Cheaply cloneable - the UI thread holds its own `ParamQueue<T>`
/// pointing to the same inner state. Both sides interact through
/// the lock-free queue; only the audio thread touches the cache.
#[derive(Clone)]
pub struct ParamQueue<T: ParamQueueType> {
    inner: Arc<ParamQueueInner<T>>,
}

pub struct ParamQueueInner<T: ParamQueueType> {
    cache: UnsafeCell<T>,
    queue: ArrayQueue<T::EventType>,

    id: ClapId,
    name: String,
    module: Option<String>,
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
    pub fn new(default: T, queue_capacity: usize) -> Self {
        Self {
            inner: Arc::new(ParamQueueInner {
                cache: UnsafeCell::new(default),
                queue: ArrayQueue::new(queue_capacity),
                name: String::from(""),
                module: None,
                id: ClapId::new(0),
            }),
        }
    }
    /// Drain pending events and return a reference to the current state.
    ///
    /// # Safety
    ///
    /// Must only be called from the audio thread. Calling from an other
    /// thread is undefined behavior.
    pub fn value(&self) -> &T {
        let value = unsafe { &mut *self.inner.cache.get() };
        while let Some(event) = self.inner.queue.pop() {
            value.handle_event(event);
        }
        value
    }

    /// Read-only snapshot of the current state for the UI thread.
    ///
    /// The returned value is a clone - the UI thread must not hold
    /// a reference into the cache. Safe to call while the audio
    /// thread is processing.
    ///
    /// # Safety
    ///
    /// This must be called ONCE per UI initialization BEFORE
    /// any event is pushed into the internal queue. Anything
    /// beyond that is undefined behavior. This is the reason this
    /// function is unsafe.
    pub unsafe fn snapshot(&self) -> T {
        // This shouldn't be poping anything as
        // UI is the only one to write events, and audio thread
        // should have drain all events already, but still.
        while self.inner.queue.pop().is_some() {}
        unsafe { (&*self.inner.cache.get()).snapshot() }
    }

    /// Push an event from the UI thread to be processed by the audio thread.
    ///
    /// Returns `Err(event)` if the queue is full, allowing the caller to
    /// skip applying the event locally and stay in sync with the audio thread.
    ///
    /// ```ignore
    /// if param_queue.push_event(event).is_ok() {
    ///     local_state.handle_event(event);
    /// }
    /// ```
    pub fn push_event(
        &self,
        event: <T as ParamQueueType>::EventType,
    ) -> Result<(), <T as ParamQueueType>::EventType> {
        self.inner.queue.push(event)
    }
}

impl<T: ParamQueueType> NamedParam for ParamQueue<T> {
    fn id(&self) -> clack_plugin::prelude::ClapId {
        self.inner.id
    }

    fn module(&self) -> Option<&str> {
        self.inner.module.as_deref()
    }

    fn name(&self) -> &str {
        &self.inner.name
    }
}

impl<T: ParamQueueType + Serialize + DeserializeOwned> Persistent for ParamQueue<T> {
    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), PluginError> {
        let value = unsafe { &*self.inner.cache.get() };
        serde_json::to_writer(writer, value).map_err(|_| PluginError::Message("serialize error"))
    }

    fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), PluginError> {
        let value: T = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("deserialize error"))?;

        // Drain any pending events - they're stale now
        while self.inner.queue.pop().is_some() {}
        unsafe { *self.inner.cache.get() = value }
        Ok(())
    }
}

impl<T: ParamQueueType> super::__ParamInitializer for ParamQueue<T> {
    fn __initialize(
        &mut self,
        name: String,
        id: clack_plugin::prelude::ClapId,
        module: Option<String>,
    ) {
        let Some(inner) = Arc::get_mut(&mut self.inner) else {
            log::error!("ParamQueue must be initialized before cloning : {name}");
            panic!("ParamQueue must be initialized before cloning");
        };
        inner.name = name;
        inner.module = module;
        inner.id = id;
    }
}
