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
    /// Returns an error if the [`Self::main_thread_tasks`] queue is full.
    #[inline]
    pub fn post_to_main(&self, task: MainThreadTask) -> Result<(), MainThreadTask> {
        self.main_tasks.push(task)
    }

    pub fn pop_in_main(&self) -> Option<MainThreadTask> {
        self.main_tasks.pop()
    }

    /// Posts a task to be executed by the audio thread from the audio thread.
    ///
    /// Returns an error if the [`Self::audio_thread_tasks`] queue is full.
    #[inline]
    pub fn post_to_audio(&self, task: AudioThreadTask) -> Result<(), AudioThreadTask> {
        self.audio_tasks.push(task)
    }

    pub fn pop_in_audio(&self) -> Option<AudioThreadTask> {
        self.audio_tasks.pop()
    }

    pub fn post_to_gui(&self, task: GuiTask) -> Result<(), GuiTask> {
        self.gui_tasks.push(task)
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

    pub fn overflows(&self) -> &AtomicQueueOverflows {
        &self.overflows
    }
}

bitflags::bitflags! {
    pub struct QueueOverflows: u8 {
        const MAIN  = 1 << 0;
        const AUDIO = 1 << 1;
        const GUI   = 1 << 2;
    }
}

pub struct AtomicQueueOverflows(AtomicU8);

impl Default for AtomicQueueOverflows {
    fn default() -> Self {
        Self(AtomicU8::new(0))
    }
}

impl AtomicQueueOverflows {
    fn set(&self, mask: QueueOverflows) {
        self.0.fetch_or(mask.bits(), Ordering::Relaxed);
    }

    fn is_overflow(&self, mask: QueueOverflows) -> bool {
        let value = self.0.load(Ordering::Relaxed);
        QueueOverflows::from_bits_retain(value).intersects(mask)
    }

    fn take_overflow(&self, mask: QueueOverflows) -> bool {
        let bitmask = mask.bits();
        let previous = self.0.fetch_and(!bitmask, Ordering::Relaxed);
        previous & bitmask != 0
    }

    pub fn set_main_overflow(&self) {
        self.set(QueueOverflows::MAIN)
    }

    pub fn is_main_overflow(&self) -> bool {
        self.is_overflow(QueueOverflows::MAIN)
    }

    pub fn take_main_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::MAIN)
    }

    pub fn set_audio_overflow(&self) {
        self.set(QueueOverflows::AUDIO)
    }

    pub fn is_audio_overflow(&self) -> bool {
        self.is_overflow(QueueOverflows::AUDIO)
    }

    pub fn take_audio_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::AUDIO)
    }

    pub fn set_gui_overflow(&self) {
        self.set(QueueOverflows::GUI)
    }

    pub fn is_gui_overflow(&self) -> bool {
        self.is_overflow(QueueOverflows::GUI)
    }

    pub fn take_gui_overflow(&self) -> bool {
        self.take_overflow(QueueOverflows::GUI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_then_is_overflow() {
        let overflow = AtomicQueueOverflows::default();
        assert!(!overflow.is_main_overflow());

        overflow.set_main_overflow();
        assert!(overflow.is_main_overflow());
    }

    #[test]
    fn flags_are_independent() {
        let overflow = AtomicQueueOverflows::default();
        overflow.set_audio_overflow();

        assert!(overflow.is_audio_overflow());
        assert!(!overflow.is_main_overflow());
        assert!(!overflow.is_gui_overflow());

        overflow.set_gui_overflow();
        assert!(overflow.is_audio_overflow());
        assert!(overflow.is_gui_overflow());
        assert!(!overflow.is_main_overflow());
    }

    #[test]
    fn take_returns_previous_and_clears_only_its_bit() {
        let overflow = AtomicQueueOverflows::default();
        overflow.set_main_overflow();
        overflow.set_gui_overflow();

        // First take observes and clears the main flag...
        assert!(overflow.take_main_overflow());
        assert!(!overflow.is_main_overflow());
        // ...without disturbing the gui flag.
        assert!(overflow.is_gui_overflow());

        // Taking an unset flag returns false and is idempotent.
        assert!(!overflow.take_main_overflow());
        assert!(!overflow.take_main_overflow());
        assert!(overflow.is_gui_overflow());

        // The audio and gui variants behave the same way.
        assert!(!overflow.take_audio_overflow());
        overflow.set_audio_overflow();
        assert!(overflow.take_audio_overflow());
        assert!(!overflow.is_audio_overflow());

        assert!(overflow.take_gui_overflow());
        assert!(!overflow.is_gui_overflow());
    }
}
