use clack_extensions::latency::HostLatency;

use crate::context::main_thread_context_traits::MainThreadContextGetter;

#[allow(private_bounds)]
pub trait ChangeLatency: MainThreadContextGetter {
    fn set_latency(&mut self, latency: u32) {
        if let Some(ext) = self.host().get_extension::<HostLatency>() {
            self.states().set_latency(latency);
            ext.changed(self.host_mut());
        }
    }
}

impl<T> ChangeLatency for T where T: MainThreadContextGetter {}
