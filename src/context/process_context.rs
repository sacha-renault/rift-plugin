use std::sync::Arc;

use clack_plugin::host::HostAudioProcessorHandle;

use crate::wrapper::{ClapPlugin, shared_states::PluginSharedState};

pub struct ProcessContext<'a, P: ClapPlugin> {
    host: &'a HostAudioProcessorHandle<'a>,
    states: Arc<PluginSharedState>,
    shared: Arc<P::SharedType>,
    num_events: usize,
}

impl<'a, P: ClapPlugin> ProcessContext<'a, P> {
    pub(crate) fn new(
        host: &'a HostAudioProcessorHandle<'a>,
        host_messages: Arc<PluginSharedState>,
        shared: Arc<P::SharedType>,
    ) -> Self {
        Self {
            host,
            states: host_messages,
            shared,
            num_events: 0,
        }
    }

    pub fn shared(&self) -> Arc<P::SharedType> {
        Arc::clone(&self.shared)
    }

    pub fn push_in_accumulator(
        &self,
        idx: usize,
        slices: impl Iterator<Item = &'a [f32]>,
    ) -> Result<(), ()> {
        if let Some(acc) = self.states.audio_accumulators.get(idx).as_ref() {
            acc.push_slices(slices);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<'a, P: ClapPlugin> super::HostStatesGetter for ProcessContext<'a, P> {
    #[inline]
    fn increment_event_count(&mut self) {
        self.num_events += 1;
    }

    #[inline]
    fn states(&self) -> Arc<PluginSharedState> {
        self.states.clone()
    }
}

impl<'a, P: ClapPlugin> Drop for ProcessContext<'a, P> {
    fn drop(&mut self) {
        if self.num_events > 0 {
            self.host.request_callback();
        }
    }
}
