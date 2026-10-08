use std::{cell::RefCell, collections::HashMap};

use crate::dev_prelude::*;

#[derive(Clone, Copy)]
struct Entry {
    ptr: ParamPtr,
    /// Normalized value of the parameter, in `[0, 1]`.
    signal: Signal<f32>,
}

pub struct ParamRegistry {
    entries: RefCell<HashMap<ClapId, Entry>>,
}

// The registry is only touched from the GUI thread
impl Model for ParamRegistry {}

impl ParamRegistry {
    pub(crate) fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
        }
    }

    /// Fetches the registry from the view tree.
    ///
    /// # Panics
    /// If the GUI wasn't created through [`crate::vizia_gui`].
    pub(crate) fn from_context(cx: &impl DataContext) -> &Self {
        cx.try_data::<Self>().expect(
            "ParamRegistry not found: parameter widgets can only be used in a GUI \
             created with `vizia_gui`",
        )
    }

    /// Returns the signal of `ptr`, creating it on first use.
    pub(crate) fn register(&self, ptr: ParamPtr) -> Signal<f32> {
        let mut entries = self.entries.borrow_mut();
        entries
            .entry(ptr.id())
            .or_insert_with(|| Entry {
                ptr,
                signal: Signal::new(ptr.normalized()),
            })
            .signal
    }

    /// Applies a value coming from the host (automation, preset load, ...).
    ///
    /// `plain` is the un-normalized value, as sent by the host.
    pub(crate) fn apply_host_value(&self, id: ClapId, plain: f32) {
        // Copy the entry out so the borrow is released before the signal
        // notifies its subscribers.
        let entry = self.entries.borrow().get(&id).copied();
        if let Some(Entry { ptr, signal }) = entry {
            signal.set(ptr.normalize(plain));
        }
    }
}

pub fn register_param<P: Param>(cx: &mut Context, param: &P) -> Signal<f32> {
    ParamRegistry::from_context(cx).register(param.as_ptr())
}
