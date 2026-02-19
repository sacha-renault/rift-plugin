use crate::utils::value_watcher::ValueWatcherU32;

pub(crate) struct HostsMessages {
    latency: ValueWatcherU32,
}

impl Default for HostsMessages {
    fn default() -> Self {
        Self {
            latency: ValueWatcherU32::new(0),
        }
    }
}

impl HostsMessages {
    pub fn set_latency(&self, latency: u32) {
        self.latency.set_value(latency);
    }

    pub fn take_latency_changed(&self) -> bool {
        self.latency.take_changed()
    }

    pub fn current_latency(&self) -> u32 {
        self.latency.value()
    }
}
