use crate::{MainThreadTask, main_thread_context_traits::HostStatesGetter};

#[allow(private_bounds)]
pub trait ChangeLatencyImpl: HostStatesGetter {
    fn set_latency(&mut self, latency: u32) {
        let latency_task = MainThreadTask::ChangeLatency(latency);
        if self.states().post_to_main(latency_task).is_ok() {
            self.increment_event_count();
        } else {
            log::error!("Couldn't push MainThreadTask::ChangeLatency({latency})")
        }
    }
}

impl<'a> ChangeLatencyImpl for super::InitContext<'a> {}
impl<'a, 'e> ChangeLatencyImpl for super::ProcessContext<'a, 'e> {}
