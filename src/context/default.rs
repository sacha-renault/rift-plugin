use crate::context::{MainThreadTask, main_thread_context_traits::HostStatesGetter};

#[allow(private_bounds)]
pub trait RequestRestartImpl: HostStatesGetter {
    fn request_restart(&mut self) {
        let latency_task = MainThreadTask::RequestRestart;
        if let Ok(_) = self.states().push_main_thread_task(latency_task) {
            self.increment_event_count();
        } else {
            log::error!("Couldn't push MainThreadTask::RequestRestart")
        }
    }
}

impl<'a> RequestRestartImpl for super::InitContext<'a> {}
impl<'a> RequestRestartImpl for super::ProcessContext<'a> {}
