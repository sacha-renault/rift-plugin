use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use atomic_bitflags::AtomicFlags;
use bitflags::bitflags;
use rift_plugin_gui::{GuiParamEvent, HostParamChange};
use rift_plugin_params::AtomicF64;
use rift_plugin_types::{TaskCapacity, TaskQueue};
use rift_plugin_utils::DEFAULT_SAMPLERATE;

use crate::tasks::ParamContextMenu;

const REL: Ordering = Ordering::Relaxed;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SharedStateFlags: u32 {
        const IS_GUI_OPENED = 1 << 1;
        const IS_PLAYING = 1 << 2;
        const PARAM_RESCAN = 1 << 3;
    }
}

/// Shared state for an audio plugin accessed by both the main and audio threads.
///
/// Contains bounded queues for posting tasks to the opposite thread. Full queues
/// evict the oldest task; see [`TaskQueue`].
pub struct PluginSharedState {
    pub(crate) latency: AtomicU32,
    pub(crate) samplerate: Arc<AtomicF64>,

    /// Queues that audio / main thread can read
    context_menu_requests: TaskQueue<ParamContextMenu>,
    param_events: TaskQueue<GuiParamEvent>,
    gui_param_changes: TaskQueue<HostParamChange>,

    states_flags: AtomicFlags<SharedStateFlags>,
}

impl PluginSharedState {
    pub fn new(
        TaskCapacity {
            gui_param_changes,
            param_events,
            context_menu,
        }: TaskCapacity,
    ) -> Self {
        Self {
            latency: AtomicU32::new(0),
            samplerate: Arc::new(AtomicF64::new(DEFAULT_SAMPLERATE)),

            context_menu_requests: TaskQueue::new(context_menu),
            param_events: TaskQueue::new(param_events),
            gui_param_changes: TaskQueue::new(gui_param_changes),

            states_flags: AtomicFlags::new(SharedStateFlags::empty()),
        }
    }

    /// Updates the plugin's processing latency in samples.
    #[inline]
    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, REL);
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.latency.load(REL)
    }

    /// Parameter edits posted by the GUI for the audio thread.
    ///
    /// Drained by the audio processor, or by the main-thread flush when the
    /// plugin is inactive.
    #[inline]
    pub fn param_events(&self) -> &TaskQueue<GuiParamEvent> {
        &self.param_events
    }

    /// Host-side parameter changes the GUI has not applied yet.
    ///
    /// Posted by the audio thread for automation, or by the main thread for
    /// inactive flushes. Drained by the GUI; a full queue evicts the oldest
    /// change, which the GUI detects through [`TaskQueue::take_overflow`].
    #[inline]
    pub fn gui_param_changes(&self) -> &TaskQueue<HostParamChange> {
        &self.gui_param_changes
    }

    /// Asks the GUI to reload every parameter value, e.g. after a state load.
    pub fn request_param_rescan(&self) {
        self.states_flags.set(SharedStateFlags::PARAM_RESCAN, REL);
    }

    /// Consumes a pending rescan request. Returns `true` at most once per request.
    pub fn take_param_rescan(&self) -> bool {
        !self
            .states_flags
            .take(SharedStateFlags::PARAM_RESCAN, REL)
            .is_empty()
    }

    /// Context-menu requests posted by the GUI for the main thread.
    #[inline]
    pub fn context_menu_requests(&self) -> &TaskQueue<ParamContextMenu> {
        &self.context_menu_requests
    }

    pub fn is_playing(&self) -> bool {
        self.states_flags
            .load(Ordering::Relaxed)
            .contains(SharedStateFlags::IS_PLAYING)
    }

    pub fn set_is_playing(&self, is_playing: bool) {
        if is_playing {
            self.states_flags.set(SharedStateFlags::IS_PLAYING, REL)
        } else {
            self.states_flags.unset(SharedStateFlags::IS_PLAYING, REL)
        };
    }

    pub fn samplerate(&self) -> f64 {
        self.samplerate.load(Ordering::Relaxed)
    }

    #[doc(hidden)]
    pub fn set_samplerate(&self, value: f64) {
        self.samplerate.store(value, Ordering::Relaxed);
    }

    pub fn is_gui_opened(&self) -> bool {
        self.states_flags
            .load(Ordering::Relaxed)
            .contains(SharedStateFlags::IS_GUI_OPENED)
    }

    pub fn set_gui_opened(&self, is_opened: bool) {
        if is_opened {
            self.states_flags.set(SharedStateFlags::IS_GUI_OPENED, REL)
        } else {
            self.states_flags
                .unset(SharedStateFlags::IS_GUI_OPENED, REL)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param_rescan_is_reported_once() {
        let states = PluginSharedState::new(TaskCapacity::small());
        assert!(!states.take_param_rescan());

        // Repeated requests collapse into a single pending one.
        states.request_param_rescan();
        states.request_param_rescan();
        assert!(states.take_param_rescan());
        assert!(!states.take_param_rescan());
    }
}
