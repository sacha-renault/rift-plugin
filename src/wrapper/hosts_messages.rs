use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub(crate) struct HostsMessages {
    latency_changed: AtomicBool,
}

impl HostsMessages {
    pub fn set_latency_changed(&self) {
        self.latency_changed.store(true, Ordering::Relaxed);
    }

    pub fn take_latency_changed(&self) -> bool {
        self.latency_changed.swap(false, Ordering::Relaxed)
    }
}
