use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use crossbeam_queue::ArrayQueue;
use rift_plugin_gui::GuiTasks as GuiTask;
use rift_plugin_params::AtomicF64;

use crate::tasks::{AudioThreadTask, MainThreadTask};

/// Shared state for an audio plugin accessed by both the main and audio threads.
///
/// Contains bounded queues for posting tasks to the opposite thread.
/// If a queue is full when pushing a task, the push operation fails and returns
/// the unposted task in an `Err`.
pub struct PluginSharedState {
    pub(crate) latency: AtomicU32,
    pub(crate) is_playing: Arc<AtomicBool>,
    pub(crate) samplerate: Arc<AtomicF64>,

    /// Queues that audio / main thread can read
    main_tasks: ArrayQueue<MainThreadTask>,
    audio_tasks: ArrayQueue<AudioThreadTask>,
    gui_tasks: ArrayQueue<GuiTask>,

    overflows: AtomicQueueOverflows,
    gui_opened: AtomicBool,
}

impl PluginSharedState {
    pub fn new(task_capacity: usize) -> Self {
        Self {
            latency: AtomicU32::new(0),
            is_playing: Arc::new(AtomicBool::new(false)),
            samplerate: Arc::new(AtomicF64::new(44100.)),

            main_tasks: ArrayQueue::new(task_capacity),
            audio_tasks: ArrayQueue::new(task_capacity),
            gui_tasks: ArrayQueue::new(task_capacity),
            overflows: AtomicQueueOverflows::default(),
            gui_opened: AtomicBool::new(false),
        }
    }

    /// Updates the plugin's processing latency in samples.
    #[inline]
    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.latency.load(Ordering::Relaxed)
    }

    /// Posts a task to be executed by the main thread from the audio thread.
    ///
    /// Returns an error if the [`Self::main_thread_tasks`] queue is full, and
    /// records the overflow so the consumer can observe that a task was dropped.
    #[inline]
    pub fn post_to_main(&self, task: MainThreadTask) -> Result<(), MainThreadTask> {
        let result = self.main_tasks.push(task);
        if result.is_err() {
            self.overflows.set_main_overflow();
        }
        result
    }

    pub fn pop_in_main(&self) -> Option<MainThreadTask> {
        self.main_tasks.pop()
    }

    /// Posts a task to be executed by the audio thread from the audio thread.
    ///
    /// Returns an error if the [`Self::audio_thread_tasks`] queue is full, and
    /// records the overflow so the consumer can observe that a task was dropped.
    #[inline]
    pub fn post_to_audio(&self, task: AudioThreadTask) -> Result<(), AudioThreadTask> {
        let result = self.audio_tasks.push(task);
        if result.is_err() {
            self.overflows.set_audio_overflow();
        }
        result
    }

    pub fn pop_in_audio(&self) -> Option<AudioThreadTask> {
        self.audio_tasks.pop()
    }

    pub fn post_to_gui(&self, task: GuiTask) -> Result<(), GuiTask> {
        let result = self.gui_tasks.push(task);
        if result.is_err() {
            self.overflows.set_gui_overflow();
        }
        result
    }

    pub fn pop_in_gui(&self) -> Option<GuiTask> {
        self.gui_tasks.pop()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn set_is_playing(&self, value: bool) {
        self.is_playing.store(value, Ordering::Relaxed);
    }

    pub fn samplerate(&self) -> f64 {
        self.samplerate.load(Ordering::Relaxed)
    }

    #[doc(hidden)]
    pub fn set_samplerate(&self, value: f64) {
        self.samplerate.store(value, Ordering::Relaxed);
    }

    /// Flags raised when a `post_to_*` had to drop a task.
    pub fn overflows(&self) -> &AtomicQueueOverflows {
        &self.overflows
    }

    pub fn is_gui_opened(&self) -> bool {
        self.gui_opened.load(Ordering::Acquire)
    }

    pub fn set_gui_opened(&self, is_opened: bool) {
        self.gui_opened.store(is_opened, Ordering::Release);
    }
}

bitflags::bitflags! {
    /// Which task queue(s) dropped a task because it was full.
    ///
    /// One bit per queue, so a single atomic can hold all of them at once.
    pub struct QueueOverflows: u8 {
        const MAIN  = 1 << 0;
        const AUDIO = 1 << 1;
        const GUI   = 1 << 2;
    }
}

/// Atomic [`QueueOverflows`], set by the producer when a `post_to_*` fails.
///
/// Sticky by design: `set_*` latches, `take_*` reads and clears (returning
/// whether it was set). The consumer polls `take_*` and resyncs on `true`.
/// Dropped update means the UI can no longer trust its own state. Or the
/// Main thread might need a flush / callback.
///
/// `set` is `Release` and `take` is `Acquire`, so a resync triggered by a
/// `take` sees whatever the producer wrote before the drop.
pub struct AtomicQueueOverflows(AtomicU8);

impl Default for AtomicQueueOverflows {
    fn default() -> Self {
        Self(AtomicU8::new(0))
    }
}

impl AtomicQueueOverflows {
    fn set(&self, mask: QueueOverflows) {
        self.0.fetch_or(mask.bits(), Ordering::Release);
    }

    fn take_overflow(&self, mask: QueueOverflows) -> bool {
        let bitmask = mask.bits();
        let previous = self.0.fetch_and(!bitmask, Ordering::Acquire);
        previous & bitmask != 0
    }

    pub fn set_main_overflow(&self) {
        self.set(QueueOverflows::MAIN)
    }

    pub fn take_main_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::MAIN)
    }

    pub fn set_audio_overflow(&self) {
        self.set(QueueOverflows::AUDIO)
    }

    pub fn take_audio_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::AUDIO)
    }

    pub fn set_gui_overflow(&self) {
        self.set(QueueOverflows::GUI)
    }

    pub fn take_gui_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::GUI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clack_plugin::utils::ClapId;

    #[test]
    fn set_then_take() {
        let overflow = AtomicQueueOverflows::default();

        // Nothing pending yet.
        assert!(!overflow.take_main_overflow());

        overflow.set_main_overflow();
        assert!(overflow.take_main_overflow());

        // Taking already cleared it.
        assert!(!overflow.take_main_overflow());
    }

    #[test]
    fn flags_are_independent() {
        let overflow = AtomicQueueOverflows::default();
        overflow.set_audio_overflow();

        // Only the audio bit got latched.
        assert!(overflow.take_audio_overflow());
        assert!(!overflow.take_main_overflow());
        assert!(!overflow.take_gui_overflow());

        // Setting the others leaves a fresh audio bit alone.
        overflow.set_main_overflow();
        overflow.set_gui_overflow();
        assert!(!overflow.take_audio_overflow());
        assert!(overflow.take_main_overflow());
        assert!(overflow.take_gui_overflow());
    }

    #[test]
    fn take_clears_only_its_own_bit() {
        let overflow = AtomicQueueOverflows::default();
        overflow.set_main_overflow();
        overflow.set_gui_overflow();

        // Taking main doesn't consume gui...
        assert!(overflow.take_main_overflow());
        assert!(overflow.take_gui_overflow());

        // ...so now both are cleared.
        assert!(!overflow.take_main_overflow());
        assert!(!overflow.take_gui_overflow());

        // Audio behaves the same.
        assert!(!overflow.take_audio_overflow());
        overflow.set_audio_overflow();
        assert!(overflow.take_audio_overflow());
        assert!(!overflow.take_audio_overflow());
    }

    #[test]
    fn posting_to_a_full_queue_sets_its_overflow_flag() {
        // Capacity of one, so the second push into any queue must fail.
        let state = PluginSharedState::new(1);

        // Main queue only.
        assert!(state.post_to_main(MainThreadTask::RequestRestart).is_ok());
        assert!(state.post_to_main(MainThreadTask::RequestRestart).is_err());
        assert!(state.overflows().take_main_overflow());
        assert!(!state.overflows().take_audio_overflow());
        assert!(!state.overflows().take_gui_overflow());

        // Audio queue only.
        assert!(
            state
                .post_to_audio(AudioThreadTask::RequestCallback)
                .is_ok()
        );
        assert!(
            state
                .post_to_audio(AudioThreadTask::RequestCallback)
                .is_err()
        );
        assert!(state.overflows().take_audio_overflow());
        assert!(!state.overflows().take_gui_overflow());

        // Gui queue only.
        let gui = GuiTask::ParamChanged {
            id: ClapId::new(0),
            value: 1.0,
        };
        assert!(state.post_to_gui(gui).is_ok());
        assert!(
            state
                .post_to_gui(GuiTask::ParamChanged {
                    id: ClapId::new(0),
                    value: 1.0,
                })
                .is_err()
        );
        assert!(state.overflows().take_gui_overflow());
    }
}
