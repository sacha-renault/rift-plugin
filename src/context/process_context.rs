use std::sync::Arc;

use clack_plugin::events::event_types::{MidiEvent, TransportFlags};
use clack_plugin::host::HostAudioProcessorHandle;
use clack_plugin::prelude::OutputEvents;
use clack_plugin::process::Process;

use rift_plugin_core::transport::{BlockIndex, BlockInfo};

use crate::prelude::MidiMessage;
use crate::wrapper::{ClapPlugin, shared_states::SharedQueues};

pub struct ProcessContext<'a, 'e, P: ClapPlugin> {
    pub(crate) host: &'a HostAudioProcessorHandle<'a>,
    pub(crate) states: Arc<SharedQueues>,
    pub(crate) shared: Arc<P::SharedType>,
    pub(crate) process: Process<'a>,
    pub(crate) samplerate: f64,
    pub(crate) block_index: BlockIndex,

    /// Count of pending events to be drained. MUST be initialized to 0; dropping with >0
    /// triggers a callback request to the host via the destructor.
    pub(crate) num_events: usize,
    pub(crate) outputs_events: &'e mut OutputEvents<'e>,
}

impl<'a, 'e, P: ClapPlugin> ProcessContext<'a, 'e, P> {
    pub fn shared(&self) -> Arc<P::SharedType> {
        Arc::clone(&self.shared)
    }

    /// Returns playback progress info (seconds/beats) if currently playing, otherwise None.
    pub fn block_info(&self) -> Option<BlockInfo> {
        if let Some(transport) = self.process.transport {
            let info = BlockInfo {
                seconds: transport.song_pos_seconds.to_float(),
                beats: transport.song_pos_beats.to_float(),
                samplerate: self.samplerate,
                tempo: transport.tempo,
                flags: TransportFlags::IS_PLAYING,
            };
            Some(info)
        } else {
            None
        }
    }

    pub fn block_index(&self) -> BlockIndex {
        self.block_index
    }

    /// Add a midi message as output event
    pub fn add_output_midi_event(&mut self, event: MidiMessage) {
        let _ = self.outputs_events.try_push::<MidiEvent>(event.into());
    }
}

impl<'a, 'e, P: ClapPlugin> super::HostStatesGetter for ProcessContext<'a, 'e, P> {
    #[inline]
    fn increment_event_count(&mut self) {
        self.num_events += 1;
    }

    #[inline]
    fn states(&self) -> Arc<SharedQueues> {
        self.states.clone()
    }
}

impl<'a, 'e, P: ClapPlugin> Drop for ProcessContext<'a, 'e, P> {
    fn drop(&mut self) {
        if self.num_events > 0 {
            // Drains the event count buffer by requesting a final callback on drop.
            self.host.request_callback();
        }
    }
}
