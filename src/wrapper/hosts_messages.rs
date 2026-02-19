use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Default)]
pub(crate) struct HostsMessages {
    latency_changed: AtomicBool,
    current_latency: AtomicU32,
}

impl HostsMessages {
    pub fn set_latency_changed(&self, latency: u32) {
        self.current_latency.store(latency, Ordering::Relaxed);
        self.latency_changed.store(true, Ordering::Relaxed);
    }

    pub fn take_latency_changed(&self) -> bool {
        self.latency_changed.swap(false, Ordering::Relaxed)
    }

    pub fn current_latency(&self) -> u32 {
        self.current_latency.load(Ordering::Relaxed)
    }
}
