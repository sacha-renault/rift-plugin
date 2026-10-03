use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

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
    main_thread_tasks: ArrayQueue<MainThreadTask>,
    audio_thread_tasks: ArrayQueue<AudioThreadTask>,
    gui_tasks: ArrayQueue<GuiTask>,
}

impl PluginSharedState {
    pub fn new(task_capacity: usize) -> Self {
        Self {
            latency: AtomicU32::new(0),
            is_playing: Arc::new(AtomicBool::new(false)),
            main_thread_tasks: ArrayQueue::new(task_capacity),
            audio_thread_tasks: ArrayQueue::new(task_capacity),
            samplerate: Arc::new(AtomicF64::new(44100.)),
            gui_tasks: ArrayQueue::new(task_capacity),
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
        self.main_thread_tasks.push(task)
    }

    pub fn pop_in_main(&self) -> Option<MainThreadTask> {
        self.main_thread_tasks.pop()
    }

    /// Posts a task to be executed by the audio thread from the audio thread.
    ///
    /// Returns an error if the [`Self::audio_thread_tasks`] queue is full.
    #[inline]
    pub fn post_to_audio(&self, task: AudioThreadTask) -> Result<(), AudioThreadTask> {
        self.audio_thread_tasks.push(task)
    }

    pub fn pop_in_audio(&self) -> Option<AudioThreadTask> {
        self.audio_thread_tasks.pop()
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
}
