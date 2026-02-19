use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub(crate) struct PluginSharedState {
    request_restart: AtomicBool,
    latency: AtomicU32,
}

impl Default for PluginSharedState {
    fn default() -> Self {
        Self {
            request_restart: AtomicBool::new(false),
            latency: AtomicU32::new(0),
        }
    }
}

impl PluginSharedState {
    #[inline]
    pub fn request_restart(&self) {
        self.request_restart.store(true, Ordering::Relaxed);
    }

    #[inline]
    pub fn take_request_restart(&self) -> bool {
        self.request_restart.swap(false, Ordering::Relaxed)
    }

    #[inline]
    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.latency.load(Ordering::Relaxed)
    }
}
