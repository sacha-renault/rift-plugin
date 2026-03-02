use core::f64;
use std::sync::Arc;

use clack_plugin::{host::HostAudioProcessorHandle, process::Process};

use crate::wrapper::{ClapPlugin, shared_states::PluginSharedState};

pub struct ProcessContext<'a, P: ClapPlugin> {
    pub(crate) host: &'a HostAudioProcessorHandle<'a>,
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) shared: Arc<P::SharedType>,
    pub(crate) process: Process<'a>,
    pub(crate) num_events: usize,
}

impl<'a, P: ClapPlugin> ProcessContext<'a, P> {
    pub fn shared(&self) -> Arc<P::SharedType> {
        Arc::clone(&self.shared)
    }

    pub fn seconds(&self) -> f64 {
        self.process
            .transport
            .map(|t| t.song_pos_seconds.to_float())
            .unwrap_or(f64::NAN)
    }

    pub fn beats(&self) -> f64 {
        self.process
            .transport
            .map(|t| t.song_pos_beats.to_float())
            .unwrap_or(f64::NAN)
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
