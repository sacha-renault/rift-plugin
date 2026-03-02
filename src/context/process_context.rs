use std::sync::Arc;

use clack_plugin::{host::HostAudioProcessorHandle, process::Process};
use hug_shared::BlockInfo;

use crate::wrapper::{ClapPlugin, shared_states::PluginSharedState};

pub struct ProcessContext<'a, P: ClapPlugin> {
    pub(crate) host: &'a HostAudioProcessorHandle<'a>,
    pub(crate) states: Arc<PluginSharedState>,
    pub(crate) shared: Arc<P::SharedType>,
    pub(crate) process: Process<'a>,
    pub(crate) samplerate: f64,

    /// THIS must be init to 0 or the entire thing is broken
    pub(crate) num_events: usize,
}

impl<'a, P: ClapPlugin> ProcessContext<'a, P> {
    pub fn shared(&self) -> Arc<P::SharedType> {
        Arc::clone(&self.shared)
    }

    pub fn block_info(&self) -> Option<BlockInfo> {
        if let Some(transport) = self.process.transport {
            let info = BlockInfo {
                seconds: transport.song_pos_seconds.to_float(),
                beats: transport.song_pos_beats.to_float(),
                samplerate: self.samplerate,
                seconds_per_beat: transport.tempo,
            };
            Some(info)
        } else {
            None
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
