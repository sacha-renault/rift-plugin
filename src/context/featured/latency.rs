use crate::context::{MainThreadTasks, main_thread_context_traits::HostStatesGetter};

#[allow(private_bounds)]
pub trait ChangeLatencyImpl: HostStatesGetter {
    fn set_latency(&mut self, latency: u32) {
        let latency_task = MainThreadTasks::ChangeLatency(latency);
        if let Ok(_) = self.states().push_main_thread_task(latency_task) {
            self.increment_event_count();
        } else {
            log::error!("Couldn't push MainThreadTask::ChangeLatency({latency})")
        }
    }
}

impl<'a> ChangeLatencyImpl for super::InitContext<'a> {}
impl<'a> ChangeLatencyImpl for super::ProcessContext<'a> {}
